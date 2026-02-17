# Tier 1 Full Validation - Execution Plan

**Status:** 🚧 IN PROGRESS  
**Date:** 2026-02-14  
**Objective:** Complete Tier 1 validation with actual model inference backends

---

## Executive Summary

Tier 1 infrastructure benchmarks have been completed successfully, confirming that the runtime infrastructure overhead is negligible (<20 µs, <0.02% of 100ms target). The next phase requires setting up the model inference backends (ONNX and GGUF) to run end-to-end tests with actual models.

### Current Status

| Phase                     | Status      | Completion |
| ------------------------- | ----------- | ---------- |
| Infrastructure Benchmarks | ✅ Complete | 100%       |
| Build Environment Setup   | 🚧 Pending  | 0%         |
| Model Acquisition         | 🚧 Pending  | 0%         |
| End-to-End Tests          | 🚧 Pending  | 0%         |
| Final Report              | 🚧 Pending  | 0%         |

---

## Completed Work

### 1. Infrastructure Benchmarks ✅

**File:** [`BASELINE_METRICS.md`](BASELINE_METRICS.md:1)

All 6 criterion benchmarks executed successfully:

| Component        | Performance     | Status        |
| ---------------- | --------------- | ------------- |
| IPC encode       | 104-135 Melem/s | ✅ EXCELLENT  |
| IPC decode       | 23.6 Melem/s    | ✅ GOOD       |
| Scheduler ops    | 2-5 Melem/s     | ✅ EXCELLENT  |
| Input validation | 2.9-4.3 ns      | ✅ NEGLIGIBLE |
| Memory acquire   | 1.05 µs         | ✅ GOOD       |
| Result creation  | 85-113 ns       | ✅ NEGLIGIBLE |

**Key Finding:** Total infrastructure overhead <20 µs, leaving 99.98% of latency budget for model inference.

### 2. Setup Documentation ✅

**File:** [`TIER1_SETUP_GUIDE.md`](TIER1_SETUP_GUIDE.md:1)

Comprehensive guide covering:

- Build dependency installation (protoc, libclang)
- Model acquisition instructions
- Directory structure setup
- Build and test procedures
- Troubleshooting guide

### 3. Directory Structure ✅

Created model directories:

```
core-runtime/fixtures/
├── models/
│   ├── gguf/      # For GGUF models
│   └── onnx/      # For ONNX models
├── baselines/     # Baseline metrics
└── prompts/       # Test prompts
```

### 4. Download Script ✅

**File:** [`core-runtime/scripts/download_models.ps1`](core-runtime/scripts/download_models.ps1:1)

PowerShell script that:

- Creates required directories
- Downloads phi3-mini-q4km.gguf automatically
- Provides instructions for ONNX models
- Verifies model presence

---

## Pending Work

### Phase 1: Build Environment Setup

#### Required Dependencies

| Tool     | Purpose                  | Installation            | Status           |
| -------- | ------------------------ | ----------------------- | ---------------- |
| protoc   | ONNX backend compilation | Chocolatey/Scoop/Manual | ❌ Not installed |
| libclang | GGUF backend binding     | Chocolatey/Scoop/Manual | ❌ Not installed |

#### Installation Options

**Option A: Chocolatey (Recommended)**

```powershell
choco install protobuf llvm
```

**Option B: Scoop**

```powershell
scoop install protobuf llvm
```

**Option C: Manual**

- Download protoc from https://github.com/protocolbuffers/protobuf/releases
- Download LLVM from https://releases.llvm.org/download.html
- Add to PATH and set environment variables

#### Verification Commands

```powershell
protoc --version    # Expected: libprotoc x.x.x
clang --version     # Expected: clang version x.x.x
```

**Estimated Time:** 15-30 minutes

---

### Phase 2: Model Acquisition

#### Required Models

| Model                    | Type            | Format | Size    | Source       |
| ------------------------ | --------------- | ------ | ------- | ------------ |
| phi3-mini-q4km.gguf      | Text Generation | GGUF   | ~2.3 GB | Hugging Face |
| tinybert-classifier.onnx | Classification  | ONNX   | ~100 MB | Hugging Face |
| minilm-embedder.onnx     | Embedding       | ONNX   | ~120 MB | Hugging Face |

#### Download Process

**Automated (GGUF only):**

```powershell
cd core-runtime
.\scripts\download_models.ps1
```

**Manual (ONNX models):**

1. Visit Hugging Face model hub
2. Search for "tinybert onnx" and "minilm onnx"
3. Download or convert from PyTorch
4. Place in `fixtures/models/onnx/`

**Estimated Time:** 10-30 minutes (depends on internet speed)

---

### Phase 3: Build and Test

#### Build Commands

```powershell
cd core-runtime

# Build with both backends
cargo build --features onnx,gguf

# Build individually for testing
cargo build --features gguf
cargo build --features onnx
```

**Estimated Time:** 20-40 minutes (first build with native deps)

#### Test Commands

```powershell
# Run GGUF integration tests
cargo test --features gguf integration_gguf

# Run ONNX integration tests
cargo test --features onnx integration_onnx

# Run all integration tests
cargo test --features onnx,gguf integration
```

**Estimated Time:** 5-15 minutes

#### Benchmark Commands

```powershell
# Run all benchmarks with model backends
cargo bench --features onnx,gguf

# Run specific benchmarks
cargo bench --features gguf generation_throughput
cargo bench --features onnx inference_latency
```

**Estimated Time:** 10-20 minutes

---

### Phase 4: Final Reporting

Update [`BASELINE_METRICS.md`](BASELINE_METRICS.md:1) with:

1. Model loading times
2. Inference latency per model
3. Total end-to-end latency
4. Memory usage per model
5. Comparison with infrastructure-only benchmarks
6. Tier 1 validation checklist status

---

## Detailed Execution Steps

### Step 1: Install Build Dependencies (15-30 min)

```powershell
# Check if Chocolatey is installed
choco --version

# If not installed, run (as Administrator):
# Set-ExecutionPolicy Bypass -Scope Process -Force; [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))

# Install dependencies
choco install protobuf llvm

# Verify installations
protoc --version
clang --version
```

### Step 2: Download Test Models (10-30 min)

```powershell
cd g:/MythologIQ/CORE/core-runtime

# Run download script
.\scripts\download_models.ps1

# Manually download ONNX models if needed
# Place in fixtures/models/onnx/
```

### Step 3: Build Project (20-40 min)

```powershell
cd g:/MythologIQ/CORE/core-runtime

# Set environment variables if needed
$env:LIBCLANG_PATH = "C:\Program Files\LLVM\bin"

# Build with both backends
cargo build --features onnx,gguf
```

### Step 4: Run Integration Tests (5-15 min)

```powershell
cd g:/MythologIQ/CORE/core-runtime

# Run all integration tests
cargo test --features onnx,gguf integration
```

### Step 5: Run Benchmarks (10-20 min)

```powershell
cd g:/MythologIQ/CORE/core-runtime

# Run all benchmarks
cargo bench --features onnx,gguf
```

### Step 6: Generate Final Report (5-10 min)

```powershell
# Update BASELINE_METRICS.md with results
# Include:
# - Model loading times
# - Inference latency
# - End-to-end latency
# - Memory usage
# - Final validation status
```

---

## Troubleshooting

### Common Issues

| Issue                                                                              | Solution                                                                   |
| ---------------------------------------------------------------------------------- | -------------------------------------------------------------------------- |
| "couldn't find any valid shared libraries matching: ['clang.dll', 'libclang.dll']" | Set `$env:LIBCLANG_PATH = "C:\Program Files\LLVM\bin"`                     |
| "Could not find `protoc`"                                                          | Install protoc or set `$env:PROTOC = "path\to\protoc.exe"`                 |
| Build takes too long                                                               | First build with native deps takes 20-40 min; subsequent builds are faster |
| "Access is denied" when cleaning cache                                             | This is a warning and can be ignored                                       |

### Getting Help

1. Check [`TIER1_SETUP_GUIDE.md`](TIER1_SETUP_GUIDE.md:1) for detailed instructions
2. Review error messages carefully
3. Verify all dependencies are installed
4. Try building each backend separately

---

## Success Criteria

Tier 1 validation is complete when:

- ✅ All infrastructure benchmarks pass (DONE)
- ✅ Build with `--features onnx,gguf` succeeds
- ✅ All integration tests pass
- ✅ End-to-end benchmarks execute
- ✅ Total latency (infrastructure + inference) < 100ms
- ✅ BASELINE_METRICS.md updated with full results
- ✅ Tier 1 validation checklist complete

---

## Estimated Total Time

| Phase                   | Time      | Cumulative |
| ----------------------- | --------- | ---------- |
| Build Environment Setup | 15-30 min | 15-30 min  |
| Model Acquisition       | 10-30 min | 25-60 min  |
| Build Project           | 20-40 min | 45-100 min |
| Run Tests               | 5-15 min  | 50-115 min |
| Run Benchmarks          | 10-20 min | 60-135 min |
| Final Report            | 5-10 min  | 65-145 min |

**Total Estimated Time:** 1-2.5 hours

---

## Quick Reference

### All Commands in Sequence

```powershell
# 1. Install dependencies
choco install protobuf llvm

# 2. Verify installations
protoc --version
clang --version

# 3. Download models
cd g:/MythologIQ/CORE/core-runtime
.\scripts\download_models.ps1

# 4. Build project
cargo build --features onnx,gguf

# 5. Run tests
cargo test --features onnx,gguf integration

# 6. Run benchmarks
cargo bench --features onnx,gguf

# 7. Update report
# Edit BASELINE_METRICS.md with results
```

---

## Next Actions

1. **Immediate:** Install build dependencies (protoc, libclang)
2. **Then:** Download test models using the provided script
3. **Then:** Build project with backend features
4. **Then:** Run integration tests and benchmarks
5. **Finally:** Update BASELINE_METRICS.md with complete results

---

## Documents Created

1. **[`BASELINE_METRICS.md`](BASELINE_METRICS.md:1)** - Infrastructure benchmark results
2. **[`TIER1_SETUP_GUIDE.md`](TIER1_SETUP_GUIDE.md:1)** - Detailed setup instructions
3. **[`TIER1_EXECUTION_PLAN.md`](TIER1_EXECUTION_PLAN.md:1)** - This execution plan
4. **[`core-runtime/scripts/download_models.ps1`](core-runtime/scripts/download_models.ps1:1)** - Model download script

---

**Document Version:** 1.0  
**Last Updated:** 2026-02-14T19:59:00Z  
**Status:** Ready for execution (pending build dependencies)
