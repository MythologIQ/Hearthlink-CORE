# Plan: Runtime Enhancements Bundle

Five composable features following Simple Made Easy principles.

## Open Questions

1. **Connection limits**: Per-session or global? → Global with configurable max
2. **Deduplication TTL**: Should output cache use same TTL as KV cache? → Separate, shorter TTL (30s default)
3. **Warmup token count**: Fixed or configurable? → Configurable via request, default 1

---

## Phase 0: Handler Split (Remediation)

**Reason**: handler.rs is at 242 lines (96.8% of 250 limit). Must split before adding new handlers.

### Strategy

Extract health check handling into dedicated `health_handler.rs` module (~37 lines extracted).

### Affected Files

- `src/ipc/handler.rs` - MODIFIED: Remove handle_health_check method
- `src/ipc/health_handler.rs` - NEW: HealthHandler with handle_health_check
- `src/ipc/mod.rs` - MODIFIED: Export HealthHandler

### Changes

**health_handler.rs** - New health handler module:
```rust
//! Health check request handling.

use super::protocol::{HealthCheckResponse, HealthCheckType};
use crate::health::HealthChecker;
use crate::models::ModelRegistry;
use crate::scheduler::RequestQueue;
use crate::shutdown::ShutdownCoordinator;
use std::sync::Arc;

/// Handles health check requests (orchestrator pattern, no auth).
pub struct HealthHandler {
    health: Arc<HealthChecker>,
    shutdown: Arc<ShutdownCoordinator>,
    model_registry: Arc<ModelRegistry>,
    queue: Arc<RequestQueue>,
}

impl HealthHandler {
    pub fn new(
        health: Arc<HealthChecker>,
        shutdown: Arc<ShutdownCoordinator>,
        model_registry: Arc<ModelRegistry>,
        queue: Arc<RequestQueue>,
    ) -> Self {
        Self { health, shutdown, model_registry, queue }
    }

    pub async fn handle(&self, check_type: HealthCheckType) -> HealthCheckResponse {
        let shutdown_state = self.shutdown.state().await;
        let models = self.model_registry.count().await;
        let queue = self.queue.len().await;

        match check_type {
            HealthCheckType::Liveness => HealthCheckResponse {
                check_type,
                ok: self.health.is_alive(),
                report: None,
            },
            HealthCheckType::Readiness => HealthCheckResponse {
                check_type,
                ok: self.health.is_ready(shutdown_state, models, queue),
                report: None,
            },
            HealthCheckType::Full => {
                let memory = self.model_registry.total_memory().await;
                let report = self.health.report(shutdown_state, models, memory, queue);
                HealthCheckResponse {
                    check_type,
                    ok: report.ready,
                    report: Some(report),
                }
            }
        }
    }
}
```

**handler.rs** - Delegate to HealthHandler:
```rust
// Remove handle_health_check method (~27 lines)
// Add HealthHandler field to IpcHandler struct
// Update constructor to accept HealthHandler
// In handle_message, delegate: IpcMessage::HealthCheck { check_type } => {
//     let response = self.health_handler.handle(check_type).await;
//     Ok((IpcMessage::HealthResponse(response), None))
// }
```

### Projected Line Counts

| File | Before | After |
|------|--------|-------|
| handler.rs | 242 | ~205 |
| health_handler.rs | 0 | ~55 |

**Headroom for new features**: ~45 lines in handler.rs

---

## Phase 1: Request Timeout & Cancellation

Adds deadline tracking and cancellation to the request pipeline.

### Affected Files

- `src/engine/inference.rs` - Add `timeout_ms` field to InferenceParams
- `src/scheduler/queue.rs` - Add `deadline`, `cancelled` to QueuedRequest + cancel method
- `src/ipc/protocol.rs` - Add `CancelRequest` message type
- `src/ipc/handler.rs` - Handle `CancelRequest`
- `tests/timeout_cancel_test.rs` - 8 tests

### Changes

**inference.rs** - Add timeout field:
```rust
pub struct InferenceParams {
    // ... existing fields ...
    /// Request timeout in milliseconds. None = no timeout.
    #[serde(default)]
    pub timeout_ms: Option<u64>,
}
```

**queue.rs** - Extend QueuedRequest and add cancellation:
```rust
use std::sync::atomic::AtomicBool;
use std::time::Instant;

pub struct QueuedRequest {
    pub id: u64,
    pub model_id: String,
    pub prompt_tokens: Vec<u32>,
    pub params: InferenceParams,
    pub enqueued_at: Instant,
    pub deadline: Option<Instant>,
    cancelled: Arc<AtomicBool>,
}

impl QueuedRequest {
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }

    pub fn is_expired(&self) -> bool {
        self.deadline.map_or(false, |d| Instant::now() > d)
    }
}

impl RequestQueue {
    /// Cancel a pending request by ID. Returns true if found.
    pub async fn cancel(&self, request_id: u64) -> bool {
        let queue = self.queue.lock().await;
        // Iterate to find and mark cancelled
    }
}
```

**protocol.rs** - Add cancel message:
```rust
#[serde(rename = "cancel_request")]
CancelRequest { request_id: RequestId },

#[serde(rename = "cancel_response")]
CancelResponse { request_id: RequestId, cancelled: bool },
```

**handler.rs** - Handle cancel:
```rust
IpcMessage::CancelRequest { request_id } => {
    let cancelled = self.queue.cancel(request_id.0).await;
    Ok((IpcMessage::CancelResponse { request_id, cancelled }, None))
}
```

### Unit Tests

- `tests/timeout_cancel_test.rs`:
  - `test_request_with_timeout` - deadline computed from timeout_ms
  - `test_request_no_timeout` - deadline is None
  - `test_expired_request_skipped` - dequeue skips expired
  - `test_cancel_pending_request` - cancel returns true
  - `test_cancel_unknown_request` - cancel returns false
  - `test_cancel_message_roundtrip` - IPC serialization
  - `test_cancel_response_roundtrip` - IPC serialization
  - `test_cancelled_request_skipped` - dequeue skips cancelled

---

## Phase 2: Model Warm-up via IPC

Allows orchestrators to prime a model before accepting production traffic.

### Affected Files

- `src/ipc/protocol.rs` - Add `WarmupRequest`, `WarmupResponse`
- `src/ipc/handler.rs` - Handle warmup with minimal forward pass
- `tests/warmup_test.rs` - 5 tests

### Changes

**protocol.rs** - Add warmup messages:
```rust
#[serde(rename = "warmup_request")]
WarmupRequest {
    model_id: String,
    /// Number of tokens to generate (default: 1)
    #[serde(default = "default_warmup_tokens")]
    tokens: usize,
},

#[serde(rename = "warmup_response")]
WarmupResponse {
    model_id: String,
    success: bool,
    error: Option<String>,
    elapsed_ms: u64,
},

fn default_warmup_tokens() -> usize { 1 }
```

**handler.rs** - Handle warmup (no-auth, orchestrator pattern):
```rust
IpcMessage::WarmupRequest { model_id, tokens } => {
    // NO AUTH REQUIRED (orchestrator pattern, same as health/metrics)
    let start = std::time::Instant::now();
    let result = self.warmup_model(&model_id, tokens).await;
    let elapsed_ms = start.elapsed().as_millis() as u64;

    let response = match result {
        Ok(()) => WarmupResponse { model_id, success: true, error: None, elapsed_ms },
        Err(e) => WarmupResponse { model_id, success: false, error: Some(e), elapsed_ms },
    };
    Ok((IpcMessage::WarmupResponse(response), None))
}

async fn warmup_model(&self, model_id: &str, tokens: usize) -> Result<(), String> {
    // Enqueue minimal inference request, wait for completion
}
```

### Unit Tests

- `tests/warmup_test.rs`:
  - `test_warmup_request_defaults` - tokens defaults to 1
  - `test_warmup_request_roundtrip` - IPC serialization
  - `test_warmup_response_success` - success case
  - `test_warmup_response_error` - error case
  - `test_warmup_no_auth_required` - orchestrator pattern

---

## Phase 3: Request Deduplication

Returns cached output for identical prompts within TTL window.

### Affected Files

- `src/scheduler/dedup.rs` - NEW: OutputCache for response deduplication
- `src/scheduler/mod.rs` - Export OutputCache
- `src/scheduler/queue.rs` - Integrate dedup check on enqueue
- `tests/dedup_test.rs` - 7 tests

### Changes

**dedup.rs** - New output cache:
```rust
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct CachedOutput {
    pub output_tokens: Vec<u32>,
    pub cached_at: Instant,
}

pub struct OutputCache {
    entries: HashMap<[u8; 32], CachedOutput>,
    ttl: Duration,
    max_entries: usize,
}

impl OutputCache {
    pub fn new(ttl: Duration, max_entries: usize) -> Self { ... }

    /// Hash prompt tokens + params for cache key
    pub fn cache_key(tokens: &[u32], params: &InferenceParams) -> [u8; 32] {
        let mut hasher = Sha256::new();
        for &t in tokens { hasher.update(t.to_le_bytes()); }
        hasher.update(params.max_tokens.to_le_bytes());
        // Include temperature, top_p, top_k for exact match
        hasher.finalize().into()
    }

    /// Get cached output if within TTL
    pub fn get(&mut self, key: &[u8; 32]) -> Option<&CachedOutput> {
        if let Some(entry) = self.entries.get(key) {
            if entry.cached_at.elapsed() <= self.ttl {
                return Some(entry);
            }
            // Expired - remove lazily
        }
        None
    }

    /// Store output for future dedup
    pub fn insert(&mut self, key: [u8; 32], output_tokens: Vec<u32>) { ... }

    /// Remove expired entries
    pub fn cleanup(&mut self) { ... }
}
```

**queue.rs** - Add dedup integration:
```rust
pub struct RequestQueue {
    // ... existing ...
    output_cache: Arc<Mutex<OutputCache>>,
}

pub async fn enqueue_with_dedup(...) -> Result<DedupResult, QueueError> {
    let key = OutputCache::cache_key(&prompt_tokens, &params);

    // Check cache first
    let mut cache = self.output_cache.lock().await;
    if let Some(cached) = cache.get(&key) {
        return Ok(DedupResult::Cached(cached.output_tokens.clone()));
    }
    drop(cache);

    // Normal enqueue
    let (id, pos) = self.enqueue(model_id, prompt_tokens, params, priority).await?;
    Ok(DedupResult::Queued { id, position: pos })
}

pub enum DedupResult {
    Cached(Vec<u32>),
    Queued { id: u64, position: usize },
}
```

### Unit Tests

- `tests/dedup_test.rs`:
  - `test_cache_key_deterministic` - same input = same key
  - `test_cache_key_params_differ` - different params = different key
  - `test_cache_hit_within_ttl` - returns cached output
  - `test_cache_miss_after_ttl` - expired entry not returned
  - `test_cache_eviction_at_capacity` - oldest evicted
  - `test_dedup_result_queued` - new request queued
  - `test_dedup_result_cached` - duplicate returns cached

---

## Phase 4: Connection Management

Limits concurrent IPC connections with session-scoped tracking.

### Affected Files

- `src/ipc/connections.rs` - NEW: ConnectionPool with limits
- `src/ipc/mod.rs` - Export ConnectionPool
- `src/ipc/auth.rs` - Add connection tracking to SessionAuth
- `tests/connections_test.rs` - 6 tests

### Changes

**connections.rs** - New connection pool:
```rust
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

pub struct ConnectionConfig {
    pub max_connections: usize,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self { max_connections: 64 }
    }
}

pub struct ConnectionPool {
    active: AtomicUsize,
    config: ConnectionConfig,
}

impl ConnectionPool {
    pub fn new(config: ConnectionConfig) -> Self {
        Self { active: AtomicUsize::new(0), config }
    }

    /// Try to acquire a connection slot. Returns guard if available.
    pub fn try_acquire(&self) -> Option<ConnectionGuard> {
        let current = self.active.load(Ordering::Relaxed);
        if current >= self.config.max_connections {
            return None;
        }

        // CAS to atomically increment
        if self.active.compare_exchange(
            current, current + 1, Ordering::SeqCst, Ordering::Relaxed
        ).is_ok() {
            return Some(ConnectionGuard { pool: self });
        }
        None
    }

    pub fn active_count(&self) -> usize {
        self.active.load(Ordering::Relaxed)
    }
}

pub struct ConnectionGuard<'a> {
    pool: &'a ConnectionPool,
}

impl Drop for ConnectionGuard<'_> {
    fn drop(&mut self) {
        self.pool.active.fetch_sub(1, Ordering::SeqCst);
    }
}
```

**auth.rs** - Track connections per session:
```rust
struct Session {
    created_at: Instant,
    last_activity: Instant,
    connection_count: AtomicUsize,
}

impl SessionAuth {
    /// Increment connection count for session
    pub async fn track_connection(&self, token: &SessionToken) -> Result<(), AuthError> { ... }

    /// Decrement connection count for session
    pub async fn release_connection(&self, token: &SessionToken) { ... }
}
```

### Unit Tests

- `tests/connections_test.rs`:
  - `test_acquire_within_limit` - succeeds when under max
  - `test_acquire_at_limit` - fails when at max
  - `test_guard_releases_on_drop` - RAII decrement
  - `test_concurrent_acquire` - thread-safe acquisition
  - `test_session_connection_tracking` - per-session counts
  - `test_connection_config_defaults` - 64 max default

---

## Phase 5: Integration & Exports

Wire all features together and update public API.

### Affected Files

- `src/lib.rs` - Add OutputCache, ConnectionPool to Runtime
- `src/ipc/mod.rs` - Export new types
- `src/ipc/handler.rs` - Integrate all features
- `tests/runtime_enhancements_integration_test.rs` - 5 integration tests

### Changes

**lib.rs** - Extend Runtime:
```rust
pub struct Runtime {
    // ... existing ...
    pub output_cache: Arc<Mutex<OutputCache>>,
    pub connections: Arc<ConnectionPool>,
}
```

**ipc/mod.rs** - Exports:
```rust
pub use connections::{ConnectionConfig, ConnectionGuard, ConnectionPool};
pub use protocol::{CancelRequest, CancelResponse, WarmupRequest, WarmupResponse};
```

**handler.rs** - Integrate dedup in inference handling:
```rust
async fn handle_inference(&self, request: InferenceRequest) -> InferenceResponse {
    // ... existing validation ...

    // Check dedup cache
    let key = OutputCache::cache_key(&request.prompt_tokens, &request.parameters);
    if let Some(cached) = self.output_cache.lock().await.get(&key) {
        return InferenceResponse::success(request.request_id, cached.output_tokens.clone(), true);
    }

    // ... existing enqueue logic ...
}
```

### Unit Tests

- `tests/runtime_enhancements_integration_test.rs`:
  - `test_cancel_streaming_request` - cancel mid-stream
  - `test_timeout_during_queue` - request expires in queue
  - `test_warmup_then_inference` - warmup reduces latency
  - `test_dedup_across_sessions` - cache shared across sessions
  - `test_connection_limit_enforced` - rejects at max

---

## Summary

| Phase | Files Created | Files Modified | Tests |
|-------|--------------|----------------|-------|
| 0 | 1 source file | 2 source files | 0 |
| 1 | 1 test file | 4 source files | 8 |
| 2 | 1 test file | 2 source files | 5 |
| 3 | 1 source + 1 test | 2 source files | 7 |
| 4 | 1 source + 1 test | 2 source files | 6 |
| 5 | 1 test file | 3 source files | 5 |
| **Total** | **7 files** | **9 files** | **31** |

**Risk Grade**: L2 (logic changes, no security impact)

**Section 4 Compliance**:
- Phase 0: handler.rs reduced from 242 to ~205 lines (headroom created)
- All new files <100 lines projected
- handler.rs after all phases: ~205 + ~29 = ~234 lines (under 250 limit)
