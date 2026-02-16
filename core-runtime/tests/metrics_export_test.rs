//! Tests for metrics export via IPC.

use veritas_sdr::ipc::{decode_message, encode_message, IpcMessage, MetricsSnapshot};
use veritas_sdr::telemetry::{HistogramSummary, MetricsStore};

// ============================================================================
// MetricsStore Tests
// ============================================================================

#[test]
fn test_counter_increment() {
    let store = MetricsStore::new();

    store.increment_counter("test_counter", 1);
    store.increment_counter("test_counter", 5);

    let snapshot = store.snapshot();
    assert_eq!(snapshot.counters.get("test_counter"), Some(&6));
}

#[test]
fn test_counter_multiple_names() {
    let store = MetricsStore::new();

    store.increment_counter("counter_a", 10);
    store.increment_counter("counter_b", 20);

    let snapshot = store.snapshot();
    assert_eq!(snapshot.counters.get("counter_a"), Some(&10));
    assert_eq!(snapshot.counters.get("counter_b"), Some(&20));
}

#[test]
fn test_gauge_set() {
    let store = MetricsStore::new();

    store.set_gauge("test_gauge", 42.5);
    let snapshot = store.snapshot();
    assert_eq!(snapshot.gauges.get("test_gauge"), Some(&42.5));

    // Overwrite with new value
    store.set_gauge("test_gauge", 100.0);
    let snapshot = store.snapshot();
    assert_eq!(snapshot.gauges.get("test_gauge"), Some(&100.0));
}

#[test]
fn test_histogram_single_value() {
    let store = MetricsStore::new();

    store.record_histogram("latency", 50.0);

    let snapshot = store.snapshot();
    let hist = snapshot.histograms.get("latency").unwrap();
    assert_eq!(hist.count, 1);
    assert_eq!(hist.sum, 50.0);
    assert_eq!(hist.min, 50.0);
    assert_eq!(hist.max, 50.0);
}

#[test]
fn test_histogram_multiple_values() {
    let store = MetricsStore::new();

    store.record_histogram("latency", 10.0);
    store.record_histogram("latency", 20.0);
    store.record_histogram("latency", 30.0);

    let snapshot = store.snapshot();
    let hist = snapshot.histograms.get("latency").unwrap();
    assert_eq!(hist.count, 3);
    assert_eq!(hist.sum, 60.0);
    assert_eq!(hist.min, 10.0);
    assert_eq!(hist.max, 30.0);
}

#[test]
fn test_histogram_min_max_tracking() {
    let store = MetricsStore::new();

    store.record_histogram("test", 100.0);
    store.record_histogram("test", 5.0);
    store.record_histogram("test", 50.0);
    store.record_histogram("test", 200.0);
    store.record_histogram("test", 1.0);

    let snapshot = store.snapshot();
    let hist = snapshot.histograms.get("test").unwrap();
    assert_eq!(hist.min, 1.0);
    assert_eq!(hist.max, 200.0);
}

#[test]
fn test_empty_snapshot() {
    let store = MetricsStore::new();
    let snapshot = store.snapshot();

    assert!(snapshot.counters.is_empty());
    assert!(snapshot.gauges.is_empty());
    assert!(snapshot.histograms.is_empty());
}

#[test]
fn test_snapshot_is_immutable() {
    let store = MetricsStore::new();

    store.increment_counter("counter", 10);
    let snapshot1 = store.snapshot();

    store.increment_counter("counter", 5);
    let snapshot2 = store.snapshot();

    // First snapshot should not change
    assert_eq!(snapshot1.counters.get("counter"), Some(&10));
    // Second snapshot has updated value
    assert_eq!(snapshot2.counters.get("counter"), Some(&15));
}

// ============================================================================
// Protocol Roundtrip Tests
// ============================================================================

#[test]
fn test_metrics_request_roundtrip() {
    let message = IpcMessage::MetricsRequest;

    let encoded = encode_message(&message).unwrap();
    let decoded = decode_message(&encoded).unwrap();

    match decoded {
        IpcMessage::MetricsRequest => {}
        _ => panic!("Expected MetricsRequest message"),
    }
}

#[test]
fn test_metrics_response_roundtrip() {
    let mut counters = std::collections::HashMap::new();
    counters.insert("requests_total".to_string(), 100);

    let mut gauges = std::collections::HashMap::new();
    gauges.insert("memory_bytes".to_string(), 1024.0);

    let mut histograms = std::collections::HashMap::new();
    histograms.insert(
        "latency_ms".to_string(),
        HistogramSummary {
            count: 50,
            sum: 500.0,
            min: 1.0,
            max: 50.0,
        },
    );

    let snapshot = MetricsSnapshot {
        counters,
        gauges,
        histograms,
    };
    let message = IpcMessage::MetricsResponse(snapshot);

    let encoded = encode_message(&message).unwrap();
    let decoded = decode_message(&encoded).unwrap();

    match decoded {
        IpcMessage::MetricsResponse(snap) => {
            assert_eq!(snap.counters.get("requests_total"), Some(&100));
            assert_eq!(snap.gauges.get("memory_bytes"), Some(&1024.0));

            let hist = snap.histograms.get("latency_ms").unwrap();
            assert_eq!(hist.count, 50);
            assert_eq!(hist.sum, 500.0);
            assert_eq!(hist.min, 1.0);
            assert_eq!(hist.max, 50.0);
        }
        _ => panic!("Expected MetricsResponse message"),
    }
}

#[test]
fn test_metrics_response_empty_snapshot() {
    let snapshot = MetricsSnapshot {
        counters: std::collections::HashMap::new(),
        gauges: std::collections::HashMap::new(),
        histograms: std::collections::HashMap::new(),
    };
    let message = IpcMessage::MetricsResponse(snapshot);

    let encoded = encode_message(&message).unwrap();
    let decoded = decode_message(&encoded).unwrap();

    match decoded {
        IpcMessage::MetricsResponse(snap) => {
            assert!(snap.counters.is_empty());
            assert!(snap.gauges.is_empty());
            assert!(snap.histograms.is_empty());
        }
        _ => panic!("Expected MetricsResponse message"),
    }
}
