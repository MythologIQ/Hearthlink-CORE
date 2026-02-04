//! Tokenization wrapper for model-agnostic token handling.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum TokenizerError {
    #[error("Tokenizer not loaded")]
    NotLoaded,

    #[error("Encoding failed: {0}")]
    EncodingFailed(String),

    #[error("Decoding failed: {0}")]
    DecodingFailed(String),

    #[error("Invalid token ID: {0}")]
    InvalidToken(u32),
}

/// Wrapper around model-specific tokenizer.
pub struct TokenizerWrapper {
    vocab_size: u32,
    eos_token: u32,
    bos_token: u32,
}

impl TokenizerWrapper {
    pub fn new(vocab_size: u32, eos_token: u32, bos_token: u32) -> Self {
        Self { vocab_size, eos_token, bos_token }
    }

    /// Encode text to token IDs (placeholder for actual tokenizer).
    pub fn encode(&self, _text: &str) -> Result<Vec<u32>, TokenizerError> {
        // In production, this would call the actual tokenizer
        // For now, return empty as the real tokenizer is loaded per-model
        Ok(Vec::new())
    }

    /// Decode token IDs to text (placeholder for actual tokenizer).
    pub fn decode(&self, tokens: &[u32]) -> Result<String, TokenizerError> {
        for &token in tokens {
            if token >= self.vocab_size {
                return Err(TokenizerError::InvalidToken(token));
            }
        }
        // In production, this would call the actual tokenizer
        Ok(String::new())
    }

    pub fn eos_token(&self) -> u32 {
        self.eos_token
    }

    pub fn bos_token(&self) -> u32 {
        self.bos_token
    }

    pub fn vocab_size(&self) -> u32 {
        self.vocab_size
    }

    /// Check if token is end-of-sequence.
    pub fn is_eos(&self, token: u32) -> bool {
        token == self.eos_token
    }
}
