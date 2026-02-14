# Plan: Tier 2 Performance Optimization

## Open Questions

1. **Memory-mapped model loading**: Should we use `memmap2` crate or platform-specific APIs? `memmap2` is more portable but platform APIs may offer better control.

2. **V2 Encoder selection**: Binary encoding options include MessagePack (`rmp-serde`), CBOR (`ciborium`), or custom varint. What's the right balance of speed vs ecosystem support?

3. **Thread pool sizing**: Should we expose thread count as configuration or auto-detect based on `std::thread::available_parallelism()`?

---

## Target Metrics

| Metric | Tier 1 (Current) | Tier 2 (Target) | Improvement |
|--------|------------------|-----------------|-------------|
| Generation | >10 tok/s | >25 tok/s | 2.5x |
| Classification P95 | <100ms | <20ms | 5x |
| Memory Ratio | <1.5x | <1.35x | 10% reduction |

---

## Phase 1: IPC Serialization Optimization

### Affected Files

- `src/ipc/encoding.rs` - Add V2 binary encoder
- `src/ipc/protocol.rs` - Wire V2 encoding into message flow
- `tests/encoding_roundtrip_test.rs` - V2 encoder tests

### Changes

#### 1.1 Implement V2Encoder (Binary)

```rust
// src/ipc/encoding.rs

/// V2 Encoder: Packed binary format for token arrays.
/// Format: [count: u32-le][token0: u32-le][token1: u32-le]...
#[derive(Debug, Clone, Copy, Default)]
pub struct V2Encoder;

impl TokenEncoder for V2Encoder {
    fn encode(&self, tokens: &[u32]) -> Vec<u8> {
        let mut buf = Vec::with_capacity(4 + tokens.len() * 4);
        buf.extend_from_slice(&(tokens.len() as u32).to_le_bytes());
        for token in tokens {
            buf.extend_from_slice(&token.to_le_bytes());
        }
        buf
    }

    fn decode(&self, bytes: &[u8]) -> Result<Vec<u32>, ProtocolError> {
        if bytes.len() < 4 {
            return Err(ProtocolError::InvalidFormat("too short".into()));
        }
        let count = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
        let expected_len = 4 + count * 4;
        if bytes.len() != expected_len {
            return Err(ProtocolError::InvalidFormat("length mismatch".into()));
        }
        let mut tokens = Vec::with_capacity(count);
        for i in 0..count {
            let offset = 4 + i * 4;
            let token = u32::from_le_bytes([
                bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3]
            ]);
            tokens.push(token);
        }
        Ok(tokens)
    }
}
```

#### 1.2 Update get_encoder()

```rust
pub fn get_encoder(version: ProtocolVersion) -> Box<dyn TokenEncoder + Send + Sync> {
    match version {
        ProtocolVersion::V1 => Box::new(V1Encoder),
        ProtocolVersion::V2 => Box::new(V2Encoder),
    }
}
```

### Unit Tests

- `tests/encoding_roundtrip_test.rs`
  - `v2_encode_empty` - Empty array encodes to 4 bytes (count=0)
  - `v2_encode_single` - Single token encodes to 8 bytes
  - `v2_roundtrip` - Round-trip preserves all values
  - `v2_decode_truncated` - Rejects truncated input
  - `v2_decode_length_mismatch` - Rejects count/data mismatch
  - `v2_vs_v1_size_comparison` - Assert V2 is smaller for typical payloads

---

## Phase 2: Memory-Mapped Model Loading

### Affected Files

- `Cargo.toml` - Add `memmap2` dependency
- `src/models/loader.rs` - Add memory-mapped loading
- `src/models/mod.rs` - Export new types
- `tests/integration_gguf_test.rs` - Memory-mapped loading tests

### Changes

#### 2.1 Add Dependency

```toml
[dependencies]
memmap2 = "0.9"
```

#### 2.2 Add MappedModel Type

```rust
// src/models/loader.rs

use memmap2::Mmap;
use std::fs::File;

/// Memory-mapped model for zero-copy loading.
pub struct MappedModel {
    _mmap: Mmap,
    data: *const u8,
    len: usize,
}

// SAFETY: Mmap is Send+Sync when underlying file is not modified
unsafe impl Send for MappedModel {}
unsafe impl Sync for MappedModel {}

impl MappedModel {
    /// Memory-map a model file for zero-copy access.
    pub fn open(path: &ModelPath) -> Result<Self, LoadError> {
        let file = File::open(path.as_path())?;
        let mmap = unsafe { Mmap::map(&file)? };
        let data = mmap.as_ptr();
        let len = mmap.len();
        Ok(Self { _mmap: mmap, data, len })
    }

    /// Get model data as a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.data, self.len) }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}
```

#### 2.3 Update ModelLoader

```rust
impl ModelLoader {
    /// Load model using memory-mapping (zero-copy).
    pub fn load_mapped(&self, model_path: &ModelPath) -> Result<MappedModel, LoadError> {
        MappedModel::open(model_path)
    }
}
```

### Unit Tests

- `tests/integration_gguf_test.rs`
  - `mmap_load_valid_file` - Successfully maps existing file
  - `mmap_load_missing_file` - Returns NotFound error
  - `mmap_data_accessible` - Can read bytes from mapped region
  - `mmap_len_matches_file_size` - Length matches file metadata

---

## Phase 3: KV-Cache Optimization

### Affected Files

- `src/memory/cache.rs` - Optimize cache with typed entries
- `src/engine/gguf/generator.rs` - Integrate KV-cache for generation
- `tests/memory_test.rs` - KV-cache performance tests

### Changes

#### 3.1 Add Typed KV-Cache Entry

```rust
// src/memory/cache.rs

/// KV-cache entry for transformer inference.
#[derive(Clone)]
pub struct KvCacheEntry {
    pub keys: Vec<f32>,
    pub values: Vec<f32>,
    pub seq_len: usize,
}

impl KvCacheEntry {
    pub fn new(hidden_size: usize, max_seq_len: usize) -> Self {
        let capacity = hidden_size * max_seq_len;
        Self {
            keys: Vec::with_capacity(capacity),
            values: Vec::with_capacity(capacity),
            seq_len: 0,
        }
    }

    /// Append new KV entries without reallocation.
    pub fn append(&mut self, new_keys: &[f32], new_values: &[f32]) {
        self.keys.extend_from_slice(new_keys);
        self.values.extend_from_slice(new_values);
        self.seq_len += 1;
    }

    /// Memory usage in bytes.
    pub fn memory_bytes(&self) -> usize {
        (self.keys.capacity() + self.values.capacity()) * std::mem::size_of::<f32>()
    }
}
```

#### 3.2 Add Specialized KV-Cache Store

```rust
// src/memory/cache.rs

/// Specialized cache for KV tensors with pre-allocation.
pub struct KvCache {
    entries: Arc<RwLock<HashMap<String, KvCacheEntry>>>,
    hidden_size: usize,
    max_seq_len: usize,
    max_entries: usize,
}

impl KvCache {
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
            self.evict_oldest(&mut entries);
        }
        entries.insert(session_id, entry);
    }

    fn evict_oldest(&self, entries: &mut HashMap<String, KvCacheEntry>) {
        if let Some(key) = entries.keys().next().cloned() {
            entries.remove(&key);
        }
    }
}
```

### Unit Tests

- `tests/memory_test.rs`
  - `kv_cache_entry_preallocates` - Capacity matches expected size
  - `kv_cache_entry_append_no_realloc` - Append within capacity doesn't reallocate
  - `kv_cache_get_or_create_returns_new` - Missing key creates new entry
  - `kv_cache_update_stores_entry` - Updated entry is retrievable
  - `kv_cache_evicts_when_full` - Oldest entry evicted at capacity

---

## Phase 4: Thread Pool Tuning

### Affected Files

- `src/scheduler/mod.rs` - Add thread pool configuration
- `src/scheduler/pool.rs` - New file for thread pool management
- `src/lib.rs` - Export thread pool types
- `tests/scheduler_test.rs` - Thread pool tests

### Changes

#### 4.1 Create Thread Pool Config

```rust
// src/scheduler/pool.rs

use std::num::NonZeroUsize;

/// Thread pool configuration for inference workers.
#[derive(Debug, Clone)]
pub struct ThreadPoolConfig {
    pub worker_threads: NonZeroUsize,
    pub stack_size: usize,
}

impl Default for ThreadPoolConfig {
    fn default() -> Self {
        let threads = std::thread::available_parallelism()
            .unwrap_or(NonZeroUsize::new(4).unwrap());
        Self {
            worker_threads: threads,
            stack_size: 2 * 1024 * 1024, // 2MB
        }
    }
}

impl ThreadPoolConfig {
    /// Create config with specific thread count.
    pub fn with_threads(count: usize) -> Self {
        Self {
            worker_threads: NonZeroUsize::new(count.max(1)).unwrap(),
            ..Default::default()
        }
    }

    /// Create config optimized for inference (fewer threads, larger stacks).
    pub fn for_inference() -> Self {
        let cores = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);
        Self {
            worker_threads: NonZeroUsize::new((cores / 2).max(2)).unwrap(),
            stack_size: 4 * 1024 * 1024, // 4MB for model weights
        }
    }
}
```

#### 4.2 Update Module Exports

```rust
// src/scheduler/mod.rs
mod pool;
pub use pool::{ThreadPoolConfig};
```

### Unit Tests

- `tests/scheduler_test.rs`
  - `thread_pool_config_default_uses_available` - Default uses available parallelism
  - `thread_pool_config_minimum_one_thread` - At least 1 thread even if 0 requested
  - `thread_pool_config_inference_halves_cores` - Inference mode uses half cores
  - `thread_pool_config_stack_size_reasonable` - Stack size >= 2MB

---

## Summary

| Phase | Focus | Key Deliverable |
|-------|-------|-----------------|
| 1 | IPC Serialization | V2 binary encoder (~50% smaller payloads) |
| 2 | Model Loading | Memory-mapped zero-copy loading |
| 3 | KV-Cache | Pre-allocated typed KV tensors |
| 4 | Thread Pool | Tuned thread configuration |

**Expected Impact**:
- V2 encoding: 10-20% latency reduction on large prompts
- Mmap loading: 30-50% faster cold start, lower RSS
- KV-cache: 40-60% faster multi-turn generation
- Thread tuning: 10-15% throughput improvement

---

_Plan follows Simple Made Easy principles_
