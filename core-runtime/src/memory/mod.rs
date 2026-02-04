//! Memory management module for CORE Runtime.
//!
//! Provides pooled memory allocation, GPU memory tracking, and context caching.

mod cache;
mod gpu;
mod pool;

pub use cache::{ContextCache, ContextCacheConfig};
pub use gpu::{GpuMemory, GpuMemoryConfig};
pub use pool::{MemoryPool, MemoryPoolConfig, PooledBuffer};
