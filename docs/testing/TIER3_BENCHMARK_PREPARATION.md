# Tier 3 Benchmark Preparation

**Date:** 2026-02-16  
**Status:** 📋 READY  
**Prerequisites:** Tier 2 Complete ✅, Security Review Complete ✅

---

## Executive Summary

This document outlines the benchmarking infrastructure and preparation required for Tier 3 testing. Tier 3 focuses on optimized performance with near-parity to unsandboxed runtimes.

### Tier 3 Performance Targets

| Metric                      | Target   | Tier 1 Baseline | Gap                 |
| --------------------------- | -------- | --------------- | ------------------- |
| Generation Throughput       | 50 tok/s | N/A (no model)  | Model-dependent     |
| Classification P95 Latency  | 5 ms     | <1 ms (infra)   | Model-dependent     |
| Embedding P95 Latency       | 3 ms     | <1 ms (infra)   | Model-dependent     |
| Memory Ratio vs Unsandboxed | 1.25x    | N/A             | Optimization target |

---

## Existing Benchmark Infrastructure

### Criterion Benchmarks (6 suites)

| Benchmark             | File                                                                        | Purpose               | Status   |
| --------------------- | --------------------------------------------------------------------------- | --------------------- | -------- |
| IPC Throughput        | [`ipc_throughput.rs`](core-runtime/benches/ipc_throughput.rs)               | Message encode/decode | ✅ Ready |
| Scheduler Throughput  | [`scheduler_throughput.rs`](core-runtime/benches/scheduler_throughput.rs)   | Request scheduling    | ✅ Ready |
| Inference Latency     | [`inference_latency.rs`](core-runtime/benches/inference_latency.rs)         | Input validation      | ✅ Ready |
| Generation Throughput | [`generation_throughput.rs`](core-runtime/benches/generation_throughput.rs) | Output creation       | ✅ Ready |
| Memory Overhead       | [`memory_overhead.rs`](core-runtime/benches/memory_overhead.rs)             | Allocation tracking   | ✅ Ready |
| Concurrent Load       | [`concurrent_load.rs`](core-runtime/benches/concurrent_load.rs)             | Parallel requests     | ✅ Ready |

### Test Fixtures Required

```
fixtures/
├── prompts/
│   ├── small.json    # 100 tokens
│   ├── medium.json   # 1,000 tokens
│   └── large.json    # 4,000 tokens
└── models/
    ├── tinybert-classifier.onnx   # 60 MB
    ├── minilm-embedder.onnx       # 80 MB
    ├── phi3-mini-q4km.gguf        # 2.2 GB
    └── smollm-360m-q8.gguf        # 400 MB
```

---

## Benchmark Execution Plan

### Phase 1: Infrastructure Benchmarks (No Model)

```bash
# Run all criterion benchmarks
cargo bench --features full

# Individual benchmarks
cargo bench --features full -- ipc_throughput
cargo bench --features full -- scheduler_throughput
cargo bench --features full -- inference_latency
cargo bench --features full -- generation_throughput
cargo bench --features full -- memory_overhead
cargo bench --features full -- concurrent_load
```

### Phase 2: ONNX Model Benchmarks

```bash
# Build with ONNX backend
cargo build --release --features onnx

# Run ONNX classification tests
cargo test --release --features onnx --test tier2_onnx_classification_test -- --nocapture

# Benchmark classification latency
cargo bench --features onnx -- inference_latency
```

### Phase 3: GGUF Model Benchmarks

```powershell
# Set LLVM environment
$env:LIBCLANG_PATH = "C:/Program Files/llvm15.0.7/bin"
$env:CMAKE_GENERATOR = "Visual Studio 17 2022"

# Build with GGUF backend
cargo build --release --features gguf

# Run GGUF integration tests
cargo test --release --features gguf --test integration_gguf_test -- --nocapture
```

### Phase 4: Full Backend Benchmarks

```bash
# Build with both backends
cargo build --release --features full

# Run all integration tests
cargo test --release --features full --test integration_onnx_test
cargo test --release --features full --test integration_gguf_test
cargo test --release --features full --test integration_end_to_end_test
```

---

## Performance Measurement Methodology

### Latency Measurements

```rust
// P50, P95, P99 latency tracking
use std::time::{Instant, Duration};

struct LatencyTracker {
    samples: Vec<Duration>,
}

impl LatencyTracker {
    fn record(&mut self, duration: Duration) {
        self.samples.push(duration);
    }

    fn p50(&self) -> Duration {
        self.percentile(50)
    }

    fn p95(&self) -> Duration {
        self.percentile(95)
    }

    fn p99(&self) -> Duration {
        self.percentile(99)
    }

    fn percentile(&self, p: u8) -> Duration {
        let mut sorted: Vec<_> = self.samples.iter().collect();
        sorted.sort();
        let idx = (sorted.len() as f64 * (p as f64 / 100.0)) as usize;
        *sorted[idx.min(sorted.len() - 1)]
    }
}
```

### Throughput Measurements

```rust
// Tokens per second calculation
struct ThroughputMeter {
    total_tokens: usize,
    start_time: Instant,
}

impl ThroughputMeter {
    fn new() -> Self {
        Self {
            total_tokens: 0,
            start_time: Instant::now(),
        }
    }

    fn add_tokens(&mut self, count: usize) {
        self.total_tokens += count;
    }

    fn tokens_per_sec(&self) -> f64 {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        self.total_tokens as f64 / elapsed
    }
}
```

### Memory Measurements

```rust
// Memory overhead tracking
use std::process;

struct MemoryTracker {
    baseline: usize,
}

impl MemoryTracker {
    fn new() -> Self {
        Self {
            baseline: Self::current_memory(),
        }
    }

    fn current_memory() -> usize {
        // Platform-specific memory query
        #[cfg(windows)]
        {
            use std::mem::MaybeUninit;
            use windows_sys::Win32::System::ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};
            // Implementation details...
        }
        0 // Stub
    }

    fn overhead(&self) -> usize {
        Self::current_memory() - self.baseline
    }

    fn ratio_vs_baseline(&self, baseline: usize) -> f64 {
        Self::current_memory() as f64 / baseline as f64
    }
}
```

---

## Optimization Features for Tier 3

### Enabled Features (from `tier3.json`)

| Feature                | Description             | Expected Impact             |
| ---------------------- | ----------------------- | --------------------------- |
| `v2_encoding`          | Packed varint encoding  | 10-100x faster IPC          |
| `mmap_loading`         | Zero-copy model loading | Reduced memory              |
| `kv_cache`             | KV cache for generation | Faster generation           |
| `thread_pool`          | Optimized thread pool   | Better CPU utilization      |
| `arena_allocator`      | Arena allocation        | Reduced allocation overhead |
| `simd_tokenizer`       | SIMD tokenization       | Faster tokenization         |
| `speculative_decoding` | Speculative decoding    | Higher throughput           |

### Feature Flags Implementation Status

| Feature                | Status         | Notes                     |
| ---------------------- | -------------- | ------------------------- |
| `onnx`                 | ✅ Implemented | Candle ONNX backend       |
| `gguf`                 | ✅ Implemented | llama-cpp-2 backend       |
| `v2_encoding`          | ✅ Implemented | Binary encoding available |
| `mmap_loading`         | ✅ Implemented | memmap2 integration       |
| `kv_cache`             | 📋 Planned     | Paged attention cache     |
| `thread_pool`          | 📋 Planned     | Tokio thread pool tuning  |
| `arena_allocator`      | 📋 Planned     | Bump allocation           |
| `simd_tokenizer`       | 📋 Planned     | SIMD-accelerated          |
| `speculative_decoding` | 📋 Planned     | Draft model speculation   |

---

## Comparison Baselines

### Ollama Comparison (from OLLAMA_COMPARISON_ANALYSIS.md)

| Metric              | Ollama | CORE Target | Ratio |
| ------------------- | ------ | ----------- | ----- |
| Generation (tok/s)  | 45-55  | 50          | ~1.0x |
| Classification (ms) | 3-8    | 5           | ~1.0x |
| Memory overhead     | 1.0x   | 1.25x       | 1.25x |
| Startup time (s)    | 2-5    | 1-2         | <1.0x |

### Competitive Analysis

| Runtime      | Generation      | Classification | Memory | Security       |
| ------------ | --------------- | -------------- | ------ | -------------- |
| CORE         | 50 tok/s target | 5ms target     | 1.25x  | ✅ Sandboxed   |
| Ollama       | 45-55 tok/s     | 3-8ms          | 1.0x   | ❌ Unsandboxed |
| llama.cpp    | 50-60 tok/s     | N/A            | 1.0x   | ❌ Unsandboxed |
| ONNX Runtime | N/A             | 2-5ms          | 1.0x   | ⚠️ Partial     |

---

## Benchmark Execution Scripts

### Full Benchmark Script (PowerShell)

```powershell
# bench_full.ps1
Write-Host "=== CORE Runtime Tier 3 Benchmarks ===" -ForegroundColor Cyan

# Environment setup
$env:LIBCLANG_PATH = "C:/Program Files/llvm15.0.7/bin"
$env:CMAKE_GENERATOR = "Visual Studio 17 2022"

# Build release with all features
Write-Host "Building release with full features..." -ForegroundColor Yellow
cargo build --release --features full

if ($LASTEXITCODE -ne 0) {
    Write-Host "Build failed!" -ForegroundColor Red
    exit 1
}

# Run benchmarks
Write-Host "`nRunning IPC throughput benchmarks..." -ForegroundColor Yellow
cargo bench --features full -- ipc_throughput

Write-Host "`nRunning scheduler throughput benchmarks..." -ForegroundColor Yellow
cargo bench --features full -- scheduler_throughput

Write-Host "`nRunning inference latency benchmarks..." -ForegroundColor Yellow
cargo bench --features full -- inference_latency

Write-Host "`nRunning generation throughput benchmarks..." -ForegroundColor Yellow
cargo bench --features full -- generation_throughput

Write-Host "`nRunning memory overhead benchmarks..." -ForegroundColor Yellow
cargo bench --features full -- memory_overhead

Write-Host "`nRunning concurrent load benchmarks..." -ForegroundColor Yellow
cargo bench --features full -- concurrent_load

Write-Host "`n=== Benchmarks Complete ===" -ForegroundColor Green
```

### Quick Benchmark Script

```powershell
# bench_quick.ps1
cargo bench --features full -- --save-baseline tier3
```

---

## Expected Results

### Infrastructure Benchmarks (Tier 1 Baseline)

| Benchmark        | Expected Range  | Target       |
| ---------------- | --------------- | ------------ |
| IPC encode       | 100-150 Melem/s | >100 Melem/s |
| IPC decode       | 20-30 Melem/s   | >20 Melem/s  |
| Scheduler        | 2-5 Melem/s     | >1 Melem/s   |
| Input validation | 2-5 ns          | <10 ns       |
| Memory acquire   | 1-2 µs          | <2 µs        |
| Result creation  | 80-120 ns       | <200 ns      |

### Model Benchmarks (Tier 3 Targets)

| Model               | Operation      | Target     | Notes        |
| ------------------- | -------------- | ---------- | ------------ |
| tinybert-classifier | Classification | <5ms P95   | ONNX backend |
| minilm-embedder     | Embedding      | <3ms P95   | ONNX backend |
| phi3-mini-q4km      | Generation     | >50 tok/s  | GGUF backend |
| smollm-360m-q8      | Generation     | >100 tok/s | GGUF backend |

---

## Reporting

### Benchmark Report Template

```markdown
# Tier 3 Benchmark Results

**Date:** YYYY-MM-DD
**Build:** release --features full
**Platform:** Windows 10

## Infrastructure Benchmarks

| Benchmark  | Result    | Target | Status |
| ---------- | --------- | ------ | ------ |
| IPC encode | X Melem/s | >100   | ✅/❌  |
| IPC decode | X Melem/s | >20    | ✅/❌  |
| ...        | ...       | ...    | ...    |

## Model Benchmarks

| Model     | Operation      | Latency/Throughput | Target | Status |
| --------- | -------------- | ------------------ | ------ | ------ |
| tinybert  | Classification | Xms P95            | <5ms   | ✅/❌  |
| phi3-mini | Generation     | X tok/s            | >50    | ✅/❌  |

## Memory Analysis

| Metric       | Value | Target | Status |
| ------------ | ----- | ------ | ------ |
| Memory ratio | X.XXx | <1.25x | ✅/❌  |

## Comparison vs Baseline

| Metric         | Tier 1 | Tier 3 | Change |
| -------------- | ------ | ------ | ------ |
| Total overhead | X µs   | Y µs   | +/-Z%  |
```

---

## Next Steps

1. ✅ **Verify benchmark fixtures exist** - Check `fixtures/prompts/*.json`
2. ✅ **Verify test models available** - Check `fixtures/models/`
3. ⬜ **Run infrastructure benchmarks** - `cargo bench --features full`
4. ⬜ **Run ONNX model benchmarks** - With test models
5. ⬜ **Run GGUF model benchmarks** - With test models
6. ⬜ **Generate comparison report** - vs Ollama baseline
7. ⬜ **Identify optimization opportunities** - From benchmark results

---

## Checklist for Tier 3 Readiness

- [x] Tier 2 testing complete (37/37 tests pass)
- [x] Security review complete
- [x] Benchmark infrastructure in place
- [ ] Test fixtures available
- [ ] Test models available
- [ ] Environment configured (LLVM, CMAKE)
- [ ] Baseline metrics documented

---

**Document Version:** 1.0  
**Last Updated:** 2026-02-16T03:52:00Z
