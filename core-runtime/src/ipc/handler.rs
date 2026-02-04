//! Request/response handling for IPC connections.

use std::sync::Arc;
use thiserror::Error;

use super::auth::{AuthError, SessionAuth, SessionToken};
use super::protocol::{
    decode_message, encode_message, InferenceRequest, InferenceResponse,
    IpcMessage, ProtocolError, RequestId,
};
use crate::scheduler::queue::RequestQueue;
use crate::scheduler::Priority;

#[derive(Error, Debug)]
pub enum HandlerError {
    #[error("Protocol error: {0}")]
    Protocol(#[from] ProtocolError),

    #[error("Authentication error: {0}")]
    Auth(#[from] AuthError),

    #[error("Not authenticated")]
    NotAuthenticated,

    #[error("Queue error: {0}")]
    QueueFull(String),
}

/// Configuration for IPC handler.
#[derive(Debug, Clone)]
pub struct IpcHandlerConfig {
    pub require_auth: bool,
}

impl Default for IpcHandlerConfig {
    fn default() -> Self {
        Self { require_auth: true }
    }
}

/// Handles IPC message processing with authentication.
pub struct IpcHandler {
    auth: Arc<SessionAuth>,
    queue: Arc<RequestQueue>,
    config: IpcHandlerConfig,
}

impl IpcHandler {
    pub fn new(
        auth: Arc<SessionAuth>,
        queue: Arc<RequestQueue>,
        config: IpcHandlerConfig,
    ) -> Self {
        Self { auth, queue, config }
    }

    /// Process incoming message bytes and return response bytes.
    pub async fn process(
        &self,
        bytes: &[u8],
        session: Option<&SessionToken>,
    ) -> Result<(Vec<u8>, Option<SessionToken>), HandlerError> {
        let message = decode_message(bytes)?;
        let (response, new_session) = self.handle_message(message, session).await?;
        let response_bytes = encode_message(&response)?;
        Ok((response_bytes, new_session))
    }

    async fn handle_message(
        &self,
        message: IpcMessage,
        session: Option<&SessionToken>,
    ) -> Result<(IpcMessage, Option<SessionToken>), HandlerError> {
        match message {
            IpcMessage::Handshake { token } => {
                let session_token = self.auth.authenticate(&token).await?;
                let response = IpcMessage::HandshakeAck {
                    session_id: session_token.as_str().to_string(),
                };
                Ok((response, Some(session_token)))
            }

            IpcMessage::InferenceRequest(request) => {
                self.require_auth(session).await?;
                let response = self.handle_inference(request).await;
                Ok((IpcMessage::InferenceResponse(response), None))
            }

            _ => {
                let error = IpcMessage::Error {
                    code: 400,
                    message: "Unexpected message type".into(),
                };
                Ok((error, None))
            }
        }
    }

    async fn require_auth(&self, session: Option<&SessionToken>) -> Result<(), HandlerError> {
        if !self.config.require_auth {
            return Ok(());
        }

        let token = session.ok_or(HandlerError::NotAuthenticated)?;
        self.auth.validate(token).await?;
        Ok(())
    }

    async fn handle_inference(&self, request: InferenceRequest) -> InferenceResponse {
        if let Err(e) = request.validate() {
            return InferenceResponse::error(request.request_id, e.to_string());
        }

        let enqueue_result = self.queue.enqueue(
            request.model_id,
            request.prompt_tokens,
            request.parameters,
            Priority::Normal,
        ).await;

        match enqueue_result {
            Ok(_) => InferenceResponse::success(request.request_id, Vec::new(), false),
            Err(e) => InferenceResponse::error(request.request_id, e.to_string()),
        }
    }
}
