# Plan: Tier 4 Performance Optimization

## Open Questions

1. **Page size for KV-cache**: vLLM uses 16 tokens/page. Should we match or tune for CPU memory hierarchy (cache line size)?

2. **Quantization format**: GGML uses Q4_0, Q4_1, Q8_0. Should we abstract over formats or pick one canonical format?

3. **Continuous batching granularity**: Per-token iteration batching vs per-layer batching? Per-token is simpler but has more overhead.

---

## Target Metrics

| Metric | Tier 3 (Current) | Tier 4 (Target) | Improvement |
|--------|------------------|-----------------|-------------|
| Generation | >50 tok/s | >70 tok/s | 40% |
| Memory Efficiency | <1.25x | <1.15x | 8% reduction |
| Batch Throughput | 8 concurrent | 16 concurrent | 2x |
| KV-Cache Utilization | ~60% | >90% | 50% |

---

## Phase 1: Paged KV-Cache

Replace contiguous KV-cache with paged memory for better utilization and dynamic sizing.

### Affected Files

- `src/memory/paged.rs` - New paged allocator
- `src/memory/mod.rs` - Export paged types
- `src/memory/cache.rs` - Update KvCache to use paged backing
- `tests/memory_test.rs` - Paged allocator tests

### Changes

#### 1.1 PagedAllocator

```rust
// src/memory/paged.rs

/// Fixed-size page for KV-cache storage.
pub struct Page {
    data: Box<[f32; PAGE_TOKENS * HIDDEN_DIM]>,
    used_slots: usize,
}

/// Page table mapping sequence positions to physical pages.
pub struct PageTable {
    entries: Vec<Option<PageId>>,
    free_pages: Vec<PageId>,
    pages: Vec<Page>,
}

impl PageTable {
    /// Allocate page for sequence position range.
    pub fn allocate(&mut self, seq_start: usize) -> Option<PageId>;

    /// Free pages when sequence completes.
    pub fn free(&mut self, page_ids: &[PageId]);

    /// Get page for reading KV at position.
    pub fn get(&self, seq_pos: usize) -> Option<&Page>;
}
```

#### 1.2 Update KvCache to Use Pages

```rust
// src/memory/cache.rs - modify KvCacheEntry

pub struct KvCacheEntry {
    page_table: PageTable,
    seq_len: usize,
}

impl KvCacheEntry {
    /// Append KV without contiguous reallocation.
    pub fn append(&mut self, keys: &[f32], values: &[f32]) -> Result<(), CacheError> {
        if self.needs_new_page() {
            self.page_table.allocate(self.seq_len)?;
        }
        self.write_to_current_page(keys, values);
        self.seq_len += 1;
        Ok(())
    }
}
```

### Unit Tests

- `tests/memory_test.rs`
  - `page_table_allocate_returns_unique_ids` - No page ID collision
  - `page_table_free_recycles_pages` - Freed pages are reusable
  - `paged_kv_append_spans_pages` - Appends cross page boundaries
  - `paged_kv_memory_bounded` - Memory stays within configured limit
  - `page_table_concurrent_allocate` - Thread-safe allocation

---

## Phase 2: Continuous Batching

Replace static batching with iteration-level batching where requests join/leave dynamically.

### Affected Files

- `src/scheduler/continuous.rs` - New continuous batcher
- `src/scheduler/mod.rs` - Export continuous types
- `src/scheduler/batch.rs` - Deprecate static batcher
- `tests/scheduler_test.rs` - Continuous batching tests

### Changes

#### 2.1 ContinuousBatcher

```rust
// src/scheduler/continuous.rs

/// Request state in continuous batch.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RequestPhase {
    Prefill,   // Processing prompt
    Decode,    // Generating tokens
    Complete,  // Finished
}

/// Slot in the continuous batch.
pub struct BatchSlot {
    request_id: RequestId,
    phase: RequestPhase,
    tokens_generated: usize,
    max_tokens: usize,
}

/// Continuous batcher with dynamic membership.
pub struct ContinuousBatcher {
    slots: Vec<Option<BatchSlot>>,
    max_slots: usize,
    pending: VecDeque<QueuedRequest>,
}

impl ContinuousBatcher {
    /// Perform one iteration across all active slots.
    pub async fn step(&mut self) -> Vec<StepResult> {
        self.admit_pending();
        let results = self.run_iteration().await;
        self.evict_completed();
        results
    }

    /// Admit pending requests into free slots.
    fn admit_pending(&mut self);

    /// Remove completed requests, freeing slots.
    fn evict_completed(&mut self);
}
```

#### 2.2 StepResult for Per-Request Output

```rust
// src/scheduler/continuous.rs

pub struct StepResult {
    pub request_id: RequestId,
    pub token: Option<u32>,
    pub finished: bool,
    pub finish_reason: Option<FinishReason>,
}
```

### Unit Tests

- `tests/scheduler_test.rs`
  - `continuous_admits_to_free_slots` - Pending requests fill empty slots
  - `continuous_evicts_on_complete` - Completed requests free slots
  - `continuous_respects_max_slots` - Never exceeds slot limit
  - `continuous_prefill_then_decode` - Phase transitions correctly
  - `continuous_interleaved_requests` - New requests join mid-batch

---

## Phase 3: Quantization Abstraction

Abstract over quantized weight formats for reduced memory and faster matmul.

### Affected Files

- `src/engine/quantize.rs` - Quantization types and ops
- `src/engine/mod.rs` - Export quantization
- `src/engine/gguf/generator.rs` - Use quantized weights
- `tests/quantize_test.rs` - Quantization tests

### Changes

#### 3.1 QuantizedTensor Abstraction

```rust
// src/engine/quantize.rs

/// Supported quantization formats.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum QuantFormat {
    F32,     // No quantization (baseline)
    Q8_0,    // 8-bit symmetric
    Q4_0,    // 4-bit symmetric
}

/// Quantized tensor with format-specific storage.
pub struct QuantizedTensor {
    format: QuantFormat,
    data: Vec<u8>,
    scales: Vec<f32>,
    shape: Vec<usize>,
}

impl QuantizedTensor {
    /// Dequantize block to f32 for computation.
    pub fn dequantize_block(&self, block_idx: usize) -> Vec<f32>;

    /// Quantized matmul (fused dequant + multiply).
    pub fn matmul(&self, input: &[f32], output: &mut [f32]);
}
```

#### 3.2 Block-wise Dequantization

```rust
// src/engine/quantize.rs

const Q4_BLOCK_SIZE: usize = 32;
const Q8_BLOCK_SIZE: usize = 32;

impl QuantizedTensor {
    /// Q4_0 dequantization: 4 bits + 1 scale per 32 elements.
    fn dequant_q4_block(&self, block: &[u8], scale: f32) -> [f32; 32] {
        let mut out = [0.0f32; 32];
        for i in 0..16 {
            let byte = block[i];
            out[i * 2] = (((byte & 0x0F) as i8) - 8) as f32 * scale;
            out[i * 2 + 1] = (((byte >> 4) as i8) - 8) as f32 * scale;
        }
        out
    }
}
```

### Unit Tests

- `tests/quantize_test.rs`
  - `q8_roundtrip_within_tolerance` - Quantize/dequant error < 0.1%
  - `q4_roundtrip_within_tolerance` - Quantize/dequant error < 1%
  - `quantized_matmul_matches_f32` - Results match within tolerance
  - `dequant_block_size_correct` - Block produces expected elements
  - `quant_memory_reduction` - Q4 uses ~8x less memory than F32

---

## Phase 4: Prefill/Decode Separation

Optimize prompt processing (prefill) separately from token generation (decode).

### Affected Files

- `src/engine/prefill.rs` - Prefill-optimized path
- `src/engine/decode.rs` - Decode-optimized path
- `src/engine/mod.rs` - Export prefill/decode
- `src/engine/inference.rs` - Route to appropriate path
- `tests/inference_types_test.rs` - Phase routing tests

### Changes

#### 4.1 PrefillExecutor (Batch-Parallel)

```rust
// src/engine/prefill.rs

/// Prefill executor optimized for parallel prompt processing.
pub struct PrefillExecutor {
    /// Chunk size for parallel processing.
    chunk_size: usize,
}

impl PrefillExecutor {
    /// Process prompt tokens in parallel chunks.
    pub fn execute(
        &self,
        tokens: &[u32],
        kv_cache: &mut KvCacheEntry,
    ) -> Result<PrefillResult, InferenceError> {
        let chunks: Vec<_> = tokens.chunks(self.chunk_size).collect();
        // Process chunks in parallel, aggregate KV
        for chunk in chunks {
            self.process_chunk(chunk, kv_cache)?;
        }
        Ok(PrefillResult { kv_len: tokens.len() })
    }
}
```

#### 4.2 DecodeExecutor (Latency-Optimized)

```rust
// src/engine/decode.rs

/// Decode executor optimized for single-token latency.
pub struct DecodeExecutor {
    /// Enable speculative decoding.
    speculative: Option<SpeculativeConfig>,
}

impl DecodeExecutor {
    /// Generate single token with minimal latency.
    pub fn step(
        &self,
        kv_cache: &KvCacheEntry,
    ) -> Result<u32, InferenceError>;
}
```

#### 4.3 Unified InferenceEngine Routing

```rust
// src/engine/inference.rs - add phase routing

impl InferenceEngine {
    pub async fn generate(
        &self,
        input: &InferenceInput,
        config: &InferenceConfig,
    ) -> Result<GenerationResult, InferenceError> {
        let tokens = self.tokenize(input)?;

        // Phase 1: Prefill (parallel)
        let mut kv_cache = self.prefill.execute(&tokens, ...)?;

        // Phase 2: Decode (sequential, latency-optimized)
        let output = self.decode.generate(&mut kv_cache, config.max_tokens)?;

        self.detokenize(&output)
    }
}
```

### Unit Tests

- `tests/inference_types_test.rs`
  - `prefill_processes_full_prompt` - All prompt tokens processed
  - `prefill_populates_kv_cache` - KV cache has correct length
  - `decode_uses_prefill_kv` - Decode reads from prefill KV
  - `prefill_chunk_size_affects_parallelism` - Larger chunks = fewer tasks
  - `decode_respects_max_tokens` - Generation stops at limit

---

## Summary

| Phase | Focus | Key Deliverable |
|-------|-------|-----------------|
| 1 | Memory | Paged KV-cache with >90% utilization |
| 2 | Scheduling | Continuous batching for 2x concurrency |
| 3 | Compute | Quantization for 4-8x memory reduction |
| 4 | Pipeline | Prefill/decode split for optimal latency |

**Expected Impact**:
- Paged KV: 30-50% memory reduction, dynamic sizing
- Continuous batching: 2x throughput on concurrent requests
- Quantization: 4-8x model memory reduction, ~20% faster matmul
- Prefill/decode: 10-20% latency reduction via specialization

---

_Plan follows Simple Made Easy principles_
