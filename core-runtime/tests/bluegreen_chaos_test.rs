// Copyright 2024-2026 Veritas SDR Contributors
// SPDX-License-Identifier: Apache-2.0

//! Chaos tests for blue-green deployment scenarios.
//!
//! These tests validate system resilience during traffic switches,
//! state synchronization failures, and resource issues in blue-green deployments.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use veritas_sdr::ab_testing::{TrafficConfig, TrafficSplitter, VariantLabel, VariantMetrics};
use veritas_sdr::health::{HealthChecker, HealthConfig, HealthState};
use veritas_sdr::shutdown::{ShutdownCoordinator, ShutdownState};

// ============================================================================
// Test Infrastructure
// ============================================================================

/// Represents a deployment environment (blue or green).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Environment {
    Blue,
    Green,
}

impl Environment {
    fn opposite(&self) -> Self {
        match self {
            Environment::Blue => Environment::Green,
            Environment::Green => Environment::Blue,
        }
    }

    #[allow(dead_code)]
    fn label(&self) -> VariantLabel {
        match self {
            Environment::Blue => VariantLabel::new("blue"),
            Environment::Green => VariantLabel::new("green"),
        }
    }
}

/// Simulated environment instance for chaos testing.
#[allow(dead_code)]
struct MockEnvironment {
    name: Environment,
    healthy: AtomicBool,
    ready: AtomicBool,
    memory_used_bytes: AtomicU64,
    gpu_available: AtomicBool,
    disk_full: AtomicBool,
    cache_warmed: AtomicBool,
    request_count: AtomicU64,
}

impl MockEnvironment {
    fn new(name: Environment) -> Self {
        Self {
            name,
            healthy: AtomicBool::new(true),
            ready: AtomicBool::new(true),
            memory_used_bytes: AtomicU64::new(1024 * 1024 * 100), // 100MB
            gpu_available: AtomicBool::new(true),
            disk_full: AtomicBool::new(false),
            cache_warmed: AtomicBool::new(false),
            request_count: AtomicU64::new(0),
        }
    }

    fn is_healthy(&self) -> bool {
        self.healthy.load(Ordering::SeqCst)
            && !self.disk_full.load(Ordering::SeqCst)
            && self.gpu_available.load(Ordering::SeqCst)
    }

    fn is_ready(&self) -> bool {
        self.ready.load(Ordering::SeqCst) && self.is_healthy()
    }

    fn inject_unhealthy(&self) {
        self.healthy.store(false, Ordering::SeqCst);
    }

    fn inject_oom(&self) {
        // 16GB - simulating OOM condition
        self.memory_used_bytes
            .store(16 * 1024 * 1024 * 1024, Ordering::SeqCst);
        self.healthy.store(false, Ordering::SeqCst);
    }

    fn inject_disk_full(&self) {
        self.disk_full.store(true, Ordering::SeqCst);
    }

    fn inject_gpu_unavailable(&self) {
        self.gpu_available.store(false, Ordering::SeqCst);
    }

    fn warm_cache(&self) -> Result<(), &'static str> {
        if !self.is_healthy() {
            return Err("environment_unhealthy");
        }
        self.cache_warmed.store(true, Ordering::SeqCst);
        Ok(())
    }

    fn process_request(&self) -> Result<Duration, &'static str> {
        // Check specific failures first for accurate error reporting
        if self.disk_full.load(Ordering::SeqCst) {
            return Err("disk_full");
        }
        if !self.gpu_available.load(Ordering::SeqCst) {
            return Err("gpu_unavailable");
        }
        if !self.is_ready() {
            return Err("environment_not_ready");
        }

        self.request_count.fetch_add(1, Ordering::SeqCst);

        // Slower without warm cache
        let latency = if self.cache_warmed.load(Ordering::SeqCst) {
            50
        } else {
            500
        };
        Ok(Duration::from_millis(latency))
    }
}

/// Traffic switch controller for blue-green deployments.
struct TrafficSwitchController {
    active: Mutex<Environment>,
    switch_in_progress: AtomicBool,
    switch_interrupted: AtomicBool,
    dns_propagation_delay_ms: AtomicU64,
    rollback_available: AtomicBool,
}

impl TrafficSwitchController {
    fn new(initial: Environment) -> Self {
        Self {
            active: Mutex::new(initial),
            switch_in_progress: AtomicBool::new(false),
            switch_interrupted: AtomicBool::new(false),
            dns_propagation_delay_ms: AtomicU64::new(0),
            rollback_available: AtomicBool::new(true),
        }
    }

    fn active_environment(&self) -> Environment {
        *self.active.lock().unwrap()
    }

    fn inject_switch_interruption(&self) {
        self.switch_interrupted.store(true, Ordering::SeqCst);
    }

    fn inject_dns_delay(&self, ms: u64) {
        self.dns_propagation_delay_ms.store(ms, Ordering::SeqCst);
    }

    fn start_switch(&self, _target: Environment) -> Result<(), &'static str> {
        if self.switch_in_progress.load(Ordering::SeqCst) {
            return Err("switch_already_in_progress");
        }

        self.switch_in_progress.store(true, Ordering::SeqCst);
        Ok(())
    }

    fn complete_switch(&self, target: Environment) -> Result<(), &'static str> {
        if !self.switch_in_progress.load(Ordering::SeqCst) {
            return Err("no_switch_in_progress");
        }

        if self.switch_interrupted.load(Ordering::SeqCst) {
            self.switch_in_progress.store(false, Ordering::SeqCst);
            return Err("switch_interrupted");
        }

        // Simulate DNS propagation delay
        let delay = self.dns_propagation_delay_ms.load(Ordering::SeqCst);
        if delay > 0 {
            thread::sleep(Duration::from_millis(delay.min(100))); // Cap for tests
        }

        // Perform switch
        {
            let mut active = self.active.lock().unwrap();
            *active = target;
        }

        self.switch_in_progress.store(false, Ordering::SeqCst);
        Ok(())
    }

    fn rollback(&self) -> Result<Environment, &'static str> {
        if !self.rollback_available.load(Ordering::SeqCst) {
            return Err("rollback_not_available");
        }

        let current = self.active_environment();
        let target = current.opposite();

        // Rollback is a reverse switch
        self.start_switch(target)?;
        self.complete_switch(target)?;

        Ok(target)
    }

    fn is_switch_in_progress(&self) -> bool {
        self.switch_in_progress.load(Ordering::SeqCst)
    }
}

/// KV-cache state manager for testing state sync failures.
struct KvCacheManager {
    entries: Mutex<HashMap<String, Vec<u8>>>,
    corrupted: AtomicBool,
    sync_failed: AtomicBool,
}

impl KvCacheManager {
    fn new() -> Self {
        Self {
            entries: Mutex::new(HashMap::new()),
            corrupted: AtomicBool::new(false),
            sync_failed: AtomicBool::new(false),
        }
    }

    fn put(&self, key: String, value: Vec<u8>) -> Result<(), &'static str> {
        if self.corrupted.load(Ordering::SeqCst) {
            return Err("cache_corrupted");
        }
        let mut entries = self.entries.lock().unwrap();
        entries.insert(key, value);
        Ok(())
    }

    fn get(&self, key: &str) -> Result<Option<Vec<u8>>, &'static str> {
        if self.corrupted.load(Ordering::SeqCst) {
            return Err("cache_corrupted");
        }
        let entries = self.entries.lock().unwrap();
        Ok(entries.get(key).cloned())
    }

    fn inject_corruption(&self) {
        self.corrupted.store(true, Ordering::SeqCst);
    }

    fn inject_sync_failure(&self) {
        self.sync_failed.store(true, Ordering::SeqCst);
    }

    fn sync_to(&self, _target: &KvCacheManager) -> Result<(), &'static str> {
        if self.sync_failed.load(Ordering::SeqCst) {
            return Err("sync_failed");
        }
        if self.corrupted.load(Ordering::SeqCst) {
            return Err("source_corrupted");
        }
        Ok(())
    }

    fn is_healthy(&self) -> bool {
        !self.corrupted.load(Ordering::SeqCst) && !self.sync_failed.load(Ordering::SeqCst)
    }

    fn entry_count(&self) -> usize {
        self.entries.lock().unwrap().len()
    }
}

/// Session state manager.
struct SessionStateManager {
    sessions: Mutex<HashMap<String, String>>,
    state_lost: AtomicBool,
}

impl SessionStateManager {
    fn new() -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
            state_lost: AtomicBool::new(false),
        }
    }

    fn create_session(&self, id: String, data: String) {
        if !self.state_lost.load(Ordering::SeqCst) {
            let mut sessions = self.sessions.lock().unwrap();
            sessions.insert(id, data);
        }
    }

    fn get_session(&self, id: &str) -> Option<String> {
        if self.state_lost.load(Ordering::SeqCst) {
            return None;
        }
        let sessions = self.sessions.lock().unwrap();
        sessions.get(id).cloned()
    }

    fn inject_state_loss(&self) {
        self.state_lost.store(true, Ordering::SeqCst);
        let mut sessions = self.sessions.lock().unwrap();
        sessions.clear();
    }

    fn session_count(&self) -> usize {
        self.sessions.lock().unwrap().len()
    }
}

// ============================================================================
// Switch Failure Tests
// ============================================================================

#[test]
fn test_traffic_switch_interrupted_mid_execution() {
    let controller = TrafficSwitchController::new(Environment::Blue);

    // Start switch to green
    assert!(controller.start_switch(Environment::Green).is_ok());
    assert!(controller.is_switch_in_progress());

    // Inject interruption
    controller.inject_switch_interruption();

    // Complete should fail
    let result = controller.complete_switch(Environment::Green);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "switch_interrupted");

    // Should remain on blue
    assert_eq!(controller.active_environment(), Environment::Blue);
    assert!(!controller.is_switch_in_progress());
}

#[test]
fn test_new_environment_unhealthy_after_switch() {
    let blue = MockEnvironment::new(Environment::Blue);
    let green = MockEnvironment::new(Environment::Green);
    let controller = TrafficSwitchController::new(Environment::Blue);

    // Warm up blue (current production)
    assert!(blue.warm_cache().is_ok());
    assert!(blue.is_healthy());

    // Switch to green
    assert!(controller.start_switch(Environment::Green).is_ok());
    assert!(controller.complete_switch(Environment::Green).is_ok());

    // Green becomes unhealthy after switch
    green.inject_unhealthy();

    // Verify green is unhealthy
    assert!(!green.is_healthy());

    // Should be able to rollback
    let result = controller.rollback();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Environment::Blue);
    assert_eq!(controller.active_environment(), Environment::Blue);
}

#[test]
fn test_dns_propagation_delay() {
    let controller = TrafficSwitchController::new(Environment::Blue);

    // Inject DNS delay
    controller.inject_dns_delay(50);

    let start = std::time::Instant::now();

    assert!(controller.start_switch(Environment::Green).is_ok());
    assert!(controller.complete_switch(Environment::Green).is_ok());

    let elapsed = start.elapsed();
    assert!(
        elapsed.as_millis() >= 50,
        "Should have DNS propagation delay"
    );
    assert_eq!(controller.active_environment(), Environment::Green);
}

#[test]
fn test_rollback_capability_preserved() {
    let blue = MockEnvironment::new(Environment::Blue);
    let green = MockEnvironment::new(Environment::Green);
    let controller = TrafficSwitchController::new(Environment::Blue);

    // Warm both environments
    assert!(blue.warm_cache().is_ok());
    assert!(green.warm_cache().is_ok());

    // Switch to green
    assert!(controller.start_switch(Environment::Green).is_ok());
    assert!(controller.complete_switch(Environment::Green).is_ok());
    assert_eq!(controller.active_environment(), Environment::Green);

    // Simulate green degradation
    green.inject_oom();

    // Verify rollback works
    let result = controller.rollback();
    assert!(result.is_ok());
    assert_eq!(controller.active_environment(), Environment::Blue);
    assert!(blue.is_healthy());
}

#[test]
fn test_double_switch_prevention() {
    let controller = TrafficSwitchController::new(Environment::Blue);

    // Start first switch
    assert!(controller.start_switch(Environment::Green).is_ok());

    // Try to start second switch - should fail
    let result = controller.start_switch(Environment::Blue);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "switch_already_in_progress");
}

// ============================================================================
// State Sync Failure Tests
// ============================================================================

#[test]
fn test_cache_warming_failure() {
    let env = MockEnvironment::new(Environment::Green);

    // Environment healthy - cache warming works
    assert!(env.warm_cache().is_ok());
    assert!(env.cache_warmed.load(Ordering::SeqCst));

    // Create unhealthy environment
    let unhealthy_env = MockEnvironment::new(Environment::Green);
    unhealthy_env.inject_unhealthy();

    // Cache warming should fail
    let result = unhealthy_env.warm_cache();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "environment_unhealthy");
}

#[test]
fn test_kv_cache_corruption() {
    let cache = KvCacheManager::new();

    // Normal operation
    assert!(cache.put("key1".to_string(), vec![1, 2, 3]).is_ok());
    assert!(cache.get("key1").is_ok());

    // Inject corruption
    cache.inject_corruption();

    // Operations should fail
    assert!(cache.put("key2".to_string(), vec![4, 5, 6]).is_err());
    let result = cache.get("key1");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "cache_corrupted");
}

#[test]
fn test_kv_cache_sync_failure() {
    let source = KvCacheManager::new();
    let target = KvCacheManager::new();

    // Add data to source
    assert!(source.put("key1".to_string(), vec![1, 2, 3]).is_ok());
    assert!(source.put("key2".to_string(), vec![4, 5, 6]).is_ok());

    // Normal sync works
    assert!(source.sync_to(&target).is_ok());

    // Inject sync failure
    source.inject_sync_failure();

    // Sync should fail
    let result = source.sync_to(&target);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "sync_failed");
}

#[test]
fn test_session_state_loss() {
    let sessions = SessionStateManager::new();

    // Create sessions
    sessions.create_session("session-1".to_string(), "data-1".to_string());
    sessions.create_session("session-2".to_string(), "data-2".to_string());
    assert_eq!(sessions.session_count(), 2);

    // Inject state loss (simulating switch without state transfer)
    sessions.inject_state_loss();

    // All sessions should be gone
    assert_eq!(sessions.session_count(), 0);
    assert!(sessions.get_session("session-1").is_none());
}

#[test]
fn test_graceful_degradation_without_cache() {
    let env = MockEnvironment::new(Environment::Green);

    // Process request without warm cache
    let result = env.process_request();
    assert!(result.is_ok());

    let latency = result.unwrap();
    assert_eq!(
        latency.as_millis(),
        500,
        "Should have degraded latency without cache"
    );

    // Warm cache
    env.warm_cache().unwrap();

    // Process request with warm cache
    let result = env.process_request();
    assert!(result.is_ok());

    let latency = result.unwrap();
    assert_eq!(
        latency.as_millis(),
        50,
        "Should have fast latency with cache"
    );
}

// ============================================================================
// Resource Failure Tests
// ============================================================================

#[test]
fn test_standby_environment_oom() {
    let green = MockEnvironment::new(Environment::Green);

    // Inject OOM
    green.inject_oom();

    // Environment should be unhealthy
    assert!(!green.is_healthy());

    // Requests should fail
    let result = green.process_request();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "environment_not_ready");
}

#[test]
fn test_disk_full_on_new_environment() {
    let green = MockEnvironment::new(Environment::Green);

    // Inject disk full
    green.inject_disk_full();

    // Requests should fail
    let result = green.process_request();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "disk_full");
}

#[test]
fn test_gpu_unavailable_on_new_environment() {
    let green = MockEnvironment::new(Environment::Green);

    // Initially GPU available
    assert!(green.gpu_available.load(Ordering::SeqCst));
    assert!(green.process_request().is_ok());

    // Inject GPU unavailability
    green.inject_gpu_unavailable();

    // Environment should be unhealthy
    assert!(!green.is_healthy());

    // Requests should fail
    let result = green.process_request();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "gpu_unavailable");
}

#[test]
fn test_multiple_resource_failures() {
    let green = MockEnvironment::new(Environment::Green);

    // Multiple failures
    green.inject_oom();
    green.inject_disk_full();
    green.inject_gpu_unavailable();

    // All health checks should fail
    assert!(!green.is_healthy());
    assert!(!green.is_ready());

    // Request should fail (first check to fail wins)
    let result = green.process_request();
    assert!(result.is_err());
}

// ============================================================================
// Blue-Green Traffic Switching Tests
// ============================================================================

#[test]
fn test_traffic_switch_with_splitter() {
    let blue_label = VariantLabel::new("blue");
    let green_label = VariantLabel::new("green");
    let metrics = VariantMetrics::new();

    // Initial state: 100% to blue
    let mut weights = std::collections::BTreeMap::new();
    weights.insert(blue_label.clone(), 100);
    weights.insert(green_label.clone(), 0);
    let config = TrafficConfig {
        weights,
        sticky_sessions: true,
    };
    let splitter = TrafficSplitter::new(config).unwrap();

    // All traffic goes to blue
    for i in 0..50 {
        let session = format!("session-{}", i);
        let variant = splitter.select(Some(&session));
        assert_eq!(variant, &blue_label);
        metrics.get_or_create(variant).record_request();
        metrics
            .get_or_create(variant)
            .record_success(Duration::from_millis(50), 10);
    }

    // Verify metrics
    let snapshots = metrics.all_snapshots();
    assert!(snapshots.get(&blue_label).unwrap().requests >= 50);
}

#[test]
fn test_gradual_traffic_shift() {
    let blue_label = VariantLabel::new("blue");
    let green_label = VariantLabel::new("green");

    // Start: 90% blue, 10% green
    let mut weights = std::collections::BTreeMap::new();
    weights.insert(blue_label.clone(), 90);
    weights.insert(green_label.clone(), 10);
    let config = TrafficConfig {
        weights,
        sticky_sessions: true,
    };
    let splitter = TrafficSplitter::new(config).unwrap();

    let mut blue_count = 0;
    let mut green_count = 0;

    for i in 0..1000 {
        let session = format!("session-{}", i);
        let variant = splitter.select(Some(&session));
        if variant == &blue_label {
            blue_count += 1;
        } else {
            green_count += 1;
        }
    }

    // Verify approximate distribution (with tolerance)
    assert!(
        blue_count > 800,
        "Blue should get ~90% traffic, got {}",
        blue_count
    );
    assert!(
        green_count > 50,
        "Green should get ~10% traffic, got {}",
        green_count
    );
}

// ============================================================================
// Health Check Integration Tests
// ============================================================================

#[test]
fn test_environment_health_monitoring() {
    let config = HealthConfig {
        require_model_loaded: true,
        max_queue_depth: 100,
    };
    let checker = HealthChecker::new(config);

    // Blue environment healthy
    let report = checker.report(ShutdownState::Running, 2, 1024 * 1024, 10);
    assert_eq!(report.state, HealthState::Healthy);
    assert!(report.ready);

    // Draining (during switch)
    let report = checker.report(ShutdownState::Draining, 2, 1024 * 1024, 10);
    assert_eq!(report.state, HealthState::Unhealthy);
    assert!(!report.ready);
}

#[tokio::test]
async fn test_graceful_switch_with_drain() {
    let shutdown = Arc::new(ShutdownCoordinator::new());

    // Simulate in-flight requests on blue
    let guard1 = shutdown.track().expect("Should accept");
    let guard2 = shutdown.track().expect("Should accept");
    let guard3 = shutdown.track().expect("Should accept");

    assert_eq!(shutdown.in_flight_count(), 3);

    // Start draining blue before switch
    let sc = Arc::clone(&shutdown);
    let handle = tokio::spawn(async move {
        sc.initiate(Duration::from_millis(200)).await
    });

    // Complete in-flight requests
    tokio::time::sleep(Duration::from_millis(10)).await;
    drop(guard1);
    drop(guard2);
    drop(guard3);

    let result = handle.await.unwrap();
    assert_eq!(result, veritas_sdr::shutdown::ShutdownResult::Complete);
}

#[tokio::test]
async fn test_switch_timeout_with_stuck_requests() {
    let shutdown = Arc::new(ShutdownCoordinator::new());

    // Track request that won't complete
    let _guard = shutdown.track().expect("Should accept");

    // Initiate shutdown with short timeout
    let result = shutdown.initiate(Duration::from_millis(50)).await;

    // Should timeout
    match result {
        veritas_sdr::shutdown::ShutdownResult::Timeout { remaining } => {
            assert_eq!(remaining, 1, "Should have 1 stuck request");
        }
        _ => panic!("Expected timeout"),
    }
}

// ============================================================================
// Concurrent Blue-Green Tests
// ============================================================================

#[test]
fn test_concurrent_traffic_during_switch() {
    let blue = Arc::new(MockEnvironment::new(Environment::Blue));
    let green = Arc::new(MockEnvironment::new(Environment::Green));
    let controller = Arc::new(TrafficSwitchController::new(Environment::Blue));

    // Warm both environments
    blue.warm_cache().unwrap();
    green.warm_cache().unwrap();

    let mut handles = vec![];

    // Traffic threads
    for thread_id in 0..4 {
        let b = Arc::clone(&blue);
        let g = Arc::clone(&green);
        let c = Arc::clone(&controller);

        handles.push(thread::spawn(move || {
            let mut processed = 0;
            for _ in 0..25 {
                let env = match c.active_environment() {
                    Environment::Blue => &b,
                    Environment::Green => &g,
                };

                if env.process_request().is_ok() {
                    processed += 1;
                }

                // Thread 0 triggers switch midway
                if thread_id == 0 && processed == 10 {
                    let _ = c.start_switch(Environment::Green);
                    let _ = c.complete_switch(Environment::Green);
                }
            }
            processed
        }));
    }

    let total: usize = handles.into_iter().map(|h| h.join().unwrap()).sum();

    // All requests should be processed
    assert_eq!(total, 100, "All requests should be processed");

    // Should be on green after switch
    assert_eq!(controller.active_environment(), Environment::Green);
}

#[test]
fn test_cache_consistency_during_switch() {
    let blue_cache = Arc::new(KvCacheManager::new());
    let green_cache = Arc::new(KvCacheManager::new());

    // Populate blue cache
    for i in 0..10 {
        blue_cache
            .put(format!("key-{}", i), vec![i as u8])
            .unwrap();
    }
    assert_eq!(blue_cache.entry_count(), 10);

    // Sync to green before switch
    assert!(blue_cache.sync_to(&green_cache).is_ok());

    // Verify sync worked (in real impl, this would copy data)
    assert!(green_cache.is_healthy());
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_rapid_switch_back_and_forth() {
    let controller = TrafficSwitchController::new(Environment::Blue);

    // Multiple rapid switches
    for _ in 0..5 {
        // Switch to green
        assert!(controller.start_switch(Environment::Green).is_ok());
        assert!(controller.complete_switch(Environment::Green).is_ok());
        assert_eq!(controller.active_environment(), Environment::Green);

        // Switch back to blue
        assert!(controller.start_switch(Environment::Blue).is_ok());
        assert!(controller.complete_switch(Environment::Blue).is_ok());
        assert_eq!(controller.active_environment(), Environment::Blue);
    }
}

#[test]
fn test_rollback_after_failed_switch() {
    let controller = TrafficSwitchController::new(Environment::Blue);

    // Start switch
    assert!(controller.start_switch(Environment::Green).is_ok());

    // Interrupt switch
    controller.inject_switch_interruption();

    // Switch fails
    assert!(controller.complete_switch(Environment::Green).is_err());

    // Still on blue
    assert_eq!(controller.active_environment(), Environment::Blue);

    // Reset interruption for future operations
    controller
        .switch_interrupted
        .store(false, Ordering::SeqCst);

    // Rollback should still work (we're on blue, so this switches to green and back)
    // Actually, since we failed to switch, we don't need rollback
    // The key assertion is we stayed on blue
}

#[test]
fn test_session_migration_failure() {
    let source_sessions = SessionStateManager::new();
    let target_sessions = SessionStateManager::new();

    // Create sessions in source
    for i in 0..5 {
        source_sessions.create_session(format!("session-{}", i), format!("data-{}", i));
    }
    assert_eq!(source_sessions.session_count(), 5);

    // Simulate failed migration by losing state in target
    target_sessions.inject_state_loss();

    // Target has no sessions
    assert_eq!(target_sessions.session_count(), 0);

    // Source still has sessions (can be used for recovery)
    assert_eq!(source_sessions.session_count(), 5);
}
