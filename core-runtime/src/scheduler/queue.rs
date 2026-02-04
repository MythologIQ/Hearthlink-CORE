//! Request queue management.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;

use super::priority::{Priority, PriorityQueue};
use crate::engine::InferenceParams;

/// Configuration for request queue.
#[derive(Debug, Clone)]
pub struct RequestQueueConfig {
    pub max_pending: usize,
}

impl Default for RequestQueueConfig {
    fn default() -> Self {
        Self { max_pending: 256 }
    }
}

/// A queued inference request.
#[derive(Debug, Clone)]
pub struct QueuedRequest {
    pub id: u64,
    pub model_id: String,
    pub prompt_tokens: Vec<u32>,
    pub params: InferenceParams,
}

/// Thread-safe request queue with priority support.
pub struct RequestQueue {
    queue: Arc<Mutex<PriorityQueue<QueuedRequest>>>,
    next_id: AtomicU64,
    config: RequestQueueConfig,
}

impl RequestQueue {
    pub fn new(config: RequestQueueConfig) -> Self {
        Self {
            queue: Arc::new(Mutex::new(PriorityQueue::new())),
            next_id: AtomicU64::new(1),
            config,
        }
    }

    /// Enqueue a new request. Returns request ID and queue position.
    pub async fn enqueue(
        &self,
        model_id: String,
        prompt_tokens: Vec<u32>,
        params: InferenceParams,
        priority: Priority,
    ) -> Result<(u64, usize), QueueError> {
        let mut queue = self.queue.lock().await;

        if queue.len() >= self.config.max_pending {
            return Err(QueueError::QueueFull);
        }

        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let request = QueuedRequest { id, model_id, prompt_tokens, params };
        let position = queue.len();
        queue.push(request, priority);

        Ok((id, position))
    }

    /// Dequeue the highest priority request.
    pub async fn dequeue(&self) -> Option<QueuedRequest> {
        self.queue.lock().await.pop()
    }

    /// Current queue length.
    pub async fn len(&self) -> usize {
        self.queue.lock().await.len()
    }

    /// Check if queue is empty.
    pub async fn is_empty(&self) -> bool {
        self.queue.lock().await.is_empty()
    }
}

#[derive(Debug)]
pub enum QueueError {
    QueueFull,
}

impl std::fmt::Display for QueueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::QueueFull => write!(f, "request queue is full"),
        }
    }
}

impl std::error::Error for QueueError {}
