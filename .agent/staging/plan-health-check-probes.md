# Plan: Health Check Probes

## Open Questions

1. **Authentication**: Should health checks require authentication? (Suggested: No - orchestrators need unauthenticated probes)
2. **Model Requirement**: Should readiness require at least one model loaded? (Suggested: Configurable, default false)

## Strategic Summary

**Why**: Enable orchestrators (Kubernetes, systemd) to verify runtime health and readiness for traffic routing decisions.

**Vibe**: stateless, composable, observable

## Current State Analysis

Existing infrastructure:
- `IpcMessage` enum - Tagged message types for IPC
- `IpcHandler` - Message routing with auth checks
- `ShutdownState` - Running/Draining/Stopped state machine
- `ModelRegistry` - Model count and memory tracking
- `RequestQueue` - Pending request count

**Gap**: No health/readiness message types. No aggregated health status.

---

## Phase 1: Health Status Types

### Affected Files

- `src/health.rs` - NEW: Health status types
- `src/lib.rs` - Export health module

### Changes

**src/health.rs** (NEW ~60 lines)

```rust
/// Overall health status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthState {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Detailed health report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    pub state: HealthState,
    pub ready: bool,
    pub accepting_requests: bool,
    pub models_loaded: usize,
    pub memory_used_bytes: usize,
    pub queue_depth: usize,
    pub uptime_secs: u64,
}

/// Health check configuration.
#[derive(Debug, Clone)]
pub struct HealthConfig {
    pub require_model_loaded: bool,
    pub max_queue_depth: usize,
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            require_model_loaded: false,
            max_queue_depth: 1000,
        }
    }
}

/// Aggregates health information from runtime components.
pub struct HealthChecker {
    config: HealthConfig,
    start_time: Instant,
}

impl HealthChecker {
    pub fn new(config: HealthConfig) -> Self;

    /// Check liveness: process is responsive.
    pub fn is_alive(&self) -> bool;

    /// Check readiness: accepting traffic.
    pub fn is_ready(&self, shutdown_state: ShutdownState, models: usize, queue: usize) -> bool;

    /// Generate full health report.
    pub fn report(&self, ...) -> HealthReport;
}
```

**src/lib.rs** - Add health export

```rust
pub mod health;

pub use health::{HealthChecker, HealthConfig, HealthReport, HealthState};
```

### Unit Tests

- `tests/health_test.rs` - NEW
  - `test_alive_always_true` - Liveness is always true
  - `test_ready_when_running` - Ready when ShutdownState::Running
  - `test_not_ready_when_draining` - Not ready during shutdown
  - `test_not_ready_when_stopped` - Not ready after stopped
  - `test_ready_respects_model_requirement` - Config flag enforced
  - `test_not_ready_when_queue_full` - Queue depth check
  - `test_report_includes_all_fields` - Full report structure

---

## Phase 2: IPC Protocol Extension

### Affected Files

- `src/ipc/protocol.rs` - Add health message types
- `src/ipc/mod.rs` - Export new types

### Changes

**src/ipc/protocol.rs** - Add health messages

```rust
/// Health check request types.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum HealthCheckType {
    Liveness,
    Readiness,
    Full,
}

/// Health check response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    pub check_type: HealthCheckType,
    pub ok: bool,
    pub report: Option<HealthReport>,
}

/// All possible IPC message types.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum IpcMessage {
    // ... existing variants ...

    #[serde(rename = "health_check")]
    HealthCheck { check_type: HealthCheckType },

    #[serde(rename = "health_response")]
    HealthResponse(HealthCheckResponse),
}
```

### Unit Tests

- `tests/protocol_test.rs` - ADD
  - `test_health_check_liveness_roundtrip` - Serialize/deserialize liveness
  - `test_health_check_readiness_roundtrip` - Serialize/deserialize readiness
  - `test_health_check_full_roundtrip` - Serialize/deserialize full report
  - `test_health_response_roundtrip` - Response serialization

---

## Phase 3: Handler Integration

### Affected Files

- `src/ipc/handler.rs` - Add health check handling
- `src/lib.rs` - Add health checker to Runtime

### Changes

**src/lib.rs** - Add health checker to Runtime

```rust
pub struct Runtime {
    // ... existing fields ...
    pub health: HealthChecker,
}

impl Runtime {
    pub fn new(config: RuntimeConfig) -> Self {
        let health = HealthChecker::new(HealthConfig::default());
        // ...
    }
}
```

**src/ipc/handler.rs** - Add health check handler

```rust
pub struct IpcHandler {
    // ... existing fields ...
    health: Arc<HealthChecker>,
    model_registry: Arc<ModelRegistry>,
}

impl IpcHandler {
    async fn handle_message(&self, message: IpcMessage, ...) -> ... {
        match message {
            // ... existing matches ...

            IpcMessage::HealthCheck { check_type } => {
                // NO AUTH REQUIRED for health checks
                let response = self.handle_health_check(check_type).await;
                Ok((IpcMessage::HealthResponse(response), None))
            }
        }
    }

    async fn handle_health_check(&self, check_type: HealthCheckType) -> HealthCheckResponse {
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
                let report = self.health.report(shutdown_state, models, queue, ...);
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

### Unit Tests

- `tests/health_test.rs` - ADD
  - `test_handler_liveness_no_auth` - Liveness works without auth
  - `test_handler_readiness_no_auth` - Readiness works without auth
  - `test_handler_full_returns_report` - Full check returns report
  - `test_handler_readiness_false_during_shutdown` - Draining returns not ready

---

## Section 4 Compliance Check

| File | Estimated Lines | Limit | Status |
|------|-----------------|-------|--------|
| health.rs | ~60 | 250 | OK |
| protocol.rs | ~155 | 250 | OK |
| handler.rs | ~175 | 250 | OK |
| lib.rs | ~130 | 250 | OK |
| health_test.rs | ~120 | 250 | OK |

## Risk Assessment

| Phase | Risk Grade | Justification |
|-------|------------|---------------|
| Health Status Types | L1 | Pure data structures, no logic |
| Protocol Extension | L1 | Adding enum variants, serialization |
| Handler Integration | L2 | New handler path, bypasses auth |

**Overall Risk Grade**: L2 (Logic changes - auth bypass for health)

## Dependencies

No new external dependencies required.

---

_Plan follows Simple Made Easy principles: health checking is a separate concern from inference, composed via IpcMessage variants._
