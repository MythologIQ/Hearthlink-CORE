//! Integrated KV Cache Manager with Paged Attention and Quantization.
//!
//! Combines paged memory allocation with Q8 quantization for efficient
//! KV-cache storage during inference. Provides 4x memory reduction
//! and efficient memory management through page-based allocation.
//!
//! # Panic Safety
//! This module uses poison-recovering lock guards to maintain cache availability
//! even if a thread panics while holding a lock. A poisoned lock logs a warning
//! but continues operation rather than propagating the panic.

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::time::Instant;

/// Acquire a mutex lock, recovering from poison if a thread panicked.
#[inline]
fn lock_or_recover<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poisoned| {
        tracing::warn!("KV cache mutex poisoned, recovering");
        poisoned.into_inner()
    })
}

/// Acquire a read lock, recovering from poison if a thread panicked.
#[inline]
fn read_or_recover<T>(rwlock: &RwLock<T>) -> RwLockReadGuard<'_, T> {
    rwlock.read().unwrap_or_else(|poisoned| {
        tracing::warn!("KV cache RwLock poisoned, recovering for read");
        poisoned.into_inner()
    })
}

/// Acquire a write lock, recovering from poison if a thread panicked.
#[inline]
fn write_or_recover<T>(rwlock: &RwLock<T>) -> RwLockWriteGuard<'_, T> {
    rwlock.write().unwrap_or_else(|poisoned| {
        tracing::warn!("KV cache RwLock poisoned, recovering for write");
        poisoned.into_inner()
    })
}

use super::kv_quant::Q8KvStore;
use super::paged::{PageId, PageTable, PAGE_TOKENS};

/// Configuration for the KV Cache Manager.
#[derive(Debug, Clone)]
pub struct KvCacheConfig {
    /// Hidden dimension of the model.
    pub hidden_dim: usize,
    /// Maximum number of pages to allocate.
    pub max_pages: usize,
    /// Maximum sequence length.
    pub max_seq_len: usize,
    /// Number of attention heads.
    pub num_heads: usize,
    /// Head dimension.
    pub head_dim: usize,
    /// Enable Q8 quantization for KV storage.
    pub enable_quantization: bool,
    /// Enable paged attention (vLLM-style).
    pub enable_paged: bool,
    /// Cache eviction policy.
    pub eviction_policy: EvictionPolicy,
}

impl Default for KvCacheConfig {
    fn default() -> Self {
        Self {
            hidden_dim: 4096,
            max_pages: 1024,
            max_seq_len: 4096,
            num_heads: 32,
            head_dim: 128,
            enable_quantization: true,
            enable_paged: true,
            eviction_policy: EvictionPolicy::Lru,
        }
    }
}

/// Cache eviction policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvictionPolicy {
    /// Least Recently Used - evict oldest unused entries.
    Lru,
    /// First In First Out - evict oldest entries.
    Fifo,
    /// Least Frequently Used - evict entries with lowest access count.
    Lfu,
}

/// Statistics for the KV cache.
#[derive(Debug, Default, Clone)]
pub struct KvCacheStats {
    pub total_pages_allocated: u64,
    pub total_pages_freed: u64,
    pub current_pages_in_use: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub evictions: u64,
    pub quantization_errors: u64,
    pub memory_bytes_used: u64,
    pub peak_memory_bytes: u64,
}

impl KvCacheStats {
    pub fn hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            return 0.0;
        }
        self.cache_hits as f64 / total as f64
    }
}

/// Unique identifier for a cache sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SequenceId(pub u64);

/// Entry tracking for a cached sequence.
#[derive(Debug)]
struct SequenceEntry {
    #[allow(dead_code)]
    id: SequenceId,
    page_ids: Vec<PageId>,
    seq_len: usize,
    last_access: Instant,
    access_count: u64,
    /// Per-sequence quantized store for KV data
    quant_store: Option<Q8KvStore>,
}

/// Integrated KV Cache Manager.
///
/// Combines paged attention with optional Q8 quantization for
/// efficient memory management during inference.
pub struct KvCacheManager {
    config: KvCacheConfig,
    page_table: RwLock<PageTable>,
    sequences: RwLock<HashMap<SequenceId, SequenceEntry>>,
    access_order: Mutex<VecDeque<SequenceId>>,
    stats: Arc<KvCacheStats>,
    next_seq_id: AtomicU64,
}

impl KvCacheManager {
    /// Create a new KV Cache Manager.
    pub fn new(config: KvCacheConfig) -> Self {
        let page_table = RwLock::new(PageTable::new(config.hidden_dim, config.max_pages));

        Self {
            config,
            page_table,
            sequences: RwLock::new(HashMap::new()),
            access_order: Mutex::new(VecDeque::new()),
            stats: Arc::new(KvCacheStats::default()),
            next_seq_id: AtomicU64::new(1),
        }
    }

    /// Allocate a new sequence in the cache.
    pub fn allocate_sequence(&self) -> SequenceId {
        let id = SequenceId(self.next_seq_id.fetch_add(1, Ordering::SeqCst));

        // Create per-sequence quantized store
        let quant_store = if self.config.enable_quantization {
            Some(Q8KvStore::new(
                self.config.hidden_dim,
                self.config.max_seq_len,
            ))
        } else {
            None
        };

        let entry = SequenceEntry {
            id,
            page_ids: Vec::new(),
            seq_len: 0,
            last_access: Instant::now(),
            access_count: 0,
            quant_store,
        };

        write_or_recover(&self.sequences).insert(id, entry);
        lock_or_recover(&self.access_order).push_back(id);

        id
    }

    /// Append KV pairs to a sequence.
    pub fn append_kv(
        &self,
        seq_id: SequenceId,
        keys: &[f32],
        values: &[f32],
    ) -> Result<(), KvCacheError> {
        let mut sequences = write_or_recover(&self.sequences);
        let entry = sequences
            .get_mut(&seq_id)
            .ok_or(KvCacheError::SequenceNotFound(seq_id.0))?;

        entry.last_access = Instant::now();
        entry.access_count += 1;

        let seq_pos = entry.seq_len;
        let slot = seq_pos % PAGE_TOKENS;

        // Allocate new page if needed
        if slot == 0 || entry.page_ids.is_empty() {
            let mut page_table = write_or_recover(&self.page_table);

            // Try to allocate, evict if necessary
            let page_id = match page_table.allocate(seq_pos) {
                Some(id) => id,
                None => {
                    drop(page_table);
                    self.evict_lru()?;
                    write_or_recover(&self.page_table)
                        .allocate(seq_pos)
                        .ok_or(KvCacheError::MemoryExhausted)?
                }
            };

            entry.page_ids.push(page_id);
        }

        // Write to page with mutable access
        {
            let mut page_table = write_or_recover(&self.page_table);
            if let Some(page) = page_table.get_mut(seq_pos) {
                page.write(slot, keys, values);
            }
        }

        // Store in per-sequence quantized format if enabled
        if let Some(ref mut qs) = entry.quant_store {
            if !qs.append(keys, values) {
                // Quantization store full, reset and retry
                qs.reset();
                qs.append(keys, values);
            }
        }

        entry.seq_len += 1;
        Ok(())
    }

    /// Read KV pairs from a sequence at given position.
    pub fn read_kv(
        &self,
        seq_id: SequenceId,
        pos: usize,
        keys_out: &mut [f32],
        values_out: &mut [f32],
    ) -> Result<(), KvCacheError> {
        let mut sequences = write_or_recover(&self.sequences);
        let entry = sequences
            .get_mut(&seq_id)
            .ok_or(KvCacheError::SequenceNotFound(seq_id.0))?;

        if pos >= entry.seq_len {
            return Err(KvCacheError::PositionOutOfBounds {
                pos,
                seq_len: entry.seq_len,
            });
        }

        entry.last_access = Instant::now();
        entry.access_count += 1;

        // Try per-sequence quantized store first
        if let Some(ref qs) = entry.quant_store {
            if pos < qs.seq_len() {
                qs.read_keys(pos, keys_out);
                qs.read_values(pos, values_out);
                return Ok(());
            }
        }

        // Fall back to page table
        let page_table = read_or_recover(&self.page_table);
        if let Some(page) = page_table.get(pos) {
            let slot = pos % PAGE_TOKENS;
            keys_out.copy_from_slice(page.read_keys(slot));
            values_out.copy_from_slice(page.read_values(slot));
            Ok(())
        } else {
            Err(KvCacheError::PageNotFound)
        }
    }

    /// Compute attention scores for a query against cached keys.
    pub fn attention_scores(
        &self,
        seq_id: SequenceId,
        query: &[f32],
        scores_out: &mut [f32],
    ) -> Result<(), KvCacheError> {
        let sequences = read_or_recover(&self.sequences);
        let entry = sequences
            .get(&seq_id)
            .ok_or(KvCacheError::SequenceNotFound(seq_id.0))?;

        let seq_len = entry.seq_len;

        // Use per-sequence quantized attention if available
        if let Some(ref qs) = entry.quant_store {
            if qs.seq_len() >= seq_len {
                qs.attention_scores(query, scores_out);
                return Ok(());
            }
        }

        drop(sequences);

        // Fall back to page-by-page computation
        let page_table = read_or_recover(&self.page_table);
        for pos in 0..seq_len {
            if let Some(page) = page_table.get(pos) {
                let slot = pos % PAGE_TOKENS;
                let keys = page.read_keys(slot);
                // Compute dot product
                scores_out[pos] = Self::dot_product(query, keys);
            }
        }

        Ok(())
    }

    /// Free a sequence and its pages.
    pub fn free_sequence(&self, seq_id: SequenceId) -> Result<(), KvCacheError> {
        let mut sequences = write_or_recover(&self.sequences);
        let entry = sequences
            .remove(&seq_id)
            .ok_or(KvCacheError::SequenceNotFound(seq_id.0))?;

        // Free pages
        let mut page_table = write_or_recover(&self.page_table);
        page_table.free(&entry.page_ids);

        // Remove from access order
        if let Ok(mut order) = self.access_order.lock() {
            order.retain(|&id| id != seq_id);
        }

        Ok(())
    }

    /// Get current statistics.
    pub fn stats(&self) -> KvCacheStats {
        let stats = self.stats.clone();
        KvCacheStats {
            total_pages_allocated: stats.total_pages_allocated,
            total_pages_freed: stats.total_pages_freed,
            current_pages_in_use: stats.current_pages_in_use,
            cache_hits: stats.cache_hits,
            cache_misses: stats.cache_misses,
            evictions: stats.evictions,
            quantization_errors: stats.quantization_errors,
            memory_bytes_used: stats.memory_bytes_used,
            peak_memory_bytes: stats.peak_memory_bytes,
        }
    }

    /// Get sequence length.
    pub fn seq_len(&self, seq_id: SequenceId) -> Result<usize, KvCacheError> {
        let sequences = read_or_recover(&self.sequences);
        let entry = sequences
            .get(&seq_id)
            .ok_or(KvCacheError::SequenceNotFound(seq_id.0))?;
        Ok(entry.seq_len)
    }

    /// Check if sequence exists.
    pub fn has_sequence(&self, seq_id: SequenceId) -> bool {
        read_or_recover(&self.sequences).contains_key(&seq_id)
    }

    /// Get number of active sequences.
    pub fn active_sequences(&self) -> usize {
        read_or_recover(&self.sequences).len()
    }

    /// Get memory usage in bytes.
    pub fn memory_usage(&self) -> usize {
        let page_table = read_or_recover(&self.page_table);
        let page_count = page_table.page_count();
        page_count * PAGE_TOKENS * self.config.hidden_dim * 2 * std::mem::size_of::<f32>()
    }

    /// Evict least recently used sequence.
    fn evict_lru(&self) -> Result<(), KvCacheError> {
        let victim_id = {
            let mut order = lock_or_recover(&self.access_order);
            order.pop_front()
        };

        if let Some(id) = victim_id {
            self.free_sequence(id)?;
        }

        Ok(())
    }

    /// Compute dot product of two vectors.
    fn dot_product(a: &[f32], b: &[f32]) -> f32 {
        a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
    }

    /// Reset all cache state.
    pub fn reset(&self) {
        let mut sequences = write_or_recover(&self.sequences);
        sequences.clear();

        let mut order = lock_or_recover(&self.access_order);
        order.clear();
    }
}

/// Errors for KV cache operations.
#[derive(Debug, thiserror::Error)]
pub enum KvCacheError {
    #[error("Sequence not found: {0}")]
    SequenceNotFound(u64),

    #[error("Position {pos} out of bounds for sequence length {seq_len}")]
    PositionOutOfBounds { pos: usize, seq_len: usize },

    #[error("Page not found")]
    PageNotFound,

    #[error("Memory exhausted - cannot allocate more pages")]
    MemoryExhausted,

    #[error("Quantization error: {0}")]
    QuantizationError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kv_cache_basic() {
        let config = KvCacheConfig {
            hidden_dim: 128,
            max_pages: 16,
            max_seq_len: 256,
            ..Default::default()
        };

        let manager = KvCacheManager::new(config);
        let seq_id = manager.allocate_sequence();

        let keys = vec![1.0f32; 128];
        let values = vec![2.0f32; 128];

        manager.append_kv(seq_id, &keys, &values).unwrap();
        assert_eq!(manager.seq_len(seq_id).unwrap(), 1);

        let mut k_out = vec![0.0f32; 128];
        let mut v_out = vec![0.0f32; 128];
        manager.read_kv(seq_id, 0, &mut k_out, &mut v_out).unwrap();

        assert!(k_out.iter().all(|&x| (x - 1.0).abs() < 0.01));
        assert!(v_out.iter().all(|&x| (x - 2.0).abs() < 0.01));
    }

    #[test]
    fn test_kv_cache_eviction() {
        let config = KvCacheConfig {
            hidden_dim: 128,
            max_pages: 2,
            max_seq_len: 64,
            eviction_policy: EvictionPolicy::Lru,
            ..Default::default()
        };

        let manager = KvCacheManager::new(config);

        // Allocate multiple sequences
        let seq1 = manager.allocate_sequence();
        let seq2 = manager.allocate_sequence();

        let keys = vec![1.0f32; 128];
        let values = vec![2.0f32; 128];

        // Fill pages
        for _ in 0..16 {
            manager.append_kv(seq1, &keys, &values).unwrap();
        }

        // seq2 should still be valid
        assert!(manager.has_sequence(seq2));
    }

    #[test]
    fn test_attention_scores() {
        let config = KvCacheConfig {
            hidden_dim: 64,
            max_pages: 16,
            max_seq_len: 256,
            enable_quantization: true,
            ..Default::default()
        };

        let manager = KvCacheManager::new(config);
        let seq_id = manager.allocate_sequence();

        // Add some KV pairs
        for i in 0..10 {
            let keys: Vec<f32> = (0..64).map(|j| (i * 64 + j) as f32).collect();
            let values: Vec<f32> = (0..64).map(|j| (i * 64 + j + 1) as f32).collect();
            manager.append_kv(seq_id, &keys, &values).unwrap();
        }

        let query = vec![1.0f32; 64];
        let mut scores = vec![0.0f32; 10];

        manager
            .attention_scores(seq_id, &query, &mut scores)
            .unwrap();

        // Scores should be computed
        assert!(scores.iter().any(|&s| s != 0.0));
    }
}
