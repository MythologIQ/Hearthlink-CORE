# Plan: Pre-Testing Hardening Bundle

## Open Questions

1. **Unicode normalization depth**: Should we normalize only for blocklist matching or also for regex patterns? NFC normalization is standard, but NFKC collapses more variants (e.g., ﬁ → fi).

2. **Dashmap vs RwLock**: Adding dashmap introduces a new dependency. Is the concurrency improvement worth the dependency cost for current workload?

---

## Phase 1: OutputFilter Security Hardening

### Why

Z.ai Report: *"the lack of Unicode normalization before filtering could allow bypass through homoglyphs or different Unicode encodings of blocked content"*

This is a security gap that could cause filter tests to pass while real-world bypass remains possible.

### Affected Files

- `Cargo.toml` - Add `unicode-normalization` dependency
- `src/engine/filter.rs` - Add NFC normalization, pre-compute lowercase blocklist
- `tests/security_filter_adversarial_test.rs` - Add Unicode bypass tests

### Changes

#### 1.1 Add Dependency

```toml
# Cargo.toml
unicode-normalization = "0.1"
```

#### 1.2 Update FilterConfig

```rust
// src/engine/filter.rs

use unicode_normalization::UnicodeNormalization;

/// Configuration for output filtering.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct FilterConfig {
    #[serde(default)]
    pub blocklist: Vec<String>,
    #[serde(default)]
    pub regex_patterns: Vec<String>,
    #[serde(default)]
    pub max_output_chars: usize,
    #[serde(default = "default_replacement")]
    pub replacement: String,
}

/// Output filter with pre-computed normalized blocklist.
pub struct OutputFilter {
    config: FilterConfig,
    compiled_patterns: Vec<Regex>,
    /// Pre-normalized, pre-lowercased blocklist for fast matching.
    normalized_blocklist: Vec<String>,
}
```

#### 1.3 Update Constructor

```rust
impl OutputFilter {
    pub fn new(config: FilterConfig) -> Result<Self, InferenceError> {
        let compiled = config
            .regex_patterns
            .iter()
            .map(|p| Regex::new(p))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| InferenceError::InputValidation(format!("invalid regex: {}", e)))?;

        // Pre-compute normalized lowercase blocklist
        let normalized_blocklist = config
            .blocklist
            .iter()
            .map(|s| s.nfc().collect::<String>().to_lowercase())
            .collect();

        Ok(Self {
            config,
            compiled_patterns: compiled,
            normalized_blocklist,
        })
    }
}
```

#### 1.4 Update filter() Method

```rust
impl OutputFilter {
    pub fn filter(&self, text: &str) -> Result<String, InferenceError> {
        let mut result = text.to_string();

        // Normalize input for comparison
        let normalized: String = result.nfc().collect();
        let lower = normalized.to_lowercase();

        // Apply blocklist with pre-computed normalized entries
        for (i, normalized_blocked) in self.normalized_blocklist.iter().enumerate() {
            if lower.contains(normalized_blocked) {
                // Replace original blocklist entry (preserves user's case in replacement)
                result = result.replace(&self.config.blocklist[i], &self.config.replacement);
            }
        }

        // Apply regex patterns
        for pattern in &self.compiled_patterns {
            result = pattern
                .replace_all(&result, &self.config.replacement)
                .to_string();
        }

        // Apply length limit
        if self.config.max_output_chars > 0 && result.len() > self.config.max_output_chars {
            result.truncate(self.config.max_output_chars);
        }

        Ok(result)
    }

    pub fn contains_blocked(&self, text: &str) -> bool {
        let normalized: String = text.nfc().collect();
        let lower = normalized.to_lowercase();

        for normalized_blocked in &self.normalized_blocklist {
            if lower.contains(normalized_blocked) {
                return true;
            }
        }
        for pattern in &self.compiled_patterns {
            if pattern.is_match(text) {
                return true;
            }
        }
        false
    }
}
```

### Unit Tests

- `tests/security_filter_adversarial_test.rs`
  - `unicode_nfc_normalization_blocks_composed` - NFC composed form (é) matches blocklist
  - `unicode_nfc_normalization_blocks_decomposed` - NFD decomposed form (e + combining acute) matches blocklist
  - `unicode_homoglyph_basic` - Cyrillic 'а' (U+0430) doesn't bypass Latin 'a' blocklist (NFC doesn't fix this, but documents limitation)
  - `precomputed_blocklist_case_insensitive` - Mixed case input matches lowercase blocklist
  - `blocklist_no_realloc_per_check` - Verify no new allocations in hot path (blocklist already lowercase)

---

## Phase 2: IPC Binary Encoding Integration

### Why

V2 binary encoder exists but may not be wired into the handler. Verify integration and add benchmark comparison.

### Affected Files

- `src/ipc/handler.rs` - Verify V2 encoding is available
- `src/ipc/protocol.rs` - Verify version negotiation
- `tests/encoding_roundtrip_test.rs` - Add V2 comprehensive tests
- `benches/ipc_throughput.rs` - Add V1 vs V2 comparison

### Changes

#### 2.1 Add V2 Encoder Tests

```rust
// tests/encoding_roundtrip_test.rs

#[test]
fn v2_encode_empty() {
    let encoder = V2Encoder;
    let encoded = encoder.encode(&[]);
    assert_eq!(encoded.len(), 4); // Just the count
    assert_eq!(encoded, [0, 0, 0, 0]);
}

#[test]
fn v2_encode_single() {
    let encoder = V2Encoder;
    let encoded = encoder.encode(&[42]);
    assert_eq!(encoded.len(), 8); // count + 1 token
}

#[test]
fn v2_roundtrip() {
    let encoder = V2Encoder;
    let tokens = vec![1, 2, 3, 100, 1000, 65535, u32::MAX];
    let encoded = encoder.encode(&tokens);
    let decoded = encoder.decode(&encoded).unwrap();
    assert_eq!(tokens, decoded);
}

#[test]
fn v2_decode_truncated() {
    let encoder = V2Encoder;
    let result = encoder.decode(&[1, 0, 0, 0]); // claims 1 token but no data
    assert!(result.is_err());
}

#[test]
fn v2_decode_length_mismatch() {
    let encoder = V2Encoder;
    let result = encoder.decode(&[2, 0, 0, 0, 1, 0, 0, 0]); // claims 2, has 1
    assert!(result.is_err());
}

#[test]
fn v2_smaller_than_v1() {
    let tokens: Vec<u32> = (0..100).collect();
    let v1 = V1Encoder.encode(&tokens);
    let v2 = V2Encoder.encode(&tokens);
    assert!(v2.len() < v1.len(), "V2 should be smaller: {} vs {}", v2.len(), v1.len());
}
```

### Unit Tests

- `tests/encoding_roundtrip_test.rs`
  - `v2_encode_empty` - Empty array produces 4-byte header
  - `v2_encode_single` - Single token produces 8 bytes
  - `v2_roundtrip` - Full round-trip preserves all values including edge cases
  - `v2_decode_truncated` - Rejects truncated input
  - `v2_decode_length_mismatch` - Rejects count/data mismatch
  - `v2_smaller_than_v1` - V2 is smaller for typical payloads (100 tokens)

---

## Phase 3: Session Storage Optimization (Optional)

### Why

Z.ai Report: *"RwLock may become a bottleneck under high concurrency; a lock-free data structure would improve scalability"*

This is optional - only implement if Tier 2 benchmarks show session contention.

### Affected Files

- `Cargo.toml` - Add `dashmap` dependency (optional)
- `src/ipc/auth.rs` - Replace `RwLock<HashMap>` with `DashMap`

### Changes

#### 3.1 Add Dependency (Feature-Gated)

```toml
# Cargo.toml
[dependencies]
dashmap = { version = "5.5", optional = true }

[features]
concurrent-sessions = ["dashmap"]
```

#### 3.2 Conditional Session Storage

```rust
// src/ipc/auth.rs

#[cfg(feature = "concurrent-sessions")]
use dashmap::DashMap;

#[cfg(feature = "concurrent-sessions")]
pub struct SessionAuth {
    sessions: DashMap<SessionToken, Session>,
    expected_token_hash: [u8; 32],
    session_timeout: Duration,
}

#[cfg(not(feature = "concurrent-sessions"))]
pub struct SessionAuth {
    sessions: Arc<RwLock<HashMap<SessionToken, Session>>>,
    expected_token_hash: [u8; 32],
    session_timeout: Duration,
}
```

### Unit Tests

- `tests/auth_test.rs`
  - `concurrent_session_creation` - Multiple threads create sessions simultaneously
  - `concurrent_validation` - Multiple threads validate same session
  - `no_deadlock_under_load` - High contention doesn't deadlock

---

## Summary

| Phase | Focus | Security Impact | Performance Impact |
|-------|-------|-----------------|-------------------|
| 1 | Unicode Normalization | High (prevents filter bypass) | Minimal |
| 2 | V2 Encoding Tests | None | Validates 10-20% latency reduction |
| 3 | DashMap Sessions | None | Optional concurrency improvement |

**Recommended Execution**:
- Phase 1: Required before security tests
- Phase 2: Required for Tier 2 benchmarks
- Phase 3: Only if benchmarks show session contention

---

_Plan follows Simple Made Easy principles_
