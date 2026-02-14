//! Tests for FlightTracker - in-flight request tracking for drain coordination.

use core_runtime::models::{DrainError, FlightTracker, ModelHandle};
use std::time::Duration;

#[tokio::test]
async fn test_track_increments_count() {
    let tracker = FlightTracker::new();
    let handle = ModelHandle::new(1);

    assert_eq!(tracker.in_flight_count(handle).await, 0);

    let guard = tracker.track(handle).await;
    assert_eq!(tracker.in_flight_count(handle).await, 1);

    drop(guard);
    assert_eq!(tracker.in_flight_count(handle).await, 0);
}

#[tokio::test]
async fn test_drain_succeeds_when_zero() {
    let tracker = FlightTracker::new();
    let handle = ModelHandle::new(1);

    let result = tracker.drain(handle, Duration::from_millis(100)).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_drain_waits_for_completion() {
    use std::sync::Arc;

    let tracker = Arc::new(FlightTracker::new());
    let handle = ModelHandle::new(1);

    let guard = tracker.track(handle).await;
    let tracker_clone = tracker.clone();

    let drain_task = tokio::spawn(async move {
        tracker_clone.drain(handle, Duration::from_millis(500)).await
    });

    // Brief delay then release
    tokio::time::sleep(Duration::from_millis(50)).await;
    drop(guard);

    let result = drain_task.await.unwrap();
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_drain_timeout() {
    let tracker = FlightTracker::new();
    let handle = ModelHandle::new(1);

    let _guard = tracker.track(handle).await;

    let result = tracker.drain(handle, Duration::from_millis(50)).await;
    assert!(matches!(result, Err(DrainError::Timeout)));
}

#[tokio::test]
async fn test_multiple_concurrent_tracks() {
    let tracker = FlightTracker::new();
    let handle = ModelHandle::new(1);

    let mut guards = Vec::new();
    for _ in 0..5 {
        guards.push(tracker.track(handle).await);
    }

    assert_eq!(tracker.in_flight_count(handle).await, 5);

    drop(guards);
    assert_eq!(tracker.in_flight_count(handle).await, 0);
}

#[tokio::test]
async fn test_track_unknown_handle() {
    let tracker = FlightTracker::new();
    let handle = ModelHandle::new(999);

    // Tracking unknown handle should create entry
    let guard = tracker.track(handle).await;
    assert_eq!(tracker.in_flight_count(handle).await, 1);

    drop(guard);
    assert_eq!(tracker.in_flight_count(handle).await, 0);
}
