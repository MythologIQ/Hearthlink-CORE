//! Telemetry module tests for CORE Runtime.

use core_runtime::telemetry::{
    init_metrics, record_memory_pool, record_queue_depth, record_request_failure,
    record_request_success, record_speculative_cycle, LogConfig, LogError, LogFormat, RequestSpan,
    SpanExt,
};
use std::path::PathBuf;
use tracing::Span;

// =============================================================================
// LogConfig Tests
// =============================================================================

#[test]
fn log_config_default_is_json() {
    let config = LogConfig::default();
    assert_eq!(config.format, LogFormat::Json);
    assert_eq!(config.level, "info");
    assert!(config.output_path.is_none());
}

#[test]
fn log_config_custom_level() {
    let config = LogConfig {
        format: LogFormat::Pretty,
        level: "debug".to_string(),
        output_path: None,
    };
    assert_eq!(config.format, LogFormat::Pretty);
    assert_eq!(config.level, "debug");
}

#[test]
fn log_config_with_output_path() {
    let config = LogConfig {
        format: LogFormat::Json,
        level: "trace".to_string(),
        output_path: Some(PathBuf::from("/tmp/test.log")),
    };
    assert_eq!(config.output_path, Some(PathBuf::from("/tmp/test.log")));
}

#[test]
fn log_format_equality() {
    assert_eq!(LogFormat::Json, LogFormat::Json);
    assert_eq!(LogFormat::Pretty, LogFormat::Pretty);
    assert_ne!(LogFormat::Json, LogFormat::Pretty);
}

// =============================================================================
// LogError Tests
// =============================================================================

#[test]
fn log_error_invalid_filter_display() {
    let error = LogError::InvalidFilter("bad filter".to_string());
    assert!(error.to_string().contains("Invalid log filter"));
    assert!(error.to_string().contains("bad filter"));
}

#[test]
fn log_error_file_open_display() {
    let error = LogError::FileOpen("permission denied".to_string());
    assert!(error.to_string().contains("Failed to open log file"));
    assert!(error.to_string().contains("permission denied"));
}

#[test]
fn log_error_already_initialized_display() {
    let error = LogError::AlreadyInitialized;
    assert!(error.to_string().contains("already initialized"));
}

// =============================================================================
// SpanExt Tests
// =============================================================================

#[test]
fn span_ext_record_result_ok() {
    let span = Span::none();
    let result: Result<i32, &str> = Ok(42);
    // Should not panic
    span.record_result(&result);
}

#[test]
fn span_ext_record_result_err() {
    let span = Span::none();
    let result: Result<i32, &str> = Err("test error");
    // Should not panic
    span.record_result(&result);
}

// =============================================================================
// RequestSpan Tests
// =============================================================================

#[test]
fn request_span_creates_span_without_panic() {
    // Without a subscriber, spans are disabled by default.
    // This test verifies creation doesn't panic.
    let span = RequestSpan::new("req-123", "model-abc");
    let _guard = span.enter();
    // Should not panic
}

#[test]
fn request_span_different_ids_create_without_panic() {
    // Verify multiple spans can be created
    let span1 = RequestSpan::new("req-1", "model-a");
    let span2 = RequestSpan::new("req-2", "model-b");
    let _guard1 = span1.enter();
    let _guard2 = span2.enter();
    // Should not panic
}

// =============================================================================
// Metrics Tests
// =============================================================================

#[test]
fn metrics_init_no_panic() {
    // Multiple calls should not panic
    init_metrics();
    init_metrics();
}

#[test]
fn record_request_success_no_panic() {
    record_request_success("test-model", 100, 50);
}

#[test]
fn record_request_failure_no_panic() {
    record_request_failure("test-model", "timeout");
}

#[test]
fn record_memory_pool_no_panic() {
    record_memory_pool(1024 * 1024);
    record_memory_pool(0);
}

#[test]
fn record_queue_depth_no_panic() {
    record_queue_depth(10);
    record_queue_depth(0);
}

#[test]
fn record_speculative_cycle_no_panic() {
    record_speculative_cycle(5, 2);
    record_speculative_cycle(0, 0);
}

#[test]
fn metrics_with_various_models() {
    record_request_success("phi-3", 50, 100);
    record_request_success("llama-3", 75, 200);
    record_request_failure("phi-3", "oom");
    record_request_failure("llama-3", "timeout");
}

#[test]
fn metrics_record_large_values() {
    record_request_success("large-model", u64::MAX / 2, u64::MAX / 2);
    record_memory_pool(usize::MAX / 2);
    record_queue_depth(usize::MAX / 2);
}

#[test]
fn metrics_record_zero_values() {
    record_request_success("zero-model", 0, 0);
    record_speculative_cycle(0, 0);
}

// =============================================================================
// Integration Tests
// =============================================================================

#[test]
fn span_with_metrics_integration() {
    let span = RequestSpan::new("integration-test", "test-model");
    let _guard = span.enter();

    // Simulate request processing
    record_queue_depth(1);
    record_memory_pool(1024);

    // Simulate success
    let result: Result<u64, &str> = Ok(42);
    span.record_result(&result);

    record_request_success("test-model", 100, 42);
    record_queue_depth(0);
}

#[test]
fn span_with_failure_metrics_integration() {
    let span = RequestSpan::new("fail-test", "fail-model");
    let _guard = span.enter();

    record_queue_depth(1);

    // Simulate failure
    let result: Result<u64, &str> = Err("test failure");
    span.record_result(&result);

    record_request_failure("fail-model", "test_failure");
    record_queue_depth(0);
}
