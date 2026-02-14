# Plan: Tier 3 Performance Optimization

## Open Questions

1. **Arena vs Slab allocator**: Arena allocator is simpler (bulk deallocation) but slab provides finer-grained reuse. Which fits inference workload better?

2. **SIMD instruction set**: Should we target AVX2 (broader compatibility) or AVX-512 (better performance on newer CPUs)? Consider runtime detection.

3. **Draft model ratio**: For speculative decoding, what draft-to-target token ratio should we use? Literature suggests 4-8 draft tokens per verification.

---

## Target Metrics

| Metric | Tier 2 (Current) | Tier 3 (Target) | Improvement |
|--------|------------------|-----------------|-------------|
| Generation | >25 tok/s | >50 tok/s | 2x |
| Classification P95 | <20ms | <5ms | 4x |
| Memory Ratio | <1.35x | <1.25x | 8% reduction |

---

## Phase 1: Lock-Free Arena Allocator

### Affected Files

- `src/memory/arena.rs` - New arena allocator implementation
- `src/memory/mod.rs` - Export arena types
- `src/memory/pool.rs` - Update to use arena backing
- `tests/memory_test.rs` - Arena allocator tests

### Changes

#### 1.1 Implement Arena Allocator

```rust
// src/memory/arena.rs

use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Fixed-size memory arena for fast bump allocation.
/// Thread-safe via atomic bump pointer.
pub struct Arena {
    buffer: Box<[UnsafeCell<u8>]>,
    offset: AtomicUsize,
    capacity: usize,
}

// SAFETY: Arena uses atomic operations for thread-safe allocation
unsafe impl Send for Arena {}
unsafe impl Sync for Arena {}

impl Arena {
    /// Create a new arena with given capacity.
    pub fn new(capacity: usize) -> Self {
        let buffer: Vec<UnsafeCell<u8>> = (0..capacity)
            .map(|_| UnsafeCell::new(0))
            .collect();
        Self {
            buffer: buffer.into_boxed_slice(),
            offset: AtomicUsize::new(0),
            capacity,
        }
    }

    /// Allocate `size` bytes with given alignment.
    /// Returns None if arena is exhausted.
    pub fn alloc(&self, size: usize, align: usize) -> Option<*mut u8> {
        loop {
            let current = self.offset.load(Ordering::Relaxed);
            let aligned = (current + align - 1) & !(align - 1);
            let new_offset = aligned + size;
            if new_offset > self.capacity {
                return None;
            }
            if self.offset.compare_exchange_weak(
                current,
                new_offset,
                Ordering::AcqRel,
                Ordering::Relaxed,
            ).is_ok() {
                return Some(self.buffer[aligned].get());
            }
        }
    }

    /// Reset arena for reuse (bulk deallocation).
    pub fn reset(&self) {
        self.offset.store(0, Ordering::Release);
    }

    /// Bytes currently allocated.
    pub fn used(&self) -> usize {
        self.offset.load(Ordering::Relaxed)
    }

    /// Total capacity in bytes.
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}
```

#### 1.2 Add Typed Arena Slice

```rust
// src/memory/arena.rs (continued)

/// A slice allocated from an arena.
pub struct ArenaSlice<'a, T> {
    ptr: *mut T,
    len: usize,
    _marker: std::marker::PhantomData<&'a T>,
}

impl<'a, T> ArenaSlice<'a, T> {
    /// Create slice from arena allocation.
    pub fn new(arena: &'a Arena, len: usize) -> Option<Self> {
        let size = len * std::mem::size_of::<T>();
        let align = std::mem::align_of::<T>();
        let ptr = arena.alloc(size, align)? as *mut T;
        Some(Self {
            ptr,
            len,
            _marker: std::marker::PhantomData,
        })
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr, self.len) }
    }

    pub fn len(&self) -> usize {
        self.len
    }
}
```

#### 1.3 Add Arena Pool for Request-Scoped Allocation

```rust
// src/memory/arena.rs (continued)

use std::collections::VecDeque;
use std::sync::Mutex;

/// Pool of arenas for request-scoped allocation.
pub struct ArenaPool {
    arenas: Mutex<VecDeque<Arena>>,
    arena_size: usize,
    max_arenas: usize,
}

impl ArenaPool {
    pub fn new(arena_size: usize, max_arenas: usize) -> Self {
        Self {
            arenas: Mutex::new(VecDeque::with_capacity(max_arenas)),
            arena_size,
            max_arenas,
        }
    }

    /// Acquire an arena from the pool.
    pub fn acquire(&self) -> Arena {
        let mut guard = self.arenas.lock().unwrap();
        guard.pop_front().unwrap_or_else(|| Arena::new(self.arena_size))
    }

    /// Return arena to pool after resetting.
    pub fn release(&self, arena: Arena) {
        arena.reset();
        let mut guard = self.arenas.lock().unwrap();
        if guard.len() < self.max_arenas {
            guard.push_back(arena);
        }
        // Otherwise drop the arena
    }
}
```

### Unit Tests

- `tests/memory_test.rs`
  - `arena_alloc_sequential` - Sequential allocations don't overlap
  - `arena_alloc_aligned` - Allocations respect alignment
  - `arena_exhaustion` - Returns None when full
  - `arena_reset_allows_reuse` - Reset enables reallocation
  - `arena_pool_acquire_release` - Pool recycles arenas
  - `arena_concurrent_alloc` - Multiple threads allocate safely

---

## Phase 2: SIMD Tokenization

### Affected Files

- `src/engine/simd_tokenizer.rs` - New SIMD-accelerated tokenizer
- `src/engine/mod.rs` - Export SIMD tokenizer
- `src/engine/tokenizer.rs` - Deprecate in favor of SIMD version
- `tests/tokenizer_test.rs` - SIMD tokenizer tests

### Changes

#### 2.1 Implement SIMD Byte Scanning

```rust
// src/engine/simd_tokenizer.rs

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

/// SIMD-accelerated tokenizer for fast text processing.
pub struct SimdTokenizer {
    vocab: Vec<TokenEntry>,
    vocab_map: std::collections::HashMap<Vec<u8>, u32>,
    eos_token: u32,
    bos_token: u32,
}

struct TokenEntry {
    bytes: Vec<u8>,
    id: u32,
}

impl SimdTokenizer {
    /// Find whitespace boundaries using SIMD.
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn find_whitespace_avx2(text: &[u8]) -> Vec<usize> {
        let mut positions = Vec::new();
        let space = _mm256_set1_epi8(b' ' as i8);
        let newline = _mm256_set1_epi8(b'\n' as i8);
        let tab = _mm256_set1_epi8(b'\t' as i8);

        let chunks = text.chunks_exact(32);
        let remainder = chunks.remainder();

        for (chunk_idx, chunk) in chunks.enumerate() {
            let data = _mm256_loadu_si256(chunk.as_ptr() as *const __m256i);
            let is_space = _mm256_cmpeq_epi8(data, space);
            let is_newline = _mm256_cmpeq_epi8(data, newline);
            let is_tab = _mm256_cmpeq_epi8(data, tab);
            let is_ws = _mm256_or_si256(
                _mm256_or_si256(is_space, is_newline),
                is_tab,
            );
            let mask = _mm256_movemask_epi8(is_ws) as u32;
            let base = chunk_idx * 32;
            for bit in 0..32 {
                if (mask >> bit) & 1 == 1 {
                    positions.push(base + bit);
                }
            }
        }

        // Handle remainder without SIMD
        let base = text.len() - remainder.len();
        for (i, &b) in remainder.iter().enumerate() {
            if b == b' ' || b == b'\n' || b == b'\t' {
                positions.push(base + i);
            }
        }

        positions
    }

    /// Fallback scalar implementation.
    fn find_whitespace_scalar(text: &[u8]) -> Vec<usize> {
        text.iter()
            .enumerate()
            .filter(|(_, &b)| b == b' ' || b == b'\n' || b == b'\t')
            .map(|(i, _)| i)
            .collect()
    }

    /// Find whitespace with automatic dispatch.
    pub fn find_whitespace(text: &[u8]) -> Vec<usize> {
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx2") {
                return unsafe { Self::find_whitespace_avx2(text) };
            }
        }
        Self::find_whitespace_scalar(text)
    }
}
```

#### 2.2 Implement Fast BPE Encoding

```rust
// src/engine/simd_tokenizer.rs (continued)

impl SimdTokenizer {
    /// Load vocabulary from file.
    pub fn from_vocab(
        vocab_bytes: &[u8],
        eos_token: u32,
        bos_token: u32,
    ) -> Result<Self, TokenizerError> {
        let mut vocab = Vec::new();
        let mut vocab_map = std::collections::HashMap::new();

        // Parse vocab (format: id<tab>bytes<newline>)
        for (id, line) in vocab_bytes.split(|&b| b == b'\n').enumerate() {
            if line.is_empty() {
                continue;
            }
            let token_bytes = line.to_vec();
            vocab_map.insert(token_bytes.clone(), id as u32);
            vocab.push(TokenEntry {
                bytes: token_bytes,
                id: id as u32,
            });
        }

        Ok(Self { vocab, vocab_map, eos_token, bos_token })
    }

    /// Encode text to token IDs using greedy BPE.
    pub fn encode(&self, text: &str) -> Vec<u32> {
        let bytes = text.as_bytes();
        let mut tokens = Vec::with_capacity(bytes.len() / 4);
        let mut pos = 0;

        while pos < bytes.len() {
            let mut best_len = 1;
            let mut best_id = bytes[pos] as u32; // Fallback to byte

            // Greedy: find longest matching token
            for len in (1..=bytes.len() - pos).rev().take(16) {
                if let Some(&id) = self.vocab_map.get(&bytes[pos..pos + len]) {
                    best_len = len;
                    best_id = id;
                    break;
                }
            }

            tokens.push(best_id);
            pos += best_len;
        }

        tokens
    }

    /// Decode token IDs to text.
    pub fn decode(&self, tokens: &[u32]) -> Result<String, TokenizerError> {
        let mut bytes = Vec::new();
        for &id in tokens {
            if let Some(entry) = self.vocab.get(id as usize) {
                bytes.extend_from_slice(&entry.bytes);
            } else {
                return Err(TokenizerError::InvalidToken(id));
            }
        }
        String::from_utf8(bytes).map_err(|e| {
            TokenizerError::DecodingFailed(e.to_string())
        })
    }

    pub fn eos_token(&self) -> u32 {
        self.eos_token
    }

    pub fn bos_token(&self) -> u32 {
        self.bos_token
    }
}
```

### Unit Tests

- `tests/tokenizer_test.rs`
  - `simd_whitespace_matches_scalar` - SIMD and scalar produce same results
  - `simd_encode_roundtrip` - Encode then decode recovers original
  - `simd_encode_empty` - Empty string produces empty tokens
  - `simd_encode_unicode` - Unicode text encodes correctly
  - `simd_decode_invalid_token` - Invalid token ID returns error
  - `simd_performance_vs_scalar` - SIMD is faster on large inputs

---

## Phase 3: Speculative Decoding

### Affected Files

- `src/engine/speculative.rs` - New speculative decoding implementation
- `src/engine/gguf/generator.rs` - Integrate speculative decoding
- `src/engine/mod.rs` - Export speculative types
- `tests/speculative_test.rs` - Speculative decoding tests

### Changes

#### 3.1 Implement Draft-Verify Loop

```rust
// src/engine/speculative.rs

use crate::engine::{GenerationResult, InferenceError};

/// Configuration for speculative decoding.
#[derive(Debug, Clone)]
pub struct SpeculativeConfig {
    /// Number of draft tokens to generate before verification.
    pub draft_tokens: usize,
    /// Acceptance threshold (0.0 to 1.0).
    pub acceptance_threshold: f32,
    /// Enable speculative decoding.
    pub enabled: bool,
}

impl Default for SpeculativeConfig {
    fn default() -> Self {
        Self {
            draft_tokens: 4,
            acceptance_threshold: 0.9,
            enabled: true,
        }
    }
}

/// Speculative decoding executor.
pub struct SpeculativeDecoder<D, T> {
    draft_model: D,
    target_model: T,
    config: SpeculativeConfig,
}

impl<D, T> SpeculativeDecoder<D, T>
where
    D: DraftModel,
    T: TargetModel,
{
    pub fn new(draft_model: D, target_model: T, config: SpeculativeConfig) -> Self {
        Self { draft_model, target_model, config }
    }

    /// Generate tokens using speculative decoding.
    pub async fn generate(
        &self,
        prompt_tokens: &[u32],
        max_tokens: u32,
    ) -> Result<Vec<u32>, InferenceError> {
        let mut output = Vec::with_capacity(max_tokens as usize);
        let mut context = prompt_tokens.to_vec();

        while output.len() < max_tokens as usize {
            // Draft phase: generate candidate tokens
            let draft = self.draft_model
                .generate_draft(&context, self.config.draft_tokens)
                .await?;

            // Verify phase: check draft against target
            let verified = self.target_model
                .verify_tokens(&context, &draft)
                .await?;

            // Accept verified tokens
            let accepted = Self::accept_tokens(&draft, &verified);
            if accepted.is_empty() {
                // Fallback: use target model directly
                let token = self.target_model.generate_one(&context).await?;
                output.push(token);
                context.push(token);
            } else {
                output.extend_from_slice(&accepted);
                context.extend_from_slice(&accepted);
            }

            // Check for EOS
            if output.last().copied() == self.target_model.eos_token() {
                break;
            }
        }

        Ok(output)
    }

    fn accept_tokens(draft: &[u32], verified: &VerifyResult) -> Vec<u32> {
        let mut accepted = Vec::new();
        for (i, &token) in draft.iter().enumerate() {
            if i < verified.accepted_count {
                accepted.push(token);
            } else {
                break;
            }
        }
        // Add correction token if verification diverged
        if let Some(correction) = verified.correction_token {
            accepted.push(correction);
        }
        accepted
    }
}
```

#### 3.2 Define Model Traits

```rust
// src/engine/speculative.rs (continued)

/// Result from verification phase.
pub struct VerifyResult {
    pub accepted_count: usize,
    pub correction_token: Option<u32>,
}

/// Draft model trait for generating candidate tokens.
#[async_trait::async_trait]
pub trait DraftModel: Send + Sync {
    async fn generate_draft(
        &self,
        context: &[u32],
        count: usize,
    ) -> Result<Vec<u32>, InferenceError>;
}

/// Target model trait for verification.
#[async_trait::async_trait]
pub trait TargetModel: Send + Sync {
    async fn verify_tokens(
        &self,
        context: &[u32],
        draft: &[u32],
    ) -> Result<VerifyResult, InferenceError>;

    async fn generate_one(&self, context: &[u32]) -> Result<u32, InferenceError>;

    fn eos_token(&self) -> Option<u32>;
}
```

#### 3.3 Integrate with GgufGenerator

```rust
// src/engine/gguf/generator.rs - add speculative support

impl GgufGenerator {
    /// Generate with speculative decoding if draft model available.
    pub async fn generate_speculative(
        &self,
        prompt_tokens: &[u32],
        max_tokens: u32,
        draft_model: Option<&dyn DraftModel>,
    ) -> Result<GenerationResult, InferenceError> {
        if let Some(draft) = draft_model {
            // Use speculative decoding
            let decoder = SpeculativeDecoder::new(
                draft,
                self,
                SpeculativeConfig::default(),
            );
            let tokens = decoder.generate(prompt_tokens, max_tokens).await?;
            // Convert tokens to text via tokenizer
            return self.tokens_to_result(&tokens);
        }

        // Fallback to standard generation
        self.generate_standard(prompt_tokens, max_tokens).await
    }
}
```

### Unit Tests

- `tests/speculative_test.rs`
  - `speculative_accepts_matching_draft` - Matching drafts fully accepted
  - `speculative_rejects_divergent_draft` - Divergent drafts partially rejected
  - `speculative_fallback_on_empty_accept` - Falls back when draft rejected
  - `speculative_stops_at_eos` - Generation stops at EOS token
  - `speculative_config_draft_tokens` - Respects draft token count
  - `speculative_disabled_uses_standard` - Disabled config uses standard path

---

## Summary

| Phase | Focus | Key Deliverable |
|-------|-------|-----------------|
| 1 | Memory Allocation | Lock-free arena with pool recycling |
| 2 | Tokenization | AVX2-accelerated whitespace/BPE |
| 3 | Generation | Speculative decode with draft-verify loop |

**Expected Impact**:
- Arena allocator: 20-30% latency reduction (no async mutex)
- SIMD tokenization: 3-5x faster encode/decode
- Speculative decoding: 2-3x generation throughput

---

_Plan follows Simple Made Easy principles_
