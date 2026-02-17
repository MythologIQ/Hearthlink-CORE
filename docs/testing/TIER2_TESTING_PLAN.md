# Tier 2 Testing Plan

**Date:** 2026-02-15  
**Status:** 🔄 IN PROGRESS  
**Objective:** Validate end-to-end performance with actual model inference

---

## Executive Summary

Tier 2 testing focuses on validating end-to-end performance with actual model inference using ONNX and GGUF backends. This is the critical phase that will determine real-world performance and competitive positioning against Ollama.

**Current Status:**

- ✅ Infrastructure optimization verified (94% improvement)
- ✅ ONNX backend compiled successfully
- ❌ GGUF backend blocked by bindgen DLL issue
- ⏸️ Model files: tinybert-classifier.onnx available (22 KB), others need download

**Immediate Priority:** Create end-to-end inference tests and run with available ONNX model.

---

## Tier 2 Performance Targets

### Target Metrics

| Metric                     | Target        | Status     |
| -------------------------- | ------------- | ---------- |
| Generation throughput      | 25 tokens/sec | 🔄 Pending |
| Classification P95 latency | 20 ms         | 🔄 Pending |
| Embedding P95 latency      | 10 ms         | 🔄 Pending |
| Memory ratio               | 1.35          | 🔄 Pending |
| Total latency              | <100 ms       | 🔄 Pending |

### Model Requirements

| Backend | Model                    | Size   | Status                    | Purpose                 |
| ------- | ------------------------ | ------ | ------------------------- | ----------------------- |
| ONNX    | tinybert-classifier.onnx | 60 MB  | ✅ Available (22 KB)      | Classification latency  |
| ONNX    | minilm-embedder.onnx     | 80 MB  | ⏸️ Placeholder (15 bytes) | Embedding tests         |
| GGUF    | phi3-mini-q4km.gguf      | 2.2 GB | ⏸️ Placeholder (15 bytes) | Generation throughput   |
| GGUF    | smollm-360m-q8.gguf      | 400 MB | ⏸️ Not downloaded         | Fast inference baseline |

---

## Current Issues

### 1. GGUF Backend Build Failure

**Error:** bindgen v0.72.1 DLL compatibility issue

**Root Cause:**

- bindgen expects `clang.dll` or `libclang.dll` on Windows
- LLVM distribution provides `clang.exe` executable
- bindgen cannot locate the required DLL files

**Impact:**

- GGUF backend validation cannot proceed until resolved
- Cannot test generation throughput with GGUF models
- ONNX backend validation remains unaffected

**Resolution Options:**

1. **Recommended:** Downgrade bindgen to v0.69.4
2. **Alternative:** Use LLVM distribution with DLL files
3. **Alternative:** Use pre-built bindings
4. **Alternative:** Use alternative GGUF crate (candle-gguf, ggml)

**Documentation:** See [`GGUF_BUILD_TROUBLESHOOTING.md`](GGUF_BUILD_TROUBLESHOOTING.md)

### 2. Model File Availability

**Available Models:**

- ✅ tinybert-classifier.onnx (22,842 bytes - real model)

**Placeholder Models (Need Download):**

- ⏸️ minilm-embedder.onnx (15 bytes)
- ⏸️ phi3-mini-q4km.gguf (15 bytes - needs 2.2 GB)
- ⏸️ smollm-360m-q8.gguf (not downloaded)

**Download Scripts:**

- `core-runtime/scripts/download_models.ps1` - PowerShell script for GGUF model download

---

## Testing Strategy

### Phase 1: ONNX Backend Testing (Immediate)

**Objective:** Validate ONNX backend with available tinybert-classifier.onnx model

**Test Plan:**

1. **End-to-End Classification Test**
   - Load tinybert-classifier.onnx model
   - Run classification inference with sample inputs
   - Measure end-to-end latency
   - Validate against 20 ms P95 target
   - Measure memory usage

2. **Performance Benchmarking**
   - Run multiple classification requests
   - Measure throughput (requests/second)
   - Measure P95 latency
   - Compare against infrastructure overhead (~361 ns)

3. **Resource Utilization**
   - Measure CPU usage during inference
   - Measure memory usage
   - Validate memory ratio target (1.35)

**Success Criteria:**

- ✅ Classification P95 latency <20 ms
- ✅ Memory ratio <1.35
- ✅ Total latency (infrastructure + inference) <100 ms

### Phase 2: ONNX Embedding Testing (After Model Download)

**Prerequisites:**

- Download minilm-embedder.onnx (~80 MB)

**Test Plan:**

1. **End-to-End Embedding Test**
   - Load minilm-embedder.onnx model
   - Run embedding inference with sample inputs
   - Measure end-to-end latency
   - Validate against 10 ms P95 target

2. **Performance Benchmarking**
   - Run multiple embedding requests
   - Measure throughput (embeddings/second)
   - Measure P95 latency

**Success Criteria:**

- ✅ Embedding P95 latency <10 ms
- ✅ Memory ratio <1.35
- ✅ Total latency <100 ms

### Phase 3: GGUF Backend Testing (After Build Fix)

**Prerequisites:**

- Resolve bindgen DLL compatibility issue
- Download phi3-mini-q4km.gguf (~2.2 GB)
- Download smollm-360m-q8.gguf (~400 MB)

**Test Plan:**

1. **End-to-End Generation Test**
   - Load phi3-mini-q4km.gguf model
   - Run text generation with sample prompts
   - Measure end-to-end latency
   - Measure tokens generated per second

2. **Performance Benchmarking**
   - Run multiple generation requests
   - Measure throughput (tokens/second)
   - Validate against 25 tokens/sec target

3. **Resource Utilization**
   - Measure CPU usage during generation
   - Measure memory usage
   - Validate memory ratio target (1.35)

**Success Criteria:**

- ✅ Generation throughput >25 tokens/sec
- ✅ Memory ratio <1.35
- ✅ Total latency <100 ms

---

## Test Implementation Plan

### 1. Create End-to-End Inference Tests

**File:** `core-runtime/tests/tier2_onnx_classification_test.rs`

**Test Cases:**

```rust
#[tokio::test]
async fn test_tinybert_classification_end_to_end() {
    // Load model
    // Run inference
    // Measure latency
    // Validate results
}

#[tokio::test]
async fn test_tinybert_classification_performance() {
    // Run multiple requests
    // Measure P95 latency
    // Measure throughput
}

#[tokio::test]
async fn test_tinybert_resource_utilization() {
    // Measure CPU usage
    // Measure memory usage
    // Validate memory ratio
}
```

### 2. Create Performance Benchmarking Suite

**File:** `core-runtime/benches/tier2_inference_bench.rs`

**Benchmarks:**

```rust
fn bench_tinybert_classification_latency(c: &mut Criterion) {
    // Measure single request latency
}

fn bench_tinybert_classification_throughput(c: &mut Criterion) {
    // Measure requests/second
}

fn bench_tinybert_classification_p95(c: &mut Criterion) {
    // Measure P95 latency
}
```

### 3. Create Resource Monitoring

**File:** `core-runtime/tests/tier2_resource_test.rs`

**Test Cases:**

```rust
#[tokio::test]
async fn test_memory_ratio_compliance() {
    // Validate memory ratio <1.35
}

#[tokio::test]
async fn test_cpu_utilization() {
    // Measure CPU usage during inference
}
```

---

## Execution Steps

### Step 1: ONNX Classification Testing (Today)

**Actions:**

1. [ ] Create tier2_onnx_classification_test.rs
2. [ ] Create tier2_inference_bench.rs
3. [ ] Create tier2_resource_test.rs
4. [ ] Run classification end-to-end tests
5. [ ] Run performance benchmarks
6. [ ] Measure resource utilization
7. [ ] Document results

**Expected Duration:** 2-3 hours

### Step 2: Model Download (Today)

**Actions:**

1. [ ] Download minilm-embedder.onnx (~80 MB)
2. [ ] Verify model file integrity
3. [ ] Update model manifest

**Expected Duration:** 30 minutes

### Step 3: ONNX Embedding Testing (Today)

**Actions:**

1. [ ] Create tier2_onnx_embedding_test.rs
2. [ ] Run embedding end-to-end tests
3. [ ] Run performance benchmarks
4. [ ] Measure resource utilization
5. [ ] Document results

**Expected Duration:** 2-3 hours

### Step 4: GGUF Build Resolution (This Week)

**Actions:**

1. [ ] Resolve bindgen DLL compatibility issue
2. [ ] Rebuild GGUF backend
3. [ ] Verify GGUF backend compilation
4. [ ] Run GGUF unit tests

**Expected Duration:** 2-4 hours

### Step 5: Model Download (This Week)

**Actions:**

1. [ ] Download phi3-mini-q4km.gguf (~2.2 GB)
2. [ ] Download smollm-360m-q8.gguf (~400 MB)
3. [ ] Verify model file integrity
4. [ ] Update model manifest

**Expected Duration:** 1-2 hours (depending on network)

### Step 6: GGUF Generation Testing (Next Week)

**Actions:**

1. [ ] Create tier2_gguf_generation_test.rs
2. [ ] Run generation end-to-end tests
3. [ ] Run performance benchmarks
4. [ ] Measure resource utilization
5. [ ] Document results

**Expected Duration:** 4-6 hours

---

## Success Metrics

### Phase 1 Success (ONNX Classification)

- ✅ All tests pass
- ✅ Classification P95 latency <20 ms
- ✅ Memory ratio <1.35
- ✅ Total latency <100 ms
- ✅ Documentation complete

### Phase 2 Success (ONNX Embedding)

- ✅ All tests pass
- ✅ Embedding P95 latency <10 ms
- ✅ Memory ratio <1.35
- ✅ Total latency <100 ms
- ✅ Documentation complete

### Phase 3 Success (GGUF Generation)

- ✅ All tests pass
- ✅ Generation throughput >25 tokens/sec
- ✅ Memory ratio <1.35
- ✅ Total latency <100 ms
- ✅ Documentation complete

### Overall Tier 2 Success

- ✅ All backends tested
- ✅ All performance targets met
- ✅ Resource utilization validated
- ✅ Competitive comparison documented
- ✅ Ready for production deployment

---

## Risk Mitigation

### Risk 1: GGUF Build Failure

**Mitigation:**

- Use alternative GGUF crate if bindgen issue persists
- Focus on ONNX backend for initial validation
- Document GGUF as optional backend until resolved

### Risk 2: Model Download Failures

**Mitigation:**

- Use multiple download sources
- Implement retry logic
- Validate checksums after download
- Use smaller test models if large models unavailable

### Risk 3: Performance Targets Not Met

**Mitigation:**

- Profile bottlenecks
- Optimize inference code
- Adjust performance targets based on realistic capabilities
- Document limitations and workarounds

---

## Deliverables

### Documentation

1. **TIER2_TESTING_PLAN.md** - This document
2. **TIER2_ONNX_RESULTS.md** - ONNX backend test results
3. **TIER2_GGUF_RESULTS.md** - GGUF backend test results
4. **TIER2_COMPARISON.md** - Competitive comparison with Ollama
5. **TIER2_FINAL_REPORT.md** - Complete Tier 2 validation report

### Test Files

1. **tier2_onnx_classification_test.rs** - ONNX classification tests
2. **tier2_onnx_embedding_test.rs** - ONNX embedding tests
3. **tier2_gguf_generation_test.rs** - GGUF generation tests
4. **tier2_inference_bench.rs** - Performance benchmarks
5. **tier2_resource_test.rs** - Resource utilization tests

### Automation Scripts

1. **test_tier2_onnx.bat** - Run ONNX tests
2. **test_tier2_gguf.bat** - Run GGUF tests
3. **bench_tier2_full.bat** - Run all Tier 2 benchmarks

---

## Next Actions

**Immediate (Today):**

1. Create Tier 2 test files
2. Run ONNX classification tests with tinybert model
3. Document results

**Short-term (This Week):**

1. Download remaining models
2. Resolve GGUF build issue
3. Run ONNX embedding tests
4. Begin GGUF testing

**Medium-term (Next Week):**

1. Complete GGUF generation testing
2. Run competitive benchmarks against Ollama
3. Create final Tier 2 report
4. Prepare for production deployment

---

**Plan Created By:** Automated Testing System  
**Documentation Version:** 1.0  
**Last Updated:** 2026-02-15T22:20:00Z
