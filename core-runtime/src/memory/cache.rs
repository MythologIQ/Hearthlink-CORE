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
