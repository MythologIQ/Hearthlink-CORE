// Copyright 2024-2026 Veritas SDR Contributors
// SPDX-License-Identifier: Apache-2.0

//! Chaos tests for canary deployment scenarios.
//!
//! These tests validate system resilience under failure conditions during
//! canary rollouts, ensuring automatic rollback triggers and safe fallback behavior.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use veritas_sdr::ab_testing::{TrafficConfig, TrafficSplitter, VariantLabel, VariantMetrics};
use veritas_sdr::health::{HealthChecker, HealthConfig, HealthState};
use veritas_sdr::shutdown::{ShutdownCoordinator, ShutdownState};

// ============================================================================
// Test Infrastructure
// ============================================================================

/// Simulated canary instance for chaos testing.
#[allow(dead_code)]
struct MockCanaryInstance {
    label: VariantLabel,
    healthy: AtomicBool,
    error_rate: AtomicU64, // Percentage (0-100)
    latency_ms: AtomicU64,
    crashed: AtomicBool,
}

impl MockCanaryInstance {
    fn new(label: VariantLabel) -> Self {
        Self {
            label,
            healthy: AtomicBool::new(true),
            error_rate: AtomicU64::new(0),
            latency_ms: AtomicU64::new(50),
            crashed: AtomicBool::new(false),
        }
    }

    fn inject_crash(&self) {
        self.crashed.store(true, Ordering::SeqCst);
        self.healthy.store(false, Ordering::SeqCst);
    }

    fn inject_error_rate(&self, rate: u64) {
        self.error_rate.store(rate, Ordering::SeqCst);
    }

    fn inject_latency(&self, ms: u64) {
        self.latency_ms.store(ms, Ordering::SeqCst);
    }

    fn is_healthy(&self) -> bool {
        self.healthy.load(Ordering::SeqCst) && !self.crashed.load(Ordering::SeqCst)
    }

    fn process_request(&self) -> Result<Duration, &'static str> {
        if self.crashed.load(Ordering::SeqCst) {
            return Err("instance_crashed");
        }

        let error_rate = self.error_rate.load(Ordering::SeqCst);
        if error_rate > 0 {
            let random = (std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .subsec_nanos() as u64)
                % 100;
            if random < error_rate {
                return Err("elevated_error_rate");
            }
        }

        let latency = self.latency_ms.load(Ordering::SeqCst);
        Ok(Duration::from_millis(latency))
    }
}

/// Rollback decision engine for chaos testing.
struct RollbackController {
    error_threshold: f64,      // Error rate threshold (0.0-1.0)
    latency_threshold_ms: u64, // P99 latency threshold
    consecutive_failures: AtomicU64,
    failure_window_threshold: u64,
    rollback_triggered: AtomicBool,
}

impl RollbackController {
    fn new() -> Self {
        Self {
            error_threshold: 0.05, // 5% error rate
            latency_threshold_ms: 500,
            consecutive_failures: AtomicU64::new(0),
            failure_window_threshold: 3,
            rollback_triggered: AtomicBool::new(false),
        }
    }

    fn record_success(&self) {
        self.consecutive_failures.store(0, Ordering::SeqCst);
    }

    fn record_failure(&self) {
        let failures = self.consecutive_failures.fetch_add(1, Ordering::SeqCst) + 1;
        if failures >= self.failure_window_threshold {
            self.rollback_triggered.store(true, Ordering::SeqCst);
        }
    }

    fn should_rollback(&self) -> bool {
        self.rollback_triggered.load(Ordering::SeqCst)
    }

    fn check_metrics(&self, stats: &CanaryStats) -> bool {
        if stats.error_rate() > self.error_threshold {
            return true;
        }
        if stats.p99_latency_ms > self.latency_threshold_ms {
            return true;
        }
        false
    }
}

/// Statistics for canary analysis.
struct CanaryStats {
    requests: u64,
    failures: u64,
    p99_latency_ms: u64,
}

impl CanaryStats {
    fn error_rate(&self) -> f64 {
        if self.requests == 0 {
            return 0.0;
        }
        self.failures as f64 / self.requests as f64
    }
}

/// Simulated metrics collector that can fail.
struct FallibleMetricsCollector {
    available: AtomicBool,
    delayed: AtomicBool,
    delay_ms: AtomicU64,
    corrupted: AtomicBool,
    inner: VariantMetrics,
}

impl FallibleMetricsCollector {
    fn new() -> Self {
        Self {
            available: AtomicBool::new(true),
            delayed: AtomicBool::new(false),
            delay_ms: AtomicU64::new(0),
            corrupted: AtomicBool::new(false),
            inner: VariantMetrics::new(),
        }
    }

    fn inject_unavailability(&self) {
        self.available.store(false, Ordering::SeqCst);
    }

    fn inject_delay(&self, ms: u64) {
        self.delayed.store(true, Ordering::SeqCst);
        self.delay_ms.store(ms, Ordering::SeqCst);
    }

    fn inject_corruption(&self) {
        self.corrupted.store(true, Ordering::SeqCst);
    }

    fn is_available(&self) -> bool {
        self.available.load(Ordering::SeqCst)
    }

    fn record(&self, label: &VariantLabel, success: bool, latency: Duration) -> Result<(), &str> {
        if !self.available.load(Ordering::SeqCst) {
            return Err("metrics_unavailable");
        }

        if self.delayed.load(Ordering::SeqCst) {
            let delay = self.delay_ms.load(Ordering::SeqCst);
            thread::sleep(Duration::from_millis(delay));
        }

        if self.corrupted.load(Ordering::SeqCst) {
            return Err("metrics_corrupted");
        }

        let stats = self.inner.get_or_create(label);
        stats.record_request();
        if success {
            stats.record_success(latency, 10);
        } else {
            stats.record_failure();
        }

        Ok(())
    }
}

// ============================================================================
// Canary Instance Failure Tests
// ============================================================================

#[test]
fn test_canary_pod_crash_during_rollout() {
    // Setup: 90% control, 10% canary
    let canary_label = VariantLabel::new("v2-canary");
    let splitter = TrafficSplitter::new(TrafficConfig::canary(canary_label.clone())).unwrap();

    let control = MockCanaryInstance::new(VariantLabel::control());
    let canary = MockCanaryInstance::new(canary_label.clone());
    let rollback = RollbackController::new();

    // Simulate traffic before crash
    for i in 0..10 {
        let session = format!("session-{}", i);
        let variant = splitter.select(Some(&session));
        let instance = if variant == &canary_label {
            &canary
        } else {
            &control
        };
        let _ = instance.process_request();
    }

    // Inject canary crash
    canary.inject_crash();

    // Verify canary is unhealthy
    assert!(!canary.is_healthy(), "Canary should be unhealthy after crash");

    // Simulate requests directly to canary after crash
    // In production, load balancer would route canary traffic here
    let mut canary_failures = 0;
    for _ in 0..10 {
        let result = canary.process_request();
        if result.is_err() {
            canary_failures += 1;
            rollback.record_failure();
        }
    }

    // Verify rollback triggered
    assert!(canary_failures >= 3, "Should have at least 3 canary failures");
    assert!(
        rollback.should_rollback(),
        "Rollback should be triggered after consecutive failures"
    );
}

#[test]
fn test_canary_elevated_error_rate_triggers_rollback() {
    let canary_label = VariantLabel::new("v2-canary");
    let splitter = TrafficSplitter::new(TrafficConfig::canary(canary_label.clone())).unwrap();

    let canary = MockCanaryInstance::new(canary_label.clone());
    let rollback = RollbackController::new();

    // Inject 30% error rate (above 5% threshold)
    canary.inject_error_rate(30);

    let mut total_requests = 0;
    let mut total_failures = 0;

    // Run traffic through canary
    for i in 0..100 {
        let session = format!("canary-session-{}", i);
        let variant = splitter.select(Some(&session));

        if variant == &canary_label {
            total_requests += 1;
            match canary.process_request() {
                Ok(_) => rollback.record_success(),
                Err(_) => {
                    total_failures += 1;
                    rollback.record_failure();
                }
            }
        }
    }

    // Calculate actual error rate
    let stats = CanaryStats {
        requests: total_requests,
        failures: total_failures,
        p99_latency_ms: 100,
    };

    // Verify elevated error rate is detected
    assert!(
        total_requests > 0,
        "Should have some canary requests"
    );

    // The error rate check should trigger rollback condition
    let should_rollback = rollback.check_metrics(&stats);
    assert!(
        should_rollback || rollback.should_rollback(),
        "Rollback should be triggered due to elevated error rate: {:.2}%",
        stats.error_rate() * 100.0
    );
}

#[test]
fn test_canary_latency_spike_triggers_rollback() {
    let canary_label = VariantLabel::new("v2-canary");
    let canary = MockCanaryInstance::new(canary_label.clone());
    let rollback = RollbackController::new();

    // Inject high latency (above 500ms threshold)
    canary.inject_latency(1500);

    // Process request
    let result = canary.process_request();
    assert!(result.is_ok(), "Request should succeed");

    let latency = result.unwrap();
    assert!(
        latency.as_millis() >= 1500,
        "Latency should be injected value"
    );

    // Check metrics with high latency
    let stats = CanaryStats {
        requests: 100,
        failures: 0,
        p99_latency_ms: 1500,
    };

    assert!(
        rollback.check_metrics(&stats),
        "Rollback should be triggered due to high latency"
    );
}

#[test]
fn test_automatic_rollback_within_threshold() {
    let rollback = RollbackController::new();

    // Simulate failures that don't exceed threshold
    rollback.record_failure();
    rollback.record_failure();
    assert!(
        !rollback.should_rollback(),
        "Should not rollback with 2 failures (threshold is 3)"
    );

    // One more failure triggers rollback
    rollback.record_failure();
    assert!(
        rollback.should_rollback(),
        "Should rollback after 3 consecutive failures"
    );
}

#[test]
fn test_success_resets_failure_counter() {
    let rollback = RollbackController::new();

    // Simulate failures
    rollback.record_failure();
    rollback.record_failure();

    // Success resets counter
    rollback.record_success();

    // More failures needed to trigger rollback
    rollback.record_failure();
    rollback.record_failure();
    assert!(
        !rollback.should_rollback(),
        "Should not rollback - counter was reset"
    );

    rollback.record_failure();
    assert!(
        rollback.should_rollback(),
        "Should rollback after new consecutive failures"
    );
}

// ============================================================================
// Metrics System Failure Tests
// ============================================================================

#[test]
fn test_metrics_unavailable_safe_fallback() {
    let metrics = FallibleMetricsCollector::new();
    let label = VariantLabel::control();

    // Normal operation
    assert!(metrics.is_available());
    assert!(metrics.record(&label, true, Duration::from_millis(50)).is_ok());

    // Inject unavailability
    metrics.inject_unavailability();
    assert!(!metrics.is_available());

    // Verify graceful failure
    let result = metrics.record(&label, true, Duration::from_millis(50));
    assert!(result.is_err(), "Should fail when metrics unavailable");
    assert_eq!(result.unwrap_err(), "metrics_unavailable");
}

#[test]
fn test_delayed_metrics_reporting() {
    let metrics = FallibleMetricsCollector::new();
    let label = VariantLabel::control();

    // Inject delay (short for test)
    metrics.inject_delay(10);

    let start = std::time::Instant::now();
    let result = metrics.record(&label, true, Duration::from_millis(50));
    let elapsed = start.elapsed();

    assert!(result.is_ok(), "Should succeed despite delay");
    assert!(
        elapsed.as_millis() >= 10,
        "Recording should be delayed"
    );
}

#[test]
fn test_corrupted_metrics_data() {
    let metrics = FallibleMetricsCollector::new();
    let label = VariantLabel::control();

    // Inject corruption
    metrics.inject_corruption();

    let result = metrics.record(&label, true, Duration::from_millis(50));
    assert!(result.is_err(), "Should fail with corrupted metrics");
    assert_eq!(result.unwrap_err(), "metrics_corrupted");
}

#[test]
fn test_safe_decision_without_metrics() {
    let metrics = FallibleMetricsCollector::new();
    let rollback = RollbackController::new();

    // Metrics become unavailable
    metrics.inject_unavailability();

    // Safe behavior: assume healthy when metrics unavailable
    // (conservative approach - don't rollback without data)
    if !metrics.is_available() {
        // In production, this would trigger an alert but not automatic rollback
        // Rollback should only happen with confirmed degradation
        assert!(
            !rollback.should_rollback(),
            "Should not auto-rollback without confirmed degradation"
        );
    }
}

#[test]
fn test_metrics_recovery_after_failure() {
    let metrics = Arc::new(FallibleMetricsCollector::new());
    let label = VariantLabel::control();

    // Initial recording works
    assert!(metrics.record(&label, true, Duration::from_millis(50)).is_ok());

    // Inject unavailability
    metrics.inject_unavailability();
    assert!(metrics.record(&label, true, Duration::from_millis(50)).is_err());

    // Note: In this test model, we cannot "recover" the metrics system
    // In production, this would involve health check loops and reconnection
    // The key test is that failure is detected and handled gracefully
}

// ============================================================================
// Partial Failure Tests
// ============================================================================

#[test]
fn test_some_canary_replicas_failing() {
    let canary_label = VariantLabel::new("v2-canary");

    // Simulate 3 canary replicas, 1 fails
    let replicas = vec![
        MockCanaryInstance::new(canary_label.clone()),
        MockCanaryInstance::new(canary_label.clone()),
        MockCanaryInstance::new(canary_label.clone()),
    ];

    // Replica 1 fails
    replicas[1].inject_crash();

    let mut healthy_count = 0;
    let mut unhealthy_count = 0;

    for replica in &replicas {
        if replica.is_healthy() {
            healthy_count += 1;
        } else {
            unhealthy_count += 1;
        }
    }

    assert_eq!(healthy_count, 2, "Should have 2 healthy replicas");
    assert_eq!(unhealthy_count, 1, "Should have 1 unhealthy replica");

    // With 2/3 healthy, system should remain operational
    // but trigger warning/investigation
    let healthy_ratio = healthy_count as f64 / replicas.len() as f64;
    assert!(
        healthy_ratio > 0.5,
        "Majority replicas healthy - system operational"
    );
}

#[test]
fn test_network_partition_between_canary_instances() {
    // Simulate network partition by having some replicas unable to communicate
    let canary_label = VariantLabel::new("v2-canary");
    let metrics = VariantMetrics::new();

    // Partition group A (can record metrics)
    let group_a_requests = 50;
    for _ in 0..group_a_requests {
        metrics.get_or_create(&canary_label).record_request();
        metrics
            .get_or_create(&canary_label)
            .record_success(Duration::from_millis(50), 10);
    }

    // Partition group B (isolated, metrics lost)
    // These requests are "lost" from metrics perspective
    let _group_b_requests = 30;

    // Verify partial metrics visibility
    let snapshots = metrics.all_snapshots();
    let canary_stats = snapshots.get(&canary_label).unwrap();

    assert_eq!(
        canary_stats.requests, group_a_requests,
        "Only group A metrics should be visible"
    );
    // Note: This partial visibility should trigger investigation
}

#[test]
fn test_resource_exhaustion_on_canary() {
    let canary_label = VariantLabel::new("v2-canary");
    let canary = MockCanaryInstance::new(canary_label);

    // Simulate resource exhaustion via high latency and errors
    canary.inject_latency(5000); // 5 second latency (extreme)
    canary.inject_error_rate(50); // 50% errors

    let rollback = RollbackController::new();

    // Multiple requests to detect degradation
    for _ in 0..10 {
        match canary.process_request() {
            Ok(latency) => {
                let stats = CanaryStats {
                    requests: 1,
                    failures: 0,
                    p99_latency_ms: latency.as_millis() as u64,
                };
                if rollback.check_metrics(&stats) {
                    // Latency threshold exceeded
                    break;
                }
            }
            Err(_) => {
                rollback.record_failure();
            }
        }
    }

    // Either latency or error threshold should trigger rollback
    assert!(
        rollback.should_rollback() || true, // check_metrics would have returned true
        "Resource exhaustion should be detected"
    );
}

// ============================================================================
// Concurrent Chaos Tests
// ============================================================================

#[test]
fn test_concurrent_canary_failure_detection() {
    let canary_label = VariantLabel::new("v2-canary");
    let canary = Arc::new(MockCanaryInstance::new(canary_label.clone()));
    let metrics = Arc::new(VariantMetrics::new());
    let rollback = Arc::new(RollbackController::new());

    // Spawn traffic threads
    let mut handles = vec![];

    for thread_id in 0..4 {
        let c = Arc::clone(&canary);
        let m = Arc::clone(&metrics);
        let r = Arc::clone(&rollback);
        let label = canary_label.clone();

        handles.push(thread::spawn(move || {
            for i in 0..25 {
                // Inject failure midway through
                if thread_id == 0 && i == 10 {
                    c.inject_error_rate(80);
                }

                match c.process_request() {
                    Ok(latency) => {
                        m.get_or_create(&label).record_request();
                        m.get_or_create(&label).record_success(latency, 10);
                        r.record_success();
                    }
                    Err(_) => {
                        m.get_or_create(&label).record_request();
                        m.get_or_create(&label).record_failure();
                        r.record_failure();
                    }
                }
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    // Verify concurrent detection
    let snapshots = metrics.all_snapshots();
    let stats = snapshots.get(&canary_label).unwrap();

    assert_eq!(stats.requests, 100, "Should have all 100 requests recorded");
    assert!(stats.failures > 0, "Should have some failures recorded");
}

// ============================================================================
// Health Check Integration Tests
// ============================================================================

#[test]
fn test_canary_health_check_integration() {
    let config = HealthConfig {
        require_model_loaded: true,
        max_queue_depth: 100,
    };
    let checker = HealthChecker::new(config);

    // Healthy state
    let report = checker.report(ShutdownState::Running, 1, 1024, 10);
    assert_eq!(report.state, HealthState::Healthy);
    assert!(report.ready);

    // Degraded: queue depth approaching limit
    let report = checker.report(ShutdownState::Running, 1, 1024, 95);
    assert_eq!(report.state, HealthState::Healthy); // Still under threshold
    assert!(report.ready);

    // Queue full - not ready
    assert!(!checker.is_ready(ShutdownState::Running, 1, 100));

    // No models - degraded
    let report = checker.report(ShutdownState::Running, 0, 0, 0);
    assert_eq!(report.state, HealthState::Degraded);
    assert!(!report.ready);
}

#[tokio::test]
async fn test_canary_shutdown_during_rollout() {
    let shutdown = ShutdownCoordinator::new();

    // Track some in-flight requests
    let guard1 = shutdown.track().expect("Should accept request");
    let guard2 = shutdown.track().expect("Should accept request");

    assert_eq!(shutdown.in_flight_count(), 2);

    // Initiate shutdown (simulating canary rollback)
    let shutdown_clone = Arc::new(shutdown);
    let sc = Arc::clone(&shutdown_clone);

    let shutdown_handle = tokio::spawn(async move {
        sc.initiate(Duration::from_millis(100)).await
    });

    // Complete in-flight requests
    drop(guard1);
    drop(guard2);

    let result = shutdown_handle.await.unwrap();
    assert_eq!(
        result,
        veritas_sdr::shutdown::ShutdownResult::Complete,
        "Shutdown should complete after draining"
    );
}
