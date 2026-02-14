//! Atomic routing table for model_id → ModelHandle resolution.
//!
//! Provides thread-safe routing operations for zero-downtime model swaps.

use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

use super::registry::ModelHandle;

#[derive(Error, Debug)]
pub enum RouterError {
    #[error("Route already exists for model: {0}")]
    RouteExists(String),
}

/// Atomic routing table: model_id → ModelHandle.
///
/// Thread-safe routing operations for resolving model IDs to handles.
/// Supports atomic swap for zero-downtime model replacement.
pub struct ModelRouter {
    routes: Arc<RwLock<HashMap<String, ModelHandle>>>,
}

impl ModelRouter {
    pub fn new() -> Self {
        Self {
            routes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Resolve model_id to handle (returns None if not routed).
    pub async fn resolve(&self, model_id: &str) -> Option<ModelHandle> {
        self.routes.read().await.get(model_id).copied()
    }

    /// Add new route (fails if already exists).
    pub async fn add_route(&self, model_id: &str, handle: ModelHandle) -> Result<(), RouterError> {
        let mut routes = self.routes.write().await;

        if routes.contains_key(model_id) {
            return Err(RouterError::RouteExists(model_id.to_string()));
        }

        routes.insert(model_id.to_string(), handle);
        Ok(())
    }

    /// Atomically swap route to new handle.
    /// Returns the old handle if route existed, None if new route created.
    pub async fn swap_route(&self, model_id: &str, new_handle: ModelHandle) -> Option<ModelHandle> {
        let mut routes = self.routes.write().await;
        routes.insert(model_id.to_string(), new_handle)
    }

    /// Remove route (returns old handle if existed).
    pub async fn remove_route(&self, model_id: &str) -> Option<ModelHandle> {
        self.routes.write().await.remove(model_id)
    }

    /// List all active routes.
    pub async fn list_routes(&self) -> Vec<(String, ModelHandle)> {
        self.routes
            .read()
            .await
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect()
    }

    /// Check if a route exists for the given model_id.
    pub async fn has_route(&self, model_id: &str) -> bool {
        self.routes.read().await.contains_key(model_id)
    }

    /// Number of active routes.
    pub async fn route_count(&self) -> usize {
        self.routes.read().await.len()
    }
}

impl Default for ModelRouter {
    fn default() -> Self {
        Self::new()
    }
}
