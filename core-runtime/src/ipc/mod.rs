//! IPC module for CORE Runtime.
//!
//! Handles named pipe/Unix socket communication with authenticated callers.
//! This is the ONLY external interface - no HTTP/REST/WebSocket allowed.

mod auth;
mod handler;
mod protocol;

pub use auth::{AuthError, SessionAuth, SessionToken};
pub use handler::{IpcHandler, IpcHandlerConfig};
pub use protocol::{
    InferenceRequest, InferenceResponse, IpcMessage, ProtocolError, RequestId,
};
