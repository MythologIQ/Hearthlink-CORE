//! TDD-Light tests for memory module.

use veritas_sdr::memory::{
    Arena, ArenaPool, ArenaSlice, ContextCache, ContextCacheConfig, GpuMemory,
    GpuMemoryConfig, GpuMemoryError, KvCache, KvCacheEntry, MemoryPool, MemoryPoolConfig,
};
use std::time::Duration;

#[test]
fn memory_pool_acquire_returns_buffer() {
    let pool = MemoryPool::new(MemoryPoolConfig {
        buffer_size: 1024,
        max_buffers: 4,
    });

    let buffer = pool.acquire();

    assert_eq!(buffer.len(), 1024);
}

#[test]
fn memory_pool_reuses_returned_buffers() {
    let pool = MemoryPool::new(MemoryPoolConfig {
        buffer_size: 1024,
        max_buffers: 4,
    });

    // Acquire and drop a buffer
    {
        let _buffer = pool.acquire();
    }

    // Buffer is now synchronously returned to pool
    let available = pool.available();
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

// KV-Cache tests

#[test]
fn kv_cache_entry_preallocates() {
    let hidden_size = 64;
    let max_seq_len = 128;
    let entry = KvCacheEntry::new(hidden_size, max_seq_len);

    // Should have capacity for hidden_size * max_seq_len floats
    let expected_capacity = hidden_size * max_seq_len;
    assert_eq!(entry.capacity(), expected_capacity);
    assert_eq!(entry.seq_len, 0);
    assert!(entry.keys.is_empty());
    assert!(entry.values.is_empty());
}

#[test]
fn kv_cache_entry_append_no_realloc() {
    let hidden_size = 64;
    let max_seq_len = 128;
    let mut entry = KvCacheEntry::new(hidden_size, max_seq_len);

    let initial_key_capacity = entry.keys.capacity();
    let initial_value_capacity = entry.values.capacity();

    // Append within capacity
    let new_keys: Vec<f32> = vec![1.0; hidden_size];
    let new_values: Vec<f32> = vec![2.0; hidden_size];
    entry.append(&new_keys, &new_values);

    // Should not have reallocated
    assert_eq!(entry.keys.capacity(), initial_key_capacity);
    assert_eq!(entry.values.capacity(), initial_value_capacity);
    assert_eq!(entry.seq_len, 1);
}

#[tokio::test]
async fn kv_cache_get_or_create_returns_new() {
    let cache = KvCache::new(64, 128, 10);

    let entry = cache.get_or_create("session_1").await;

    // Should get a new entry with correct capacity
    assert_eq!(entry.capacity(), 64 * 128);
    assert_eq!(entry.seq_len, 0);
}

#[tokio::test]
async fn kv_cache_update_stores_entry() {
    let cache = KvCache::new(64, 128, 10);

    let mut entry = KvCacheEntry::new(64, 128);
    entry.append(&vec![1.0; 64], &vec![2.0; 64]);

    cache.update("session_1".to_string(), entry).await;

    let retrieved = cache.get_or_create("session_1").await;
    assert_eq!(retrieved.seq_len, 1);
    assert_eq!(retrieved.keys.len(), 64);
}

#[test]
fn kv_cache_evicts_when_full() {
    let cache = KvCache::new(64, 128, 2);

    let entry1 = KvCacheEntry::new(64, 128);
    let entry2 = KvCacheEntry::new(64, 128);
    let entry3 = KvCacheEntry::new(64, 128);

    cache.update_sync("session_1".to_string(), entry1);
    cache.update_sync("session_2".to_string(), entry2);
    cache.update_sync("session_3".to_string(), entry3);

    // Should have evicted one entry
    assert_eq!(cache.len_sync(), 2);
}

#[test]
fn kv_cache_entry_memory_bytes() {
    let hidden_size = 64;
    let max_seq_len = 128;
    let entry = KvCacheEntry::new(hidden_size, max_seq_len);

    let expected_bytes = (hidden_size * max_seq_len) * 2 * std::mem::size_of::<f32>();
    assert_eq!(entry.memory_bytes(), expected_bytes);
}

// Arena allocator tests

#[test]
fn arena_alloc_sequential() {
    let arena = Arena::new(1024);

    let ptr1 = arena.alloc(64, 8).unwrap();
    let ptr2 = arena.alloc(64, 8).unwrap();

    // Allocations should not overlap
    let diff = (ptr2 as usize).wrapping_sub(ptr1 as usize);
    assert!(diff >= 64, "allocations overlap");
}

#[test]
fn arena_alloc_aligned() {
    let arena = Arena::new(1024);

    // Allocate with 16-byte alignment
    let ptr = arena.alloc(32, 16).unwrap();

    assert_eq!(ptr as usize % 16, 0, "allocation not aligned");
}

#[test]
fn arena_exhaustion() {
    let arena = Arena::new(128);

    // First allocation succeeds
    let result1 = arena.alloc(100, 8);
    assert!(result1.is_some());

    // Second allocation exceeds capacity
    let result2 = arena.alloc(100, 8);
    assert!(result2.is_none());
}

#[test]
fn arena_reset_allows_reuse() {
    let arena = Arena::new(128);

    arena.alloc(100, 8).unwrap();
    assert!(arena.used() > 0);

    arena.reset();

    assert_eq!(arena.used(), 0);
    let result = arena.alloc(100, 8);
    assert!(result.is_some());
}

#[test]
fn arena_pool_acquire_release() {
    let pool = ArenaPool::new(1024, 4);

    // Pool starts empty
    assert_eq!(pool.available(), 0);

    // Acquire creates new arena
    let arena1 = pool.acquire();
    assert_eq!(arena1.capacity(), 1024);

    // Release returns to pool
    pool.release(arena1);
    assert_eq!(pool.available(), 1);

    // Next acquire reuses pooled arena
    let arena2 = pool.acquire();
    assert_eq!(arena2.capacity(), 1024);
    assert_eq!(pool.available(), 0);
}

#[test]
fn arena_concurrent_alloc() {
    use std::sync::Arc;
    use std::thread;

    let arena = Arc::new(Arena::new(1024 * 1024)); // 1MB
    let mut handles = vec![];

    for _ in 0..4 {
        let arena = Arc::clone(&arena);
        handles.push(thread::spawn(move || {
            for _ in 0..100 {
                let ptr = arena.alloc(64, 8);
                assert!(ptr.is_some());
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Should have allocated 4 threads * 100 allocations * 64 bytes = 25600 bytes (+ alignment)
    assert!(arena.used() >= 4 * 100 * 64);
}

#[test]
fn arena_slice_typed() {
    let arena = Arena::new(1024);

    let mut slice: ArenaSlice<u32> = ArenaSlice::new(&arena, 10).unwrap();
    assert_eq!(slice.len(), 10);
    assert!(!slice.is_empty());

    // Write and read values
    let data = slice.as_mut_slice();
    for (i, val) in data.iter_mut().enumerate() {
        *val = i as u32;
    }

    let read = slice.as_slice();
    for (i, &val) in read.iter().enumerate() {
        assert_eq!(val, i as u32);
    }
}
