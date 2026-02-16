//! Telemetry module for CORE Runtime.
//!
//! Provides structured logging, distributed tracing, and metrics collection.
//! All output is file-based or via existing IPC - no network dependencies.

mod logging;
mod metrics;
pub mod security_log;
mod spans;
mod store;

pub use logging::{init_logging, LogConfig, LogError, LogFormat};
pub use metrics::{
    init_metrics, record_memory_pool, record_queue_depth, record_request_failure,
    record_request_success, record_speculative_cycle,
};
pub use security_log::{log_security_event, SecurityEvent, SecuritySeverity};
pub use spans::{RequestSpan, SpanExt};
pub use store::{HistogramSummary, MetricsSnapshot, MetricsStore};
