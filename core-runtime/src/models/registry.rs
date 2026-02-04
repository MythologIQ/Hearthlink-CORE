//! Model registry for tracking loaded models.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::loader::ModelMetadata;

/// Unique handle to a loaded model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModelHandle(u64);

impl ModelHandle {
    pub fn id(&self) -> u64 {
        self.0
    }
}

struct LoadedModel {
    metadata: ModelMetadata,
    memory_bytes: usize,
}

/// Thread-safe registry of loaded models.
pub struct ModelRegistry {
    models: Arc<RwLock<HashMap<ModelHandle, LoadedModel>>>,
    next_id: AtomicU64,
}

impl ModelRegistry {
    pub fn new() -> Self {
        Self {
            models: Arc::new(RwLock::new(HashMap::new())),
            next_id: AtomicU64::new(1),
        }
    }

    /// Register a new model and return its handle.
    pub async fn register(&self, metadata: ModelMetadata, memory_bytes: usize) -> ModelHandle {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let handle = ModelHandle(id);

        let model = LoadedModel { metadata, memory_bytes };
        self.models.write().await.insert(handle, model);

        handle
    }

    /// Check if a model handle is valid.
    pub async fn contains(&self, handle: ModelHandle) -> bool {
        self.models.read().await.contains_key(&handle)
    }

    /// Get metadata for a loaded model.
    pub async fn get_metadata(&self, handle: ModelHandle) -> Option<ModelMetadata> {
        self.models.read().await.get(&handle).map(|m| m.metadata.clone())
    }

    /// Remove a model from the registry.
    pub async fn unregister(&self, handle: ModelHandle) -> Option<usize> {
        self.models.write().await.remove(&handle).map(|m| m.memory_bytes)
    }

    /// Total memory used by all registered models.
    pub async fn total_memory(&self) -> usize {
        self.models.read().await.values().map(|m| m.memory_bytes).sum()
    }

    /// Number of loaded models.
    pub async fn count(&self) -> usize {
        self.models.read().await.len()
    }
}

impl Default for ModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}
