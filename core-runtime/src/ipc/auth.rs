//! Handshake token and session ID validation.
//!
//! SECURITY: This module enforces that only authenticated callers can
//! communicate with the runtime. All requests MUST have valid session.
//!
//! Security features:
//! - Constant-time token comparison (prevents timing attacks)
//! - CSPRNG session IDs (prevents session prediction)
//! - Rate limiting (prevents brute-force attacks)
//! - Session timeout (limits exposure window)
//! - Security audit logging (enables forensic analysis)

use crate::telemetry::{log_security_event, SecurityEvent};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::RwLock;

/// Maximum failed authentication attempts before rate limiting kicks in.
const MAX_FAILED_ATTEMPTS: u64 = 5;

/// Duration to block after too many failed attempts.
const RATE_LIMIT_DURATION: Duration = Duration::from_secs(30);

/// Duration to track failed attempts for rate limiting.
const ATTEMPT_WINDOW: Duration = Duration::from_secs(60);

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Invalid handshake token")]
    InvalidToken,

    #[error("Session not found")]
    SessionNotFound,

    #[error("Session expired")]
    SessionExpired,

    #[error("Authentication required")]
    NotAuthenticated,

    #[error("Too many failed attempts, please try again later")]
    RateLimited,
}

/// Validated session token from handshake.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SessionToken(String);

impl SessionToken {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

struct Session {
    created_at: Instant,
    last_activity: Instant,
    connection_count: AtomicUsize,
}

/// Rate limiter for authentication attempts.
struct RateLimiter {
    /// Number of failed attempts in the current window.
    failed_attempts: AtomicU64,
    /// Time of the first failed attempt in the current window.
    window_start: std::sync::Mutex<Option<Instant>>,
    /// Time until rate limiting expires (if active).
    blocked_until: std::sync::Mutex<Option<Instant>>,
}

impl RateLimiter {
    fn new() -> Self {
        Self {
            failed_attempts: AtomicU64::new(0),
            window_start: std::sync::Mutex::new(None),
            blocked_until: std::sync::Mutex::new(None),
        }
    }

    /// Check if authentication is currently rate limited.
    fn is_rate_limited(&self) -> bool {
        // Check if we're in a blocked period
        if let Ok(blocked_until) = self.blocked_until.lock() {
            if let Some(until) = *blocked_until {
                if Instant::now() < until {
                    return true;
                }
            }
        }
        false
    }

    /// Record a failed authentication attempt.
    fn record_failure(&self) {
        let now = Instant::now();

        // Check if we need to reset the window
        if let Ok(window_start) = self.window_start.lock() {
            let should_reset = window_start
                .map(|start| now.duration_since(start) > ATTEMPT_WINDOW)
                .unwrap_or(true);

            if should_reset {
                self.failed_attempts.store(1, Ordering::SeqCst);
                drop(window_start);
                if let Ok(mut ws) = self.window_start.lock() {
                    *ws = Some(now);
                }
                return;
            }
        }

        // Increment failed attempts
        let attempts = self.failed_attempts.fetch_add(1, Ordering::SeqCst) + 1;

        // Check if we should block
        if attempts >= MAX_FAILED_ATTEMPTS {
            if let Ok(mut blocked_until) = self.blocked_until.lock() {
                *blocked_until = Some(now + RATE_LIMIT_DURATION);
            }
        }
    }

    /// Reset rate limiting after successful authentication.
    fn reset(&self) {
        self.failed_attempts.store(0, Ordering::SeqCst);
        if let Ok(mut window_start) = self.window_start.lock() {
            *window_start = None;
        }
        if let Ok(mut blocked_until) = self.blocked_until.lock() {
            *blocked_until = None;
        }
    }
}

/// Manages session authentication.
pub struct SessionAuth {
    sessions: Arc<RwLock<HashMap<SessionToken, Session>>>,
    expected_token_hash: [u8; 32],
    session_timeout: Duration,
    rate_limiter: RateLimiter,
}

impl SessionAuth {
    /// Create new auth manager with expected handshake token.
    pub fn new(expected_token: &str, session_timeout: Duration) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(expected_token.as_bytes());
        let expected_token_hash: [u8; 32] = hasher.finalize().into();

        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            expected_token_hash,
            session_timeout,
            rate_limiter: RateLimiter::new(),
        }
    }

    /// Validate handshake token and create session.
    /// Implements rate limiting to prevent brute-force attacks.
    pub async fn authenticate(&self, token: &str) -> Result<SessionToken, AuthError> {
        // Check rate limiting first
        if self.rate_limiter.is_rate_limited() {
            log_security_event(
                SecurityEvent::RateLimited,
                "Authentication blocked due to rate limiting",
                &[("reason", "too_many_failures")],
            );
            return Err(AuthError::RateLimited);
        }

        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        let token_hash: [u8; 32] = hasher.finalize().into();

        if !constant_time_compare(token_hash.as_slice(), self.expected_token_hash.as_slice()) {
            // Record failed attempt for rate limiting
            self.rate_limiter.record_failure();
            log_security_event(
                SecurityEvent::AuthFailure,
                "Invalid handshake token",
                &[("reason", "invalid_token")],
            );
            return Err(AuthError::InvalidToken);
        }

        // Reset rate limiter on successful authentication
        self.rate_limiter.reset();

        let session_id = generate_session_id();
        let session_token = SessionToken(session_id);
        let now = Instant::now();

        self.sessions.write().await.insert(
            session_token.clone(),
            Session {
                created_at: now,
                last_activity: now,
                connection_count: AtomicUsize::new(0),
            },
        );

        log_security_event(
            SecurityEvent::AuthSuccess,
            "Authentication successful",
            &[("session_prefix", &session_token.as_str()[..8])],
        );

        Ok(session_token)
    }

    /// Validate session token and update activity.
    pub async fn validate(&self, token: &SessionToken) -> Result<(), AuthError> {
        let mut sessions = self.sessions.write().await;
        let session = sessions.get_mut(token).ok_or_else(|| {
            log_security_event(
                SecurityEvent::InvalidSession,
                "Invalid session token used",
                &[("session_prefix", &token.as_str()[..8])],
            );
            AuthError::SessionNotFound
        })?;

        if session.created_at.elapsed() > self.session_timeout {
            sessions.remove(token);
            log_security_event(
                SecurityEvent::SessionExpired,
                "Session expired",
                &[("session_prefix", &token.as_str()[..8])],
            );
            return Err(AuthError::SessionExpired);
        }

        session.last_activity = Instant::now();
        Ok(())
    }

    /// Remove expired sessions.
    pub async fn cleanup(&self) {
        let mut sessions = self.sessions.write().await;
        sessions.retain(|_, s| s.created_at.elapsed() <= self.session_timeout);
    }

    /// Increment connection count for session.
    pub async fn track_connection(&self, token: &SessionToken) -> Result<usize, AuthError> {
        let sessions = self.sessions.read().await;
        let session = sessions.get(token).ok_or(AuthError::SessionNotFound)?;
        let new_count = session.connection_count.fetch_add(1, Ordering::SeqCst) + 1;
        Ok(new_count)
    }

    /// Decrement connection count for session.
    pub async fn release_connection(&self, token: &SessionToken) {
        let sessions = self.sessions.read().await;
        if let Some(session) = sessions.get(token) {
            session.connection_count.fetch_sub(1, Ordering::SeqCst);
        }
    }

    /// Get current connection count for session.
    pub async fn connection_count(&self, token: &SessionToken) -> Result<usize, AuthError> {
        let sessions = self.sessions.read().await;
        let session = sessions.get(token).ok_or(AuthError::SessionNotFound)?;
        Ok(session.connection_count.load(Ordering::Relaxed))
    }
}

/// Constant-time comparison to prevent timing attacks.
fn constant_time_compare(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter()
        .zip(b.iter())
        .fold(0u8, |acc, (x, y)| acc | (x ^ y))
        == 0
}

/// Generate a cryptographically secure random session ID.
/// Uses OsRng for entropy from the operating system's CSPRNG.
fn generate_session_id() -> String {
    use rand::RngCore;

    // Generate 32 bytes of cryptographically secure random data
    let mut random_bytes = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(random_bytes.as_mut_slice());

    // Encode as hex for a 64-character session ID
    hex::encode(random_bytes)
}
