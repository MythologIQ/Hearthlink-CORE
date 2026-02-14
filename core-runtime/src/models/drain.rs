//! In-flight request tracking for drain coordination.
//!
//! Tracks active requests per model handle to enable graceful draining
//! during model hot-swap operations.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::RwLock;

use super::registry::ModelHandle;

#[derive(Error, Debug)]
pub enum DrainError {
    #[error("Drain timed out waiting for in-flight requests")]
    Timeout,
}

/// Tracks in-flight requests per model handle for drain coordination.
pub struct FlightTracker {
    in_flight: Arc<RwLock<HashMap<ModelHandle, Arc<AtomicU32>>>>,
}

impl FlightTracker {
    pub fn new() -> Self {
        Self {
            in_flight: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Increment in-flight count for model (returns RAII guard).
    pub async fn track(&self, handle: ModelHandle) -> FlightGuard {
        let counter = {
            let mut map = self.in_flight.write().await;
            map.entry(handle)
                .or_insert_with(|| Arc::new(AtomicU32::new(0)))
                .clone()
        };

        counter.fetch_add(1, Ordering::SeqCst);

        FlightGuard { counter }
    }

    /// Get current in-flight count for a model.
    pub async fn in_flight_count(&self, handle: ModelHandle) -> u32 {
        let map = self.in_flight.read().await;
        map.get(&handle)
            .map(|c| c.load(Ordering::SeqCst))
            .unwrap_or(0)
    }

    /// Wait until in-flight count reaches zero (with timeout).
    pub async fn drain(&self, handle: ModelHandle, timeout: Duration) -> Result<(), DrainError> {
        let deadline = tokio::time::Instant::now() + timeout;
        let poll_interval = Duration::from_millis(10);

        loop {
            let count = self.in_flight_count(handle).await;
            if count == 0 {
                return Ok(());
            }

            if tokio::time::Instant::now() >= deadline {
                return Err(DrainError::Timeout);
            }

            tokio::time::sleep(poll_interval).await;
        }
    }

    /// Remove tracking entry for a model (cleanup after unload).
    pub async fn remove(&self, handle: ModelHandle) {
        self.in_flight.write().await.remove(&handle);
    }
}

impl Default for FlightTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// RAII guard that decrements in-flight count on drop.
pub struct FlightGuard {
    counter: Arc<AtomicU32>,
}

impl Drop for FlightGuard {
    fn drop(&mut self) {
        self.counter.fetch_sub(1, Ordering::SeqCst);
    }
}
