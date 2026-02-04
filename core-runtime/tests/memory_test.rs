//! TDD-Light tests for memory module.

use core_runtime::memory::{
    ContextCache, ContextCacheConfig, GpuMemory, GpuMemoryConfig, GpuMemoryError,
    MemoryPool, MemoryPoolConfig,
};
use std::time::Duration;

#[tokio::test]
async fn memory_pool_acquire_returns_buffer() {
    let pool = MemoryPool::new(MemoryPoolConfig {
        buffer_size: 1024,
        max_buffers: 4,
    });

    let buffer = pool.acquire().await;

    assert_eq!(buffer.len(), 1024);
}

#[tokio::test]
async fn memory_pool_reuses_returned_buffers() {
    let pool = MemoryPool::new(MemoryPoolConfig {
        buffer_size: 1024,
        max_buffers: 4,
    });

    // Acquire and drop a buffer
    {
        let _buffer = pool.acquire().await;
    }

    // Give time for async drop to complete
    tokio::time::sleep(Duration::from_millis(10)).await;

    let available = pool.available().await;
    assert_eq!(available, 1);
}

#[test]
fn gpu_memory_reserve_tracks_allocation() {
    let gpu = GpuMemory::new(GpuMemoryConfig {
        max_bytes: 1024,
    });

    let reservation = gpu.reserve(256).unwrap();

    assert_eq!(gpu.allocated(), 256);
    assert_eq!(gpu.available(), 768);
    assert_eq!(reservation.bytes(), 256);
}

#[test]
fn gpu_memory_reserve_fails_when_exhausted() {
    let gpu = GpuMemory::new(GpuMemoryConfig {
        max_bytes: 1024,
    });

    let result = gpu.reserve(2048);

    assert!(matches!(result, Err(GpuMemoryError::OutOfMemory { .. })));
}

#[test]
fn gpu_memory_release_frees_allocation() {
    let gpu = GpuMemory::new(GpuMemoryConfig {
        max_bytes: 1024,
    });

    let reservation = gpu.reserve(256).unwrap();
    gpu.release(reservation);

    assert_eq!(gpu.allocated(), 0);
    assert_eq!(gpu.available(), 1024);
}

#[tokio::test]
async fn context_cache_store_and_retrieve() {
    let cache = ContextCache::new(ContextCacheConfig {
        max_entries: 10,
        ttl: Duration::from_secs(60),
    });

    cache.store("key1".to_string(), vec![1, 2, 3]).await;
    let result = cache.get("key1").await;

    assert_eq!(result, Some(vec![1, 2, 3]));
}

#[tokio::test]
async fn context_cache_returns_none_for_missing() {
    let cache = ContextCache::new(ContextCacheConfig::default());

    let result = cache.get("nonexistent").await;

    assert!(result.is_none());
}

#[tokio::test]
async fn context_cache_evicts_when_full() {
    let cache = ContextCache::new(ContextCacheConfig {
        max_entries: 2,
        ttl: Duration::from_secs(60),
    });

    cache.store("key1".to_string(), vec![1]).await;
    cache.store("key2".to_string(), vec![2]).await;
    cache.store("key3".to_string(), vec![3]).await;

    assert_eq!(cache.len().await, 2);
}
