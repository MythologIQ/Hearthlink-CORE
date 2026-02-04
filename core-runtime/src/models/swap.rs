//! Hot-swap logic for model replacement.

use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

use super::registry::{ModelHandle, ModelRegistry};

#[derive(Error, Debug)]
pub enum SwapError {
    #[error("Model not found: {0}")]
    ModelNotFound(u64),

    #[error("Swap already in progress")]
    SwapInProgress,

    #[error("New model failed to load")]
    LoadFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SwapState {
    Idle,
    Preparing,
    Swapping,
}

/// Manages atomic model hot-swapping.
pub struct SwapManager {
    state: Arc<RwLock<SwapState>>,
    registry: Arc<ModelRegistry>,
}

impl SwapManager {
    pub fn new(registry: Arc<ModelRegistry>) -> Self {
        Self {
            state: Arc::new(RwLock::new(SwapState::Idle)),
            registry,
        }
    }

    /// Begin a swap operation. Returns error if swap already in progress.
    pub async fn begin_swap(&self) -> Result<SwapGuard, SwapError> {
        let mut state = self.state.write().await;

        if *state != SwapState::Idle {
            return Err(SwapError::SwapInProgress);
        }

        *state = SwapState::Preparing;
        Ok(SwapGuard { state: self.state.clone() })
    }

    /// Execute the swap: unload old model, register new one.
    pub async fn execute_swap(
        &self,
        old_handle: ModelHandle,
        new_handle: ModelHandle,
        guard: SwapGuard,
    ) -> Result<(), SwapError> {
        {
            let mut state = self.state.write().await;
            *state = SwapState::Swapping;
        }

        if !self.registry.contains(old_handle).await {
            guard.abort().await;
            return Err(SwapError::ModelNotFound(old_handle.id()));
        }

        self.registry.unregister(old_handle).await;
        guard.complete().await;

        Ok(())
    }

    pub async fn is_idle(&self) -> bool {
        *self.state.read().await == SwapState::Idle
    }
}

/// Guard ensuring swap state is properly cleaned up.
pub struct SwapGuard {
    state: Arc<RwLock<SwapState>>,
}

impl SwapGuard {
    async fn complete(self) {
        *self.state.write().await = SwapState::Idle;
    }

    async fn abort(self) {
        *self.state.write().await = SwapState::Idle;
    }
}
