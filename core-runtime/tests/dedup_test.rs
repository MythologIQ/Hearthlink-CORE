//! Tests for request deduplication functionality.

use std::time::Duration;

use gg_core::engine::InferenceParams;
use gg_core::scheduler::{OutputCache, OutputCacheConfig};

#[test]
fn test_cache_key_deterministic() {
    let tokens = vec![1, 2, 3, 4, 5];
    let params = InferenceParams::default();

    let key1 = OutputCache::cache_key(&tokens, &params);
    let key2 = OutputCache::cache_key(&tokens, &params);

    assert_eq!(key1, key2);
}

#[test]
fn test_cache_key_differs_by_tokens() {
    let params = InferenceParams::default();

    let key1 = OutputCache::cache_key(&[1, 2, 3], &params);
    let key2 = OutputCache::cache_key(&[1, 2, 4], &params);

    assert_ne!(key1, key2);
}

#[test]
fn test_cache_key_differs_by_params() {
    let tokens = vec![1, 2, 3];

    let params1 = InferenceParams {
        max_tokens: 100,
        ..Default::default()
    };
    let params2 = InferenceParams {
        max_tokens: 200,
        ..Default::default()
    };

    let key1 = OutputCache::cache_key(&tokens, &params1);
    let key2 = OutputCache::cache_key(&tokens, &params2);

    assert_ne!(key1, key2);
}

#[test]
fn test_cache_hit_within_ttl() {
    let config = OutputCacheConfig {
        ttl: Duration::from_secs(60),
        max_entries: 100,
    };
    let mut cache = OutputCache::new(config);

    let tokens = vec![1, 2, 3];
    let params = InferenceParams::default();
    let key = OutputCache::cache_key(&tokens, &params);

    // Insert output
    cache.insert(key, vec![10, 20, 30]);

    // Should hit
    let cached = cache.get(&key);
    assert!(cached.is_some());
    assert_eq!(cached.unwrap().output_tokens, vec![10, 20, 30]);
}

#[test]
fn test_cache_miss_after_ttl() {
    let config = OutputCacheConfig {
        ttl: Duration::from_millis(1), // Very short TTL
        max_entries: 100,
    };
    let mut cache = OutputCache::new(config);

    let tokens = vec![1, 2, 3];
    let params = InferenceParams::default();
    let key = OutputCache::cache_key(&tokens, &params);

    // Insert output
    cache.insert(key, vec![10, 20, 30]);

    // Wait for TTL to expire
    std::thread::sleep(Duration::from_millis(10));

    // Should miss (expired)
    let cached = cache.get(&key);
    assert!(cached.is_none());
}

#[test]
fn test_cache_eviction_at_capacity() {
    let config = OutputCacheConfig {
        ttl: Duration::from_secs(60),
        max_entries: 2, // Very small capacity
    };
    let mut cache = OutputCache::new(config);

    let params = InferenceParams::default();

    // Insert 3 entries (capacity is 2)
    let key1 = OutputCache::cache_key(&[1], &params);
    let key2 = OutputCache::cache_key(&[2], &params);
    let key3 = OutputCache::cache_key(&[3], &params);

    cache.insert(key1, vec![10]);
    std::thread::sleep(Duration::from_millis(1)); // Ensure different timestamps
    cache.insert(key2, vec![20]);
    std::thread::sleep(Duration::from_millis(1));
    cache.insert(key3, vec![30]); // Should evict key1 (oldest)

    // Should be at capacity
    assert_eq!(cache.len(), 2);

    // key1 should be evicted, key2 and key3 should remain
    assert!(cache.get(&key1).is_none());
    assert!(cache.get(&key2).is_some());
    assert!(cache.get(&key3).is_some());
}

#[test]
fn test_cache_cleanup() {
    let config = OutputCacheConfig {
        ttl: Duration::from_millis(1),
        max_entries: 100,
    };
    let mut cache = OutputCache::new(config);

    let params = InferenceParams::default();
    let key1 = OutputCache::cache_key(&[1], &params);
    let key2 = OutputCache::cache_key(&[2], &params);

    cache.insert(key1, vec![10]);
    cache.insert(key2, vec![20]);

    assert_eq!(cache.len(), 2);

    // Wait for TTL to expire
    std::thread::sleep(Duration::from_millis(10));

    // Cleanup should remove all expired entries
    cache.cleanup();

    assert_eq!(cache.len(), 0);
}

#[test]
fn test_cache_empty() {
    let config = OutputCacheConfig::default();
    let cache = OutputCache::new(config);

    assert!(cache.is_empty());
    assert_eq!(cache.len(), 0);
}
