# Veritas SDR

**Veritas** (Truth, Integrity, Correctness) + **SDR** (Secure Deterministic Runtime)

A security-first inference runtime for air-gapped and compliance-sensitive environments.

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Security](https://img.shields.io/badge/Security-Hardened-brightgreen.svg)](docs/security/THREAT_MODEL.md)
[![Tests](https://img.shields.io/badge/Tests-430%2B-blue.svg)](docs/testing/)

---

## Overview

Veritas SDR is a sandboxed, offline inference engine providing comprehensive security isolation with zero network dependencies. Designed for air-gapped deployments and compliance-sensitive environments requiring predictable performance and strict security controls.

### Key Features

| Feature | Description |
|---------|-------------|
| **Security-First** | No network stack, Rust memory safety, input/output filtering |
| **Air-Gapped Ready** | No telemetry, no external dependencies, self-contained |
| **Simple Deployment** | Single binary, no installation, copy and run |
| **Compliance Built-in** | Audit logging, PII detection, AES-256-GCM encryption |
| **Dual Backend** | GGUF for generation, ONNX for classification/embedding |

### Verified Claims

| Claim | Evidence |
|-------|----------|
| No network dependencies | Cargo.toml audit, forbidden dependency list |
| Single binary distribution | MIT/Apache dependencies, static linking |
| Rust memory safety | Language guarantee, no unsafe in core paths |
| 361ns infrastructure overhead | Benchmark verified |
| 430+ security tests | Full test suite passing |

---

## Quick Start

### Prerequisites

- Rust 1.70+
- LLVM 15.0.7 (for GGUF backend)
- Visual Studio 2022 (Windows)

### Build

```bash
# Windows environment setup
$env:LIBCLANG_PATH = "C:/Program Files/llvm15.0.7/bin"
$env:CMAKE_GENERATOR = "Visual Studio 17 2022"

# Build with all features
cargo build --release --features full
```

### Test

```bash
# Unit tests
cargo test --lib

# Security tests
cargo test --lib security::

# Benchmarks
cargo bench

# Fuzz tests (requires nightly)
cd core-runtime
cargo +nightly fuzz run fuzz_ipc_json -- -max_total_time=300
```

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
|  | GGUF Backend|  |ONNX Backend |  |   IPC Protocol      |  |
|  | (llama.cpp) |  |  (Candle)   |  |   (Named Pipes)     |  |
|  +-------------+  +-------------+  +---------------------+  |
+-------------------------------------------------------------+
```

---

## Security

| Feature | Implementation |
|---------|----------------|
| Sandbox Isolation | Process-level, seccomp/AppContainer |
| Prompt Injection Protection | 55+ patterns, Aho-Corasick matching |
| PII Detection | 13 types with redaction |
| Output Sanitization | Format validation, content filtering |
| Model Encryption | AES-256-GCM, PBKDF2 key derivation |
| Audit Logging | 13 event types, SIEM-compatible |
| Authentication | Constant-time comparison, rate limiting |

See [Threat Model](docs/security/THREAT_MODEL.md) for detailed security analysis.

---

## Performance

| Metric | Veritas SDR | HTTP Runtimes |
|--------|-------------|---------------|
| IPC Latency | 361 ns | 1-10 ms |
| Memory Management | 30 ns | 100-500 us |
| Scheduling | 0.67 ns | 10-50 us |

Performance validated against [tier targets](docs/CONCEPT.md#tier-progression-targets).

---

## Compatible Models

### GGUF (Text Generation)

| Model | Sizes | Quantization |
|-------|-------|--------------|
| Phi-3 | 3.8B, 7B | Q4_K_M, Q5_K_M, Q8_0 |
| Llama 3 | 8B, 70B | Q4_K_M, Q5_K_M, Q8_0 |
| Mistral | 7B, 8x7B | Q4_K_M, Q5_K_M |

### ONNX (Classification/Embedding)

| Model | Task | Dimensions |
|-------|------|------------|
| BERT | Classification, Embedding | 768 |
| MiniLM | Embedding, Classification | 384 |

---

## System Requirements

| Component | Minimum | Recommended |
|-----------|---------|-------------|
| CPU | 4 cores | 8 cores |
| RAM | 8 GB | 16 GB |
| GPU | Optional | NVIDIA 8GB |
| OS | Windows 10+, Ubuntu 20.04+, macOS 12+ | - |

---

## Usage

```rust
use veritas_sdr::{Runtime, RuntimeConfig};
use veritas_sdr::engine::{InferenceInput, InferenceParams};
use veritas_sdr::security::PromptInjectionFilter;

// Initialize runtime
let config = RuntimeConfig::default();
let runtime = Runtime::new(config);

// Security scan
let filter = PromptInjectionFilter::default();
let (is_safe, risk_score, _) = filter.scan("Your prompt here");
if !is_safe {
    return Err("Prompt blocked by security filter");
}

// Run inference
let input = InferenceInput::Prompt("Explain quantum computing.".to_string());
let params = InferenceParams::default();
let output = runtime.infer(&model, input, params).await?;
```

See [Usage Guide](docs/USAGE_GUIDE.md) for complete API documentation.

---

## Documentation

### Core

| Document | Description |
|----------|-------------|
| [Usage Guide](docs/USAGE_GUIDE.md) | API reference and usage patterns |
| [Concept](docs/CONCEPT.md) | Design philosophy and constraints |
| [Dependency Analysis](docs/DEPENDENCY_ANALYSIS.md) | Dependency audit and licensing |

### Security

| Document | Description |
|----------|-------------|
| [Threat Model](docs/security/THREAT_MODEL.md) | STRIDE analysis and attack trees |
| [Security Analysis](docs/security/SECURITY_ANALYSIS_REPORT.md) | Vulnerability remediations |

### Testing

| Document | Description |
|----------|-------------|
| [Tier 2 Report](docs/testing/TIER2_COMPLETION_REPORT.md) | Competitive performance validation |
| [Tier 3 Report](docs/testing/TIER3_OPTIMIZATION_REPORT.md) | Advanced optimization results |

### Build

| Document | Description |
|----------|-------------|
| [GGUF Build Guide](docs/build/GGUF_BUILD_TROUBLESHOOTING.md) | Backend build instructions |

---

## Project Status

See [ROADMAP.md](ROADMAP.md) for development status and planned features.

---

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

---

## Contributing

1. Read the [CLA](CLA.md)
2. Fork the repository
3. Create a feature branch
4. Submit a pull request

---

## Security

See [SECURITY.md](SECURITY.md) for vulnerability reporting.

---

Copyright 2024-2026 Veritas SDR Contributors
