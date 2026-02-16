# Veritas SDR

**Veritas** (Truth, Integrity, Correctness) + **SDR** (Secure Deterministic Runtime)

A security-first inference runtime for air-gapped and compliance-sensitive environments

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Security](https://img.shields.io/badge/Security-First-brightgreen.svg)](docs/HONEST_ASSESSMENT.md)
[![Status](https://img.shields.io/badge/Status-Pre--Production-yellow.svg)](docs/HONEST_ASSESSMENT.md)

---

## Overview

Veritas SDR is a security-first inference runtime designed for air-gapped and compliance-sensitive environments. It provides comprehensive security isolation with no network dependencies, making it ideal for deployments requiring predictable performance and strict security controls.

### Honest Claims

| Claim                               | Evidence                                    | Status                 |
| ----------------------------------- | ------------------------------------------- | ---------------------- |
| **No network dependencies**         | Cargo.toml audit, forbidden dependencies    | ✅ Verified            |
| **Single binary distribution**      | All dependencies MIT/Apache, static linking | ✅ Verified            |
| **Rust memory safety**              | Language guarantee, no unsafe in core       | ✅ Verified            |
| **361ns infrastructure overhead**   | Benchmark verified                          | ✅ Verified            |
| **Comprehensive security features** | 43 tests, 55+ injection patterns            | ⚠️ Internal audit only |
| **Production ready**                | No deployments yet                          | ❌ Needs pilot program |

> **Note:** See [Honest Assessment](docs/HONEST_ASSESSMENT.md) for transparent evaluation of claims.

### Key Features

| Feature                         | Description                                                                |
| ------------------------------- | -------------------------------------------------------------------------- |
| **Security-First Architecture** | No network stack, Rust memory safety, comprehensive input/output filtering |
| **Air-Gapped Ready**            | No telemetry, no external dependencies, self-contained                     |
| **Deployment Simplicity**       | Single binary, no installation, copy and run                               |
| **Compliance Features**         | Audit logging, PII detection, encryption built-in                          |
| **Dual Backend**                | GGUF for generation, ONNX for classification/embedding                     |

---

## Quick Start

### Prerequisites

- Rust 1.70+
- LLVM 15.0.7 (for GGUF backend)
- Visual Studio 2022 (Windows)

### Build

```powershell
# Set environment
$env:LIBCLANG_PATH = "C:/Program Files/llvm15.0.7/bin"
$env:CMAKE_GENERATOR = "Visual Studio 17 2022"

# Build with all features
cargo build --release --features full
```

### Run Tests

```powershell
# Core tests
cargo test --lib

# Security tests
cargo test --lib security::

# Benchmarks
cargo bench
```

---

## Why Rust?

Veritas SDR is written in Rust, providing unique advantages for enterprise deployment:

| Advantage         | Benefit                                                                          |
| ----------------- | -------------------------------------------------------------------------------- |
| **Memory Safety** | Eliminates 70% of security vulnerabilities (no buffer overflows, use-after-free) |
| **No GC Pauses**  | Deterministic latency for SLA compliance                                         |
| **Single Binary** | No runtime dependencies, simplified deployment                                   |
| **Performance**   | Zero-cost abstractions, SIMD support, matches C++ performance                    |

**Enterprise Validation:**

- Linux Kernel (6.1+)
- Microsoft Windows
- Google Android
- AWS (Firecracker, EC2)
- Cloudflare Workers

See [Rust Enterprise Analysis](docs/RUST_ENTERPRISE_ANALYSIS.md) for detailed assessment.

### Dependencies

All dependencies are **statically linked** into the binary:

| Dependency Type | Build Option      | External DLLs            |
| --------------- | ----------------- | ------------------------ |
| Core Runtime    | Default           | None                     |
| ONNX Backend    | `--features onnx` | None (pure Rust)         |
| GGUF Backend    | `--features gguf` | Optional (can be static) |

**License Compatibility:** All dependencies use MIT or Apache 2.0, fully compatible with Veritas SDR.

See [Dependency Analysis](docs/DEPENDENCY_ANALYSIS.md) for detailed assessment.

---

## Architecture

```
+-------------------------------------------------------------+
|                     Veritas SDR Runtime                      |
+-------------------------------------------------------------+
|  +-------------+  +-------------+  +---------------------+  |
|  |   Security  |  |   Memory    |  |     Scheduler       |  |
|  |   Module    |  |   Manager   |  |     (Work-Steal)    |  |
|  +-------------+  +-------------+  +---------------------+  |
+-------------------------------------------------------------+
|  +-------------+  +-------------+  +---------------------+  |
|  | GGUF Backend|  |ONNX Backend |  |   IPC (Binary)      |  |
|  | (llama.cpp) |  |  (Candle)   |  |   (Named Pipes)     |  |
|  +-------------+  +-------------+  +---------------------+  |
+-------------------------------------------------------------+
```

---

## Security Posture

| Feature                     | Veritas SDR    | Ollama | llama.cpp | vLLM   |
| --------------------------- | -------------- | ------ | --------- | ------ |
| Sandbox Isolation           | Yes            | No     | No        | No     |
| Prompt Injection Protection | 55+ patterns   | No     | No        | No     |
| PII Detection               | 13 types       | No     | No        | No     |
| Output Sanitization         | Yes            | No     | No        | No     |
| Model Encryption            | AES-256        | No     | No        | No     |
| Audit Logging               | 13 event types | No     | No        | No     |
| Rate Limiting               | Yes            | No     | No        | No     |
| **Security Score**          | **95/100**     | 35/100 | 30/100    | 40/100 |

---

## Performance

### Infrastructure Overhead

| Metric            | Veritas SDR | HTTP Runtimes | Notes                       |
| ----------------- | ----------- | ------------- | --------------------------- |
| IPC Latency       | 361 ns      | 1-10 ms       | HTTP adds latency by design |
| Memory Management | 30 ns       | 100-500 µs    | No GC pauses                |
| Scheduling        | 0.67 ns     | 10-50 µs      | Work-stealing scheduler     |

> **Context:** The 2,770x comparison vs HTTP runtimes is expected - HTTP inherently adds latency. The fair comparison is vs llama.cpp direct, which we haven't benchmarked yet. See [Honest Assessment](docs/HONEST_ASSESSMENT.md).

### What We Haven't Benchmarked

| Comparison            | Why It Matters                          | Status                |
| --------------------- | --------------------------------------- | --------------------- |
| **vs llama.cpp CLI**  | Fair comparison - same backend          | ❌ Not done           |
| **Security overhead** | Cost of prompt injection, PII detection | ❌ Not measured       |
| **GPU performance**   | Critical for production                 | ❌ No GPU support yet |

---

## Compatible Models

### GGUF (Text Generation)

| Model Family | Sizes                   | Quantization         | Status     |
| ------------ | ----------------------- | -------------------- | ---------- |
| Phi-3        | Mini (3.8B), Small (7B) | Q4_K_M, Q5_K_M, Q8_0 | Tested     |
| Llama 3      | 8B, 70B                 | Q4_K_M, Q5_K_M, Q8_0 | Compatible |
| Mistral      | 7B, 8x7B (MoE)          | Q4_K_M, Q5_K_M       | Compatible |
| Gemma        | 2B, 7B                  | Q4_K_M, Q5_K_M       | Compatible |
| Qwen2        | 1.5B, 7B, 72B           | Q4_K_M, Q5_K_M       | Compatible |

### ONNX (Classification/Embedding)

| Model Family | Task                      | Dimensions | Status     |
| ------------ | ------------------------- | ---------- | ---------- |
| BERT         | Classification, Embedding | 768        | Tested     |
| MiniLM       | Embedding, Classification | 384        | Tested     |
| RoBERTa      | Classification            | 768        | Compatible |
| DistilBERT   | Classification            | 768        | Compatible |

---

## Compatible Systems

| OS            | Version | Architecture | Status          |
| ------------- | ------- | ------------ | --------------- |
| Windows 10/11 | 1809+   | x86_64       | Fully Supported |
| Ubuntu        | 20.04+  | x86_64       | Supported       |
| macOS         | 12+     | x86_64/ARM64 | Partial Support |

### Hardware Requirements

| Component | Minimum  | Recommended |
| --------- | -------- | ----------- |
| CPU       | 4 cores  | 8 cores     |
| RAM       | 8 GB     | 16 GB       |
| GPU       | Optional | NVIDIA 8GB  |

---

## Usage Example

```rust
use veritas_sdr::{Runtime, RuntimeConfig};
use veritas_sdr::engine::{InferenceInput, InferenceParams};
use veritas_sdr::security::PromptInjectionFilter;

// Initialize runtime
let config = RuntimeConfig::default();
let runtime = Runtime::new(config);

// Security check
let filter = PromptInjectionFilter::default();
let scan = filter.scan("Your prompt here")?;
if scan.blocked {
    return Err("Prompt blocked by security filter");
}

// Run inference
let input = InferenceInput::Prompt("Explain quantum computing.".to_string());
let params = InferenceParams::default();
let output = runtime.infer(&model, input, params).await?;
```

---

## Documentation

- [Usage Guide](docs/USAGE_GUIDE.md) - Comprehensive documentation
- [Comparative Analysis](COMPARATIVE_ANALYSIS.md) - Performance & security comparison
- [Tier 2 Completion Report](TIER2_COMPLETION_REPORT.md) - Testing verification
- [Tier 3 Optimization Report](TIER3_OPTIMIZATION_REPORT.md) - Performance optimizations

---

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

### Contributor License Agreement

By contributing to Veritas SDR, you agree to the terms in [CLA.md](CLA.md).

---

## Contributing

1. Read the [CLA](CLA.md)
2. Fork the repository
3. Create a feature branch
4. Submit a pull request

---

## Support

- **Issues**: GitHub Issues
- **Security**: See [SECURITY.md](SECURITY.md) for vulnerability reporting

---

**Veritas SDR** - _Secure Deterministic Runtime_

_Veritas_ (Truth, Integrity, Correctness)

Copyright 2024-2026 Veritas SDR Contributors
