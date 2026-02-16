//! Inference engine module for CORE Runtime.
//!
//! Handles tokenization, inference execution, and token streaming.
//! Provides the `InferenceModel` trait and supporting types.

pub mod config;
pub mod decode;
pub mod error;
pub mod filter;
pub mod flash_attn;
pub mod gguf;
pub mod gpu;
pub mod input;
pub mod onnx;
pub mod output;
pub mod prefill;
pub mod quantize;
pub mod simd_matmul;
mod simd_neon;
pub mod simd_tokenizer;
pub mod simd_tokenizer_v2;
pub mod speculative;
pub mod speculative_v2;

mod inference;
mod streaming;
mod tokenizer;

pub use config::InferenceConfig;
pub use decode::{DecodeConfig, DecodeExecutor, DecodeStepResult};
pub use error::InferenceError;
pub use filter::{FilterConfig, OutputFilter};
pub use flash_attn::{FlashAttn, FlashAttnConfig};
pub use inference::{InferenceEngine, InferenceParams, InferenceResult};
pub use input::{ChatMessage, ChatRole, InferenceInput};
pub use input::{MAX_BATCH_SIZE, MAX_INPUT_TOKENS, MAX_TEXT_BYTES};
pub use output::{ClassificationResult, EmbeddingResult, EntityResult};
pub use output::{FinishReason, GenerationResult, InferenceOutput};
pub use prefill::{PrefillConfig, PrefillExecutor, PrefillResult};
pub use quantize::{QuantFormat, QuantizedTensor, QUANT_BLOCK_SIZE};
pub use simd_matmul::{dot_q4, dot_q8, init_simd};
pub use simd_tokenizer::SimdTokenizer;
pub use simd_tokenizer_v2::{
    SimdTokenizer as SimdTokenizerV2, TokenizerError as TokenizerV2Error, TokenizerStats,
};
pub use speculative::{
    DraftModel, SpeculativeConfig, SpeculativeDecoder, TargetModel, VerifyResult,
};
pub use speculative_v2::{
    SpeculativeConfig as SpeculativeV2Config, SpeculativeDecoder as SpeculativeV2Decoder,
    SpeculativeStats,
};
pub use streaming::{StreamingOutput, TokenStream};
pub use tokenizer::{TokenizerError, TokenizerWrapper};

// Backend re-exports
pub use gguf::{GgufConfig, GgufGenerator, GgufModel};
pub use gpu::{GpuBackend, GpuConfig, GpuDevice, GpuError, GpuManager, GpuMemory, GpuMemoryPool};
pub use onnx::{OnnxClassifier, OnnxConfig, OnnxEmbedder, OnnxModel};

/// What a model can do — used by the InferenceModel trait.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InferenceCapability {
    TextClassification,
    TextGeneration,
    Embedding,
    NamedEntityRecognition,
}
