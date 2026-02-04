//! Token streaming output for incremental responses.

use std::sync::Arc;
use tokio::sync::mpsc;

/// A single streamed token output.
#[derive(Debug, Clone)]
pub struct StreamingOutput {
    pub token: u32,
    pub is_final: bool,
}

/// Async stream of generated tokens.
pub struct TokenStream {
    receiver: mpsc::Receiver<StreamingOutput>,
}

impl TokenStream {
    /// Create a new token stream with sender/receiver pair.
    pub fn new(buffer_size: usize) -> (TokenStreamSender, Self) {
        let (sender, receiver) = mpsc::channel(buffer_size);
        (TokenStreamSender { sender }, Self { receiver })
    }

    /// Receive the next token, if available.
    pub async fn next(&mut self) -> Option<StreamingOutput> {
        self.receiver.recv().await
    }

    /// Collect all remaining tokens into a vector.
    pub async fn collect(mut self) -> Vec<u32> {
        let mut tokens = Vec::new();
        while let Some(output) = self.next().await {
            tokens.push(output.token);
            if output.is_final {
                break;
            }
        }
        tokens
    }
}

/// Sender half for pushing tokens to a stream.
pub struct TokenStreamSender {
    sender: mpsc::Sender<StreamingOutput>,
}

impl TokenStreamSender {
    /// Send a token to the stream.
    pub async fn send(&self, token: u32, is_final: bool) -> Result<(), StreamSendError> {
        self.sender
            .send(StreamingOutput { token, is_final })
            .await
            .map_err(|_| StreamSendError::Closed)
    }

    /// Close the stream by dropping the sender.
    pub fn close(self) {
        drop(self.sender);
    }
}

#[derive(Debug)]
pub struct StreamSendError;

impl std::fmt::Display for StreamSendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "stream closed")
    }
}

impl std::error::Error for StreamSendError {}
