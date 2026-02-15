# Tier 1 Full Validation - Setup Guide

**Status:** 🚧 SETUP REQUIRED  
**Date:** 2026-02-14  
**Purpose:** Complete Tier 1 validation with actual model inference backends

---

## Current Status

### ✅ Completed

- Tier 1 infrastructure benchmarks (BASELINE_METRICS.md)
- All 6 benchmark files fixed and executed
- Infrastructure overhead confirmed <20 µs (<0.02% of 100ms target)

### 🚧 Pending - Build Dependencies

Both model backends require additional build tools on Windows:

| Backend | Required Tool | Purpose                                    | Status           |
| ------- | ------------- | ------------------------------------------ | ---------------- |
| ONNX    | `protoc`      | Protocol Buffers compiler for candle-onnx  | ❌ Not installed |
| GGUF    | `libclang`    | LLVM library for bindgen (llama-cpp-sys-2) | ❌ Not installed |

---

## Build Environment Setup

### Option 1: Install via Package Manager (Recommended)

#### Using Chocolatey (if installed)

```powershell
# Install protoc (protobuf compiler)
choco install protobuf

# Install LLVM (includes libclang)
choco install llvm
```

#### Using Scoop (if installed)

```powershell
# Install protoc
scoop install protobuf

# Install LLVM
scoop install llvm
```

### Option 2: Manual Installation

#### Installing protoc (Protocol Buffers Compiler)

1. Download from: https://github.com/protocolbuffers/protobuf/releases
2. Choose the Windows pre-built binary (e.g., `protoc-xx.x-win64.zip`)
3. Extract to a directory (e.g., `C:\tools\protobuf`)
4. Add to PATH: `C:\tools\protobuf\bin`
5. Verify installation:
   ```powershell
   protoc --version
   ```

#### Installing LLVM (for libclang)

1. Download from: https://releases.llvm.org/download.html
2. Choose the Windows pre-built binary (e.g., `LLVM-xx.x.x-win64.exe`)
3. Run installer and add to PATH
4. Verify installation:
   ```powershell
   clang --version
   ```

### Option 3: Set Environment Variables

If you have these tools installed but Cargo can't find them:

```powershell
# Set LIBCLANG_PATH (for GGUF backend)
$env:LIBCLANG_PATH = "C:\Program Files\LLVM\bin"

# Set PROTOC path (for ONNX backend)
$env:PATH += ";C:\tools\protobuf\bin"

# For persistent changes, add to System Environment Variables
```

---

## Test Models Acquisition

### Required Models

| Model                    | Type            | Format | Purpose                 | Size    |
| ------------------------ | --------------- | ------ | ----------------------- | ------- |
| phi3-mini-q4km.gguf      | Text Generation | GGUF   | GGUF backend testing    | ~2.3 GB |
| tinybert-classifier.onnx | Classification  | ONNX   | ONNX classifier testing | ~100 MB |
| minilm-embedder.onnx     | Embedding       | ONNX   | ONNX embedder testing   | ~120 MB |

### Model Sources

#### 1. phi3-mini-q4km.gguf

- **Source:** Hugging Face - Microsoft Phi-3
- **Download URL:** https://huggingface.co/microsoft/Phi-3-mini-4k-instruct-gguf/resolve/main/Phi-3-mini-4k-instruct-q4_k_m.gguf
- **Alternative:** Use Ollama to pull: `ollama pull phi3`
- **Location:** Ollama models are typically in `~/.ollama/models`

#### 2. tinybert-classifier.onnx

- **Source:** Hugging Face - TinyBERT models
- **Search:** https://huggingface.co/models?search=tinybert+onnx
- **Example:** `prajjwal1/bert-tiny` (convert to ONNX if needed)
- **Note:** May need to convert from PyTorch to ONNX using `torch.onnx.export()`

#### 3. minilm-embedder.onnx

- **Source:** Hugging Face - MiniLM models
- **Search:** https://huggingface.co/models?search=minilm+onnx
- **Example:** `sentence-transformers/all-MiniLM-L6-v2`
- **Note:** May need to convert from PyTorch to ONNX

### Model Directory Structure

Create the following structure in `core-runtime/fixtures/`:

```
core-runtime/fixtures/
├── models/
│   ├── gguf/
│   │   └── phi3-mini-q4km.gguf
│   └── onnx/
│       ├── tinybert-classifier.onnx
│       └── minilm-embedder.onnx
├── baselines/
│   └── baseline_metrics.json
└── prompts/
    ├── small.json
    ├── medium.json
    └── large.json
```

### Model Download Script

Create `core-runtime/scripts/download_models.ps1`:

```powershell
# Create directories
New-Item -ItemType Directory -Force -Path "fixtures/models/gguf"
New-Item -ItemType Directory -Force -Path "fixtures/models/onnx"

# Download phi3-mini-q4km.gguf
Write-Host "Downloading phi3-mini-q4km.gguf..."
Invoke-WebRequest -Uri "https://huggingface.co/microsoft/Phi-3-mini-4k-instruct-gguf/resolve/main/Phi-3-mini-4k-instruct-q4_k_m.gguf" -OutFile "fixtures/models/gguf/phi3-mini-q4km.gguf"

# Note: ONNX models may need manual acquisition or conversion
Write-Host "Please download ONNX models manually or convert from PyTorch"
Write-Host "See TIER1_SETUP_GUIDE.md for details"
```

---

## Build and Validation Steps

### Step 1: Install Build Dependencies

Choose one of the installation methods above and verify:

```powershell
# Verify protoc
protoc --version
# Expected: libprotoc x.x.x

# Verify clang/libclang
clang --version
# Expected: clang version x.x.x
```

### Step 2: Create Model Directories

```powershell
cd core-runtime
mkdir -p fixtures/models/gguf
mkdir -p fixtures/models/onnx
```

### Step 3: Download Test Models

Run the download script or manually download models to the appropriate directories.

### Step 4: Build with Backend Features

```powershell
cd core-runtime

# Build with both backends
cargo build --features onnx,gguf

# Or build individually for testing
cargo build --features gguf
cargo build --features onnx
```

### Step 5: Run Integration Tests

```powershell
# Run GGUF integration tests
cargo test --features gguf integration_gguf

# Run ONNX integration tests
cargo test --features onnx integration_onnx

# Run all integration tests
cargo test --features onnx,gguf integration
```

### Step 6: Run End-to-End Benchmarks

```powershell
# Run benchmarks with model backends
cargo bench --features onnx,gguf

# Or run specific benchmarks
cargo bench --features gguf generation_throughput
cargo bench --features onnx inference_latency
```

### Step 7: Update BASELINE_METRICS.md

Document the end-to-end inference results, including:

- Model loading times
- Inference latency per model
- Total end-to-end latency
- Memory usage per model
- Comparison with infrastructure-only benchmarks

---

## Troubleshooting

### Issue: "couldn't find any valid shared libraries matching: ['clang.dll', 'libclang.dll']"

**Solution:** Set the LIBCLANG_PATH environment variable:

```powershell
$env:LIBCLANG_PATH = "C:\Program Files\LLVM\bin"
```

### Issue: "Could not find `protoc`"

**Solution:** Install protoc and add to PATH, or set PROTOC environment variable:

```powershell
$env:PROTOC = "C:\tools\protobuf\bin\protoc.exe"
```

### Issue: "Access is denied" when cleaning cargo cache

**Solution:** This is a warning and can be ignored. The build should still complete.

### Issue: Build takes too long

**Solution:** The first build with native dependencies can take 10-30 minutes. Subsequent builds will be faster due to cargo caching.

---

## Alternative: Pre-built Binaries

If build dependencies are difficult to install, consider:

1. **Use pre-built llama.cpp binaries** for GGUF testing
2. **Use ONNX Runtime** directly for ONNX model testing
3. **Test with mock models** to validate infrastructure without actual inference

---

## Next Steps

Once build environment is set up:

1. ✅ Install protoc and libclang
2. ✅ Create model directory structure
3. ✅ Download/acquire test models
4. ✅ Build with `--features onnx,gguf`
5. ✅ Run integration tests
6. ✅ Run end-to-end benchmarks
7. ✅ Update BASELINE_METRICS.md with full results
8. ✅ Generate Tier 1 validation report

---

## Quick Reference Commands

```powershell
# Install dependencies (Chocolatey)
choco install protobuf llvm

# Set environment variables
$env:LIBCLANG_PATH = "C:\Program Files\LLVM\bin"
$env:PATH += ";C:\Program Files\LLVM\bin"

# Verify installations
protoc --version
clang --version

# Build project
cd core-runtime
cargo build --features onnx,gguf

# Run tests
cargo test --features onnx,gguf

# Run benchmarks
cargo bench --features onnx,gguf
```

---

## Contact & Support

If you encounter issues:

1. Check the error messages carefully
2. Verify all dependencies are installed correctly
3. Ensure environment variables are set
4. Try building each backend separately first

**Document Version:** 1.0  
**Last Updated:** 2026-02-14T19:55:00Z
