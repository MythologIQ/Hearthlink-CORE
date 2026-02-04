//! Wire format and schema validation for IPC messages.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::engine::InferenceParams;

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("Invalid message format: {0}")]
    InvalidFormat(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Message too large: {size} bytes (max {max})")]
    MessageTooLarge { size: usize, max: usize },
}

/// Unique request identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RequestId(pub u64);

/// Inference request from caller.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceRequest {
    pub request_id: RequestId,
    pub model_id: String,
    pub prompt_tokens: Vec<u32>,
    pub parameters: InferenceParams,
}

impl InferenceRequest {
    pub fn validate(&self) -> Result<(), ProtocolError> {
        if self.model_id.is_empty() {
            return Err(ProtocolError::MissingField("model_id".into()));
        }
        if self.prompt_tokens.is_empty() {
            return Err(ProtocolError::MissingField("prompt_tokens".into()));
        }
        Ok(())
    }
}

/// Inference response to caller.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResponse {
    pub request_id: RequestId,
    pub output_tokens: Vec<u32>,
    pub finished: bool,
    pub error: Option<String>,
}

impl InferenceResponse {
    pub fn success(request_id: RequestId, output_tokens: Vec<u32>, finished: bool) -> Self {
        Self { request_id, output_tokens, finished, error: None }
    }

    pub fn error(request_id: RequestId, error: String) -> Self {
        Self { request_id, output_tokens: Vec::new(), finished: true, error: Some(error) }
    }
}

/// All possible IPC message types.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum IpcMessage {
    #[serde(rename = "handshake")]
    Handshake { token: String },

    #[serde(rename = "handshake_ack")]
    HandshakeAck { session_id: String },

    #[serde(rename = "inference_request")]
    InferenceRequest(InferenceRequest),

    #[serde(rename = "inference_response")]
    InferenceResponse(InferenceResponse),

    #[serde(rename = "error")]
    Error { code: u32, message: String },
}

const MAX_MESSAGE_SIZE: usize = 16 * 1024 * 1024; // 16 MB

/// Encode message to JSON bytes.
pub fn encode_message(message: &IpcMessage) -> Result<Vec<u8>, ProtocolError> {
    let bytes = serde_json::to_vec(message)?;
    if bytes.len() > MAX_MESSAGE_SIZE {
        return Err(ProtocolError::MessageTooLarge {
            size: bytes.len(),
            max: MAX_MESSAGE_SIZE,
        });
    }
    Ok(bytes)
}

/// Decode message from JSON bytes.
pub fn decode_message(bytes: &[u8]) -> Result<IpcMessage, ProtocolError> {
    if bytes.len() > MAX_MESSAGE_SIZE {
        return Err(ProtocolError::MessageTooLarge {
            size: bytes.len(),
            max: MAX_MESSAGE_SIZE,
        });
    }
    Ok(serde_json::from_slice(bytes)?)
}
