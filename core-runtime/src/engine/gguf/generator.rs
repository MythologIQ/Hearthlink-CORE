//! GGUF-based text generation model.
//!
//! Wraps llama-cpp-rs for text generation tasks.

use std::sync::atomic::{AtomicUsize, Ordering};

use crate::engine::{
    FinishReason, GenerationResult, InferenceCapability, InferenceConfig,
    InferenceError, InferenceInput, InferenceOutput,
};

/// GGUF text generation model using llama-cpp-rs.
pub struct GgufGenerator {
    model_id: String,
    memory_bytes: AtomicUsize,
    /// Context window size (used when llama-cpp backend is enabled)
    #[allow(dead_code)]
    context_size: u32,
    #[cfg(feature = "gguf")]
    _model: Option<()>, // Placeholder for llama-cpp model
}

impl GgufGenerator {
    /// Create a new generator with the given model ID.
    pub fn new(model_id: String, context_size: u32) -> Self {
        Self {
            model_id,
            memory_bytes: AtomicUsize::new(0),
            context_size,
            #[cfg(feature = "gguf")]
            _model: None,
        }
    }

    /// Generate text from a prompt.
    fn generate_text(
        &self,
        prompt: &str,
        max_tokens: u32,
    ) -> Result<GenerationResult, InferenceError> {
        if prompt.is_empty() {
            return Err(InferenceError::InputValidation("prompt cannot be empty".into()));
        }

        // Stub: Return mock generation result
        // Real implementation would run llama-cpp inference
        let generated = format!("[Generated from: {}...]", &prompt[..prompt.len().min(20)]);

        Ok(GenerationResult {
            text: generated,
            tokens_generated: max_tokens.min(10),
            finish_reason: FinishReason::MaxTokens,
        })
    }

    /// Format chat messages into a prompt string.
    fn format_chat_prompt(
        &self,
        messages: &[crate::engine::ChatMessage],
    ) -> Result<String, InferenceError> {
        let mut prompt = String::new();
        for msg in messages {
            let role_tag = match msg.role {
                crate::engine::ChatRole::System => "<|system|>",
                crate::engine::ChatRole::User => "<|user|>",
                crate::engine::ChatRole::Assistant => "<|assistant|>",
            };
            prompt.push_str(role_tag);
            prompt.push_str(&msg.content);
            prompt.push_str("<|end|>\n");
        }
        prompt.push_str("<|assistant|>");
        Ok(prompt)
    }
}

#[async_trait::async_trait]
impl super::GgufModel for GgufGenerator {
    fn model_id(&self) -> &str {
        &self.model_id
    }

    fn capabilities(&self) -> &[InferenceCapability] {
        &[InferenceCapability::TextGeneration]
    }

    fn memory_usage(&self) -> usize {
        self.memory_bytes.load(Ordering::SeqCst)
    }

    async fn infer(
        &self,
        input: &InferenceInput,
        config: &InferenceConfig,
    ) -> Result<InferenceOutput, InferenceError> {
        input.validate()?;
        config.validate()?;

        let max_tokens = config.max_tokens.unwrap_or(256);

        match input {
            InferenceInput::Text(prompt) => {
                let result = self.generate_text(prompt, max_tokens)?;
                Ok(InferenceOutput::Generation(result))
            }
            InferenceInput::ChatMessages(messages) => {
                let prompt = self.format_chat_prompt(messages)?;
                let result = self.generate_text(&prompt, max_tokens)?;
                Ok(InferenceOutput::Generation(result))
            }
            InferenceInput::TextBatch(_) => Err(InferenceError::CapabilityNotSupported(
                "batch generation not supported".into(),
            )),
        }
    }

    async fn unload(&mut self) -> Result<(), InferenceError> {
        self.memory_bytes.store(0, Ordering::SeqCst);
        #[cfg(feature = "gguf")]
        {
            self._model = None;
        }
        Ok(())
    }
}
