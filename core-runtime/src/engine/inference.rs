//! Core inference execution with real model delegation.

use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

use crate::engine::gguf::GgufModel;
use crate::engine::{InferenceConfig, InferenceInput, InferenceOutput};
use crate::models::ModelHandle;

#[derive(Error, Debug)]
pub enum InferenceError {
    #[error("Model not loaded: {0}")]
    ModelNotLoaded(String),

    #[error("Invalid parameters: {0}")]
    InvalidParams(String),

    #[error("Inference failed: {0}")]
    ExecutionFailed(String),

    #[error("Context length exceeded: max {max}, got {got}")]
    ContextExceeded { max: usize, got: usize },
}

/// Parameters controlling inference behavior (IPC protocol).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InferenceParams {
    pub max_tokens: usize,
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: usize,
    /// Enable token-by-token streaming response.
    #[serde(default)]
    pub stream: bool,
    /// Request timeout in milliseconds. None = no timeout.
    #[serde(default)]
    pub timeout_ms: Option<u64>,
}

impl Default for InferenceParams {
    fn default() -> Self {
        Self {
            max_tokens: 256,
            temperature: 0.7,
            top_p: 0.9,
            top_k: 40,
            stream: false,
            timeout_ms: None,
        }
    }
}

impl InferenceParams {
    pub fn validate(&self) -> Result<(), InferenceError> {
        if self.max_tokens == 0 {
            return Err(InferenceError::InvalidParams("max_tokens must be > 0".into()));
        }
        if self.temperature < 0.0 {
            return Err(InferenceError::InvalidParams("temperature must be >= 0".into()));
        }
        if self.top_p <= 0.0 || self.top_p > 1.0 {
            return Err(InferenceError::InvalidParams("top_p must be in (0, 1]".into()));
        }
        Ok(())
    }

    /// Convert to internal InferenceConfig format.
    pub fn to_config(&self) -> InferenceConfig {
        InferenceConfig {
            max_tokens: Some(self.max_tokens as u32),
            temperature: self.temperature,
            top_p: self.top_p,
            top_k: self.top_k as u32,
            repetition_penalty: 1.1,
            timeout_ms: self.timeout_ms.unwrap_or(30_000),
            max_memory_bytes: None,
        }
    }
}

/// Result of inference execution.
#[derive(Debug, Clone)]
pub struct InferenceResult {
    /// Generated text output.
    pub output: String,
    pub tokens_generated: usize,
    pub finished: bool,
}

/// Executes model inference by delegating to registered models.
pub struct InferenceEngine {
    max_context_length: usize,
    /// Models indexed by model_id for lookup.
    models: Arc<RwLock<HashMap<String, Arc<dyn GgufModel>>>>,
    /// ModelHandle to model_id mapping.
    handle_to_id: Arc<RwLock<HashMap<u64, String>>>,
}

impl InferenceEngine {
    pub fn new(max_context_length: usize) -> Self {
        Self {
            max_context_length,
            models: Arc::new(RwLock::new(HashMap::new())),
            handle_to_id: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a model for inference.
    pub async fn register_model(
        &self,
        model_id: String,
        handle: ModelHandle,
        model: Arc<dyn GgufModel>,
    ) {
        self.models.write().await.insert(model_id.clone(), model);
        self.handle_to_id.write().await.insert(handle.id(), model_id);
    }

    /// Unregister a model.
    pub async fn unregister_model(&self, model_id: &str) {
        self.models.write().await.remove(model_id);
        let mut handles = self.handle_to_id.write().await;
        handles.retain(|_, v| v != model_id);
    }

    /// Run inference on text prompt using the specified model.
    pub async fn run(
        &self,
        model_id: &str,
        prompt: &str,
        params: &InferenceParams,
    ) -> Result<InferenceResult, InferenceError> {
        params.validate()?;

        // Look up model by ID
        let models = self.models.read().await;
        let model = models.get(model_id).ok_or_else(|| {
            InferenceError::ModelNotLoaded(model_id.to_string())
        })?;

        // Check context length (approximate by bytes)
        if prompt.len() > self.max_context_length {
            return Err(InferenceError::ContextExceeded {
                max: self.max_context_length,
                got: prompt.len(),
            });
        }

        // Convert params to internal config
        let config = params.to_config();
        let input = InferenceInput::Text(prompt.to_string());

        // Delegate to actual model
        let output = model.infer(&input, &config).await.map_err(|e| {
            InferenceError::ExecutionFailed(e.to_string())
        })?;

        // Extract generation result
        match output {
            InferenceOutput::Generation(gen) => Ok(InferenceResult {
                output: gen.text,
                tokens_generated: gen.tokens_generated as usize,
                finished: true,
            }),
            _ => Err(InferenceError::ExecutionFailed(
                "Model returned non-generation output".into(),
            )),
        }
    }

    /// Run inference by handle (legacy API compatibility).
    pub async fn run_by_handle(
        &self,
        handle: ModelHandle,
        prompt: &str,
        params: &InferenceParams,
    ) -> Result<InferenceResult, InferenceError> {
        let handles = self.handle_to_id.read().await;
        let model_id = handles.get(&handle.id()).ok_or_else(|| {
            InferenceError::ModelNotLoaded(format!("handle {}", handle.id()))
        })?;
        self.run(model_id, prompt, params).await
    }

    pub fn max_context_length(&self) -> usize {
        self.max_context_length
    }

    /// Check if a model is registered.
    pub async fn has_model(&self, model_id: &str) -> bool {
        self.models.read().await.contains_key(model_id)
    }
}
