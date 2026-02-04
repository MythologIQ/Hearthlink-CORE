//! Memory pooling for efficient buffer reuse.

use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Configuration for memory pool.
#[derive(Debug, Clone)]
pub struct MemoryPoolConfig {
    pub buffer_size: usize,
    pub max_buffers: usize,
}

impl Default for MemoryPoolConfig {
    fn default() -> Self {
        Self {
            buffer_size: 4096,
            max_buffers: 64,
        }
    }
}

/// A buffer obtained from the memory pool.
pub struct PooledBuffer {
    data: Vec<u8>,
    pool: Arc<Mutex<VecDeque<Vec<u8>>>>,
}

impl PooledBuffer {
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.data
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl Drop for PooledBuffer {
    fn drop(&mut self) {
        let data = std::mem::take(&mut self.data);
        let pool = self.pool.clone();
        tokio::spawn(async move {
            let mut guard = pool.lock().await;
            guard.push_back(data);
        });
    }
}

/// Thread-safe memory pool for buffer reuse.
pub struct MemoryPool {
    buffers: Arc<Mutex<VecDeque<Vec<u8>>>>,
    config: MemoryPoolConfig,
}

impl MemoryPool {
    pub fn new(config: MemoryPoolConfig) -> Self {
        Self {
            buffers: Arc::new(Mutex::new(VecDeque::with_capacity(config.max_buffers))),
            config,
        }
    }

    /// Acquire a buffer from the pool, or allocate a new one.
    pub async fn acquire(&self) -> PooledBuffer {
        let mut guard = self.buffers.lock().await;
        let data = guard.pop_front().unwrap_or_else(|| {
            vec![0u8; self.config.buffer_size]
        });
        drop(guard);

        PooledBuffer {
            data,
            pool: self.buffers.clone(),
        }
    }

    /// Current number of available buffers in pool.
    pub async fn available(&self) -> usize {
        self.buffers.lock().await.len()
    }
}
