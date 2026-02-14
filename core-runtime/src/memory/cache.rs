//! Context caching for inference state.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Configuration for context cache.
#[derive(Debug, Clone)]
pub struct ContextCacheConfig {
    pub max_entries: usize,
    pub ttl: Duration,
}

impl Default for ContextCacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 128,
            ttl: Duration::from_secs(300), // 5 minutes
        }
    }
}

struct CacheEntry {
    data: Vec<u8>,
    created_at: Instant,
}

/// LRU cache for inference context data.
pub struct ContextCache {
    entries: Arc<RwLock<HashMap<String, CacheEntry>>>,
    config: ContextCacheConfig,
}

impl ContextCache {
    pub fn new(config: ContextCacheConfig) -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::with_capacity(config.max_entries))),
            config,
        }
    }

    /// Store context data with the given key.
    pub async fn store(&self, key: String, data: Vec<u8>) {
        let mut entries = self.entries.write().await;

        if entries.len() >= self.config.max_entries {
            self.evict_oldest(&mut entries);
        }

        entries.insert(key, CacheEntry {
            data,
            created_at: Instant::now(),
        });
    }

    /// Retrieve cached context data if valid.
    pub async fn get(&self, key: &str) -> Option<Vec<u8>> {
        let entries = self.entries.read().await;
        let entry = entries.get(key)?;

        if entry.created_at.elapsed() > self.config.ttl {
            return None;
        }

        Some(entry.data.clone())
    }

    /// Remove expired entries.
    pub async fn cleanup(&self) {
        let mut entries = self.entries.write().await;
        entries.retain(|_, entry| entry.created_at.elapsed() <= self.config.ttl);
    }

    fn evict_oldest(&self, entries: &mut HashMap<String, CacheEntry>) {
        if let Some(oldest_key) = entries
            .iter()
            .min_by_key(|(_, e)| e.created_at)
            .map(|(k, _)| k.clone())
        {
            entries.remove(&oldest_key);
        }
    }

    pub async fn len(&self) -> usize {
        self.entries.read().await.len()
    }

    pub async fn is_empty(&self) -> bool {
        self.entries.read().await.is_empty()
    }
}

// KV-Cache for transformer inference optimization

/// KV-cache entry for transformer inference.
/// Pre-allocates capacity to avoid reallocation during generation.
#[derive(Clone)]
pub struct KvCacheEntry {
    pub keys: Vec<f32>,
    pub values: Vec<f32>,
    pub seq_len: usize,
    capacity: usize,
}

impl KvCacheEntry {
    /// Create a new KV-cache entry with pre-allocated capacity.
    pub fn new(hidden_size: usize, max_seq_len: usize) -> Self {
        let capacity = hidden_size * max_seq_len;
        Self {
            keys: Vec::with_capacity(capacity),
            values: Vec::with_capacity(capacity),
            seq_len: 0,
            capacity,
        }
    }

    /// Append new KV entries without reallocation (if within capacity).
    pub fn append(&mut self, new_keys: &[f32], new_values: &[f32]) {
        self.keys.extend_from_slice(new_keys);
        self.values.extend_from_slice(new_values);
        self.seq_len += 1;
    }

    /// Memory usage in bytes.
    pub fn memory_bytes(&self) -> usize {
        (self.keys.capacity() + self.values.capacity()) * std::mem::size_of::<f32>()
    }

    /// Check if at capacity.
    pub fn is_full(&self) -> bool {
        self.keys.len() >= self.capacity
    }

    /// Get pre-allocated capacity.
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

/// Specialized cache for KV tensors with pre-allocation.
/// Optimized for multi-turn generation where KV state is reused.
pub struct KvCache {
    entries: Arc<RwLock<HashMap<String, KvCacheEntry>>>,
    hidden_size: usize,
    max_seq_len: usize,
    max_entries: usize,
}

impl KvCache {
    /// Create a new KV-cache with specified dimensions.
    pub fn new(hidden_size: usize, max_seq_len: usize, max_entries: usize) -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::with_capacity(max_entries))),
            hidden_size,
            max_seq_len,
            max_entries,
        }
    }

    /// Get or create a KV-cache entry for a session.
    pub async fn get_or_create(&self, session_id: &str) -> KvCacheEntry {
        let entries = self.entries.read().await;
        if let Some(entry) = entries.get(session_id) {
            return entry.clone();
        }
        drop(entries);

        KvCacheEntry::new(self.hidden_size, self.max_seq_len)
    }

    /// Update KV-cache for a session.
    pub async fn update(&self, session_id: String, entry: KvCacheEntry) {
        let mut entries = self.entries.write().await;
        if entries.len() >= self.max_entries && !entries.contains_key(&session_id) {
            self.evict_one(&mut entries);
        }
        entries.insert(session_id, entry);
    }

    /// Evict one entry (FIFO-style, not true LRU).
    fn evict_one(&self, entries: &mut HashMap<String, KvCacheEntry>) {
        if let Some(key) = entries.keys().next().cloned() {
            entries.remove(&key);
        }
    }

    /// Remove a specific session's cache.
    pub async fn remove(&self, session_id: &str) {
        let mut entries = self.entries.write().await;
        entries.remove(session_id);
    }

    /// Current number of cached sessions.
    pub async fn len(&self) -> usize {
        self.entries.read().await.len()
    }

    /// Check if cache is empty.
    pub async fn is_empty(&self) -> bool {
        self.entries.read().await.is_empty()
    }
}
