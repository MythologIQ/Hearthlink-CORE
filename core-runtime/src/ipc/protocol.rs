//! Wire format and schema validation for IPC messages.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::engine::InferenceParams;
use crate::health::HealthReport;
use crate::telemetry::MetricsSnapshot;

/// Protocol version for negotiating encoding strategies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProtocolVersion {
    /// V1: JSON encoding of token arrays (current default).
    V1,
    /// V2: Packed varint encoding (experimental).
    V2,
}

impl Default for ProtocolVersion {
    fn default() -> Self {
        Self::V1
    }
}

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

/// Single token chunk for streaming responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    pub request_id: RequestId,
    pub token: u32,
    pub is_final: bool,
    pub error: Option<String>,
}

impl StreamChunk {
    /// Create a non-final token chunk.
    pub fn token(request_id: RequestId, token: u32) -> Self {
        Self { request_id, token, is_final: false, error: None }
    }

    /// Create the final token chunk.
    pub fn final_token(request_id: RequestId, token: u32) -> Self {
        Self { request_id, token, is_final: true, error: None }
    }

    /// Create an error chunk (always final).
    pub fn error(request_id: RequestId, error: String) -> Self {
        Self { request_id, token: 0, is_final: true, error: Some(error) }
    }
}

/// Warmup request to prime a model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarmupRequest {
    pub model_id: String,
    /// Number of tokens to generate (default: 1).
    #[serde(default = "default_warmup_tokens")]
    pub tokens: usize,
}

fn default_warmup_tokens() -> usize {
    1
}

/// Warmup response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarmupResponse {
    pub model_id: String,
    pub success: bool,
    pub error: Option<String>,
    pub elapsed_ms: u64,
}

impl WarmupResponse {
    pub fn success(model_id: String, elapsed_ms: u64) -> Self {
        Self { model_id, success: true, error: None, elapsed_ms }
    }

    pub fn error(model_id: String, error: String, elapsed_ms: u64) -> Self {
        Self { model_id, success: false, error: Some(error), elapsed_ms }
    }
}

/// Health check request types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthCheckType {
    Liveness,
    Readiness,
    Full,
}

/// Health check response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    pub check_type: HealthCheckType,
    pub ok: bool,
    pub report: Option<HealthReport>,
}

/// All possible IPC message types.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum IpcMessage {
    #[serde(rename = "handshake")]
    Handshake {
        token: String,
        /// Optional protocol version request. Defaults to V1 if not specified.
        #[serde(default)]
        protocol_version: Option<ProtocolVersion>,
    },

    #[serde(rename = "handshake_ack")]
    HandshakeAck {
        session_id: String,
        /// Negotiated protocol version for this session.
        #[serde(default)]
        protocol_version: ProtocolVersion,
    },

    #[serde(rename = "inference_request")]
    InferenceRequest(InferenceRequest),

    #[serde(rename = "inference_response")]
    InferenceResponse(InferenceResponse),

    #[serde(rename = "stream_chunk")]
    StreamChunk(StreamChunk),

    #[serde(rename = "health_check")]
    HealthCheck { check_type: HealthCheckType },

    #[serde(rename = "health_response")]
    HealthResponse(HealthCheckResponse),

    #[serde(rename = "metrics_request")]
    MetricsRequest,

    #[serde(rename = "metrics_response")]
    MetricsResponse(MetricsSnapshot),

    #[serde(rename = "cancel_request")]
    CancelRequest { request_id: RequestId },

    #[serde(rename = "cancel_response")]
    CancelResponse { request_id: RequestId, cancelled: bool },

    #[serde(rename = "warmup_request")]
    WarmupRequest(WarmupRequest),

    #[serde(rename = "warmup_response")]
    WarmupResponse(WarmupResponse),

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

/// Encode message to binary (bincode) for high-performance IPC.
/// ~10-100x faster than JSON for same-machine communication.
pub fn encode_message_binary(message: &IpcMessage) -> Result<Vec<u8>, ProtocolError> {
    let bytes = bincode::serialize(message)
        .map_err(|e| ProtocolError::InvalidFormat(e.to_string()))?;
    if bytes.len() > MAX_MESSAGE_SIZE {
        return Err(ProtocolError::MessageTooLarge {
            size: bytes.len(),
            max: MAX_MESSAGE_SIZE,
        });
    }
    Ok(bytes)
}

/// Decode message from binary (bincode) bytes.
/// ~10-100x faster than JSON for same-machine communication.
pub fn decode_message_binary(bytes: &[u8]) -> Result<IpcMessage, ProtocolError> {
    if bytes.len() > MAX_MESSAGE_SIZE {
        return Err(ProtocolError::MessageTooLarge {
            size: bytes.len(),
            max: MAX_MESSAGE_SIZE,
        });
    }
    bincode::deserialize(bytes).map_err(|e| ProtocolError::InvalidFormat(e.to_string()))
}
