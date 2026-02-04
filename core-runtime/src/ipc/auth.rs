//! Handshake token and session ID validation.
//!
//! SECURITY: This module enforces that only authenticated callers can
//! communicate with the runtime. All requests MUST have valid session.

use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::RwLock;

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
}

/// Manages session authentication.
pub struct SessionAuth {
    sessions: Arc<RwLock<HashMap<SessionToken, Session>>>,
    expected_token_hash: [u8; 32],
    session_timeout: Duration,
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
        }
    }

    /// Validate handshake token and create session.
    pub async fn authenticate(&self, token: &str) -> Result<SessionToken, AuthError> {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        let token_hash: [u8; 32] = hasher.finalize().into();

        if !constant_time_compare(&token_hash, &self.expected_token_hash) {
            return Err(AuthError::InvalidToken);
        }

        let session_id = generate_session_id();
        let session_token = SessionToken(session_id);
        let now = Instant::now();

        self.sessions.write().await.insert(
            session_token.clone(),
            Session { created_at: now, last_activity: now },
        );

        Ok(session_token)
    }

    /// Validate session token and update activity.
    pub async fn validate(&self, token: &SessionToken) -> Result<(), AuthError> {
        let mut sessions = self.sessions.write().await;
        let session = sessions.get_mut(token).ok_or(AuthError::SessionNotFound)?;

        if session.created_at.elapsed() > self.session_timeout {
            sessions.remove(token);
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
}

/// Constant-time comparison to prevent timing attacks.
fn constant_time_compare(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b.iter()).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

fn generate_session_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    let mut hasher = Sha256::new();
    hasher.update(timestamp.to_le_bytes());
    hasher.update(std::process::id().to_le_bytes());
    hex::encode(hasher.finalize())
}
