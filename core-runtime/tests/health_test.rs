//! Health check tests for CORE Runtime.

use core_runtime::health::{HealthChecker, HealthConfig, HealthState};
use core_runtime::ipc::{
    decode_message, encode_message, HealthCheckResponse, HealthCheckType, IpcMessage,
};
use core_runtime::shutdown::ShutdownState;

// ============================================================================
// Health Checker Tests
// ============================================================================

#[test]
fn test_alive_always_true() {
    let checker = HealthChecker::default();
    assert!(checker.is_alive());
}

#[test]
fn test_ready_when_running() {
    let checker = HealthChecker::default();
    assert!(checker.is_ready(ShutdownState::Running, 0, 0));
}

#[test]
fn test_not_ready_when_draining() {
    let checker = HealthChecker::default();
    assert!(!checker.is_ready(ShutdownState::Draining, 0, 0));
}

#[test]
fn test_not_ready_when_stopped() {
    let checker = HealthChecker::default();
    assert!(!checker.is_ready(ShutdownState::Stopped, 0, 0));
}

#[test]
fn test_ready_respects_model_requirement() {
    let config = HealthConfig {
        require_model_loaded: true,
        max_queue_depth: 1000,
    };
    let checker = HealthChecker::new(config);

    // Not ready when no models loaded
    assert!(!checker.is_ready(ShutdownState::Running, 0, 0));

    // Ready when model is loaded
    assert!(checker.is_ready(ShutdownState::Running, 1, 0));
}

#[test]
fn test_not_ready_when_queue_full() {
    let config = HealthConfig {
        require_model_loaded: false,
        max_queue_depth: 10,
    };
    let checker = HealthChecker::new(config);

    // Ready when queue is below limit
    assert!(checker.is_ready(ShutdownState::Running, 0, 9));

    // Not ready when queue is at limit
    assert!(!checker.is_ready(ShutdownState::Running, 0, 10));
}

#[test]
fn test_report_includes_all_fields() {
    let checker = HealthChecker::default();
    let report = checker.report(ShutdownState::Running, 2, 1024, 5);

    assert_eq!(report.state, HealthState::Healthy);
    assert!(report.ready);
    assert!(report.accepting_requests);
    assert_eq!(report.models_loaded, 2);
    assert_eq!(report.memory_used_bytes, 1024);
    assert_eq!(report.queue_depth, 5);
    // uptime_secs will be 0 or very small in tests
}

// ============================================================================
// Protocol Roundtrip Tests
// ============================================================================

#[test]
fn test_health_check_liveness_roundtrip() {
    let message = IpcMessage::HealthCheck {
        check_type: HealthCheckType::Liveness,
    };

    let encoded = encode_message(&message).unwrap();
    let decoded = decode_message(&encoded).unwrap();

    match decoded {
        IpcMessage::HealthCheck { check_type } => {
            assert_eq!(check_type, HealthCheckType::Liveness);
        }
        _ => panic!("Expected HealthCheck message"),
    }
}

#[test]
fn test_health_check_readiness_roundtrip() {
    let message = IpcMessage::HealthCheck {
        check_type: HealthCheckType::Readiness,
    };

    let encoded = encode_message(&message).unwrap();
    let decoded = decode_message(&encoded).unwrap();

    match decoded {
        IpcMessage::HealthCheck { check_type } => {
            assert_eq!(check_type, HealthCheckType::Readiness);
        }
        _ => panic!("Expected HealthCheck message"),
    }
}

#[test]
fn test_health_check_full_roundtrip() {
    let message = IpcMessage::HealthCheck {
        check_type: HealthCheckType::Full,
    };

    let encoded = encode_message(&message).unwrap();
    let decoded = decode_message(&encoded).unwrap();

    match decoded {
        IpcMessage::HealthCheck { check_type } => {
            assert_eq!(check_type, HealthCheckType::Full);
        }
        _ => panic!("Expected HealthCheck message"),
    }
}

#[test]
fn test_health_response_roundtrip() {
    let response = HealthCheckResponse {
        check_type: HealthCheckType::Liveness,
        ok: true,
        report: None,
    };
    let message = IpcMessage::HealthResponse(response);

    let encoded = encode_message(&message).unwrap();
    let decoded = decode_message(&encoded).unwrap();

    match decoded {
        IpcMessage::HealthResponse(resp) => {
            assert_eq!(resp.check_type, HealthCheckType::Liveness);
            assert!(resp.ok);
            assert!(resp.report.is_none());
        }
        _ => panic!("Expected HealthResponse message"),
    }
}
