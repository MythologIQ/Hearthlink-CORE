# Dependency Analysis: GG-CORE

**Document Purpose:** Analyze dependencies, build options, and licensing implications
**Date:** 2026-02-16
**Status:** Complete

---

## Executive Summary

GG-CORE (Greatest Good - Contained Offline Restricted Execution) has **two types of dependencies**:

1. **Rust Crates** - Compiled into the binary (static linking)
2. **Native Libraries** - Optional backends (ONNX, GGUF)

### Key Findings

| Factor                    | Assessment                                        |
| ------------------------- | ------------------------------------------------- |
| **Static Linking**        | All Rust crates compile into single binary        |
| **Native Libraries**      | Optional, can be statically or dynamically linked |
| **License Compatibility** | All dependencies are Apache 2.0 or MIT compatible |
| **Distribution**          | Single binary possible for core runtime           |

---

## 1. Dependency Categories

### 1.1 Core Dependencies (Always Included)

These dependencies are **compiled into the binary** and become part of the executable:

| Dependency       | Purpose                | License        | Build Type |
| ---------------- | ---------------------- | -------------- | ---------- |
| **tokio**        | Async runtime          | MIT            | Static     |
| **serde**        | Serialization          | MIT/Apache-2.0 | Static     |
| **serde_json**   | JSON parsing           | MIT/Apache-2.0 | Static     |
| **bincode**      | Binary serialization   | MIT            | Static     |
| **interprocess** | IPC (named pipes)      | MIT            | Static     |
| **thiserror**    | Error handling         | MIT/Apache-2.0 | Static     |
| **async-trait**  | Async traits           | MIT/Apache-2.0 | Static     |
| **sha2**         | SHA-256 hashing        | MIT/Apache-2.0 | Static     |
| **hex**          | Hex encoding           | MIT/Apache-2.0 | Static     |
| **rand**         | CSPRNG                 | MIT/Apache-2.0 | Static     |
| **regex**        | Pattern matching       | MIT/Apache-2.0 | Static     |
| **aho-corasick** | Multi-pattern matching | MIT/Apache-2.0 | Static     |
| **aes**          | AES encryption         | MIT/Apache-2.0 | Static     |
| **toml**         | Config parsing         | MIT/Apache-2.0 | Static     |
| **memmap2**      | Memory-mapped files    | MIT            | Static     |
| **tracing**      | Structured logging     | MIT            | Static     |
| **metrics**      | Metrics facade         | MIT            | Static     |
| **parking_lot**  | Synchronization        | MIT/Apache-2.0 | Static     |
| **dashmap**      | Concurrent map         | MIT            | Static     |
| **num_cpus**     | CPU detection          | MIT/Apache-2.0 | Static     |
| **hostname**     | Machine ID             | MIT/Apache-2.0 | Static     |
| **tempfile**     | Temp files             | MIT/Apache-2.0 | Static     |
| **windows-sys**  | Windows API            | MIT/Apache-2.0 | Static     |

**Result:** All core dependencies become part of the binary - no external DLLs required.

### 1.2 Optional Backend Dependencies

These are **feature-gated** and only included when needed:

| Dependency      | Purpose        | License        | Native Code | Static Option |
| --------------- | -------------- | -------------- | ----------- | ------------- |
| **candle-core** | ML tensors     | MIT/Apache-2.0 | No          | Pure Rust     |
| **candle-onnx** | ONNX inference | MIT/Apache-2.0 | No          | Pure Rust     |
| **llama-cpp-2** | GGUF inference | MIT            | Yes (C++)   | Can be static |

---

## 2. Build Options

### 2.1 Option A: Pure Rust Build (Recommended)

```powershell
# Build with ONNX backend (pure Rust)
cargo build --release --features onnx

# Result: Single binary, no native dependencies
```

| Component    | Included | Native Dependencies        |
| ------------ | -------- | -------------------------- |
| Core Runtime | Yes      | None                       |
| ONNX Backend | Yes      | None (Candle is pure Rust) |
| GGUF Backend | No       | N/A                        |

**Advantages:**

- Single binary distribution
- No DLL/SO dependencies
- Cross-compilation friendly
- Smaller attack surface

### 2.2 Option B: Full Build with GGUF

```powershell
# Build with all backends
cargo build --release --features full

# Result: Binary + optional native library for GGUF
```

| Component    | Included | Native Dependencies |
| ------------ | -------- | ------------------- |
| Core Runtime | Yes      | None                |
| ONNX Backend | Yes      | None                |
| GGUF Backend | Yes      | llama.cpp (C++)     |

**GGUF Backend Options:**

| Option          | Build Flag       | Result                           |
| --------------- | ---------------- | -------------------------------- |
| Dynamic Linking | Default          | Requires llama.dll / libllama.so |
| Static Linking  | `LLAMA_STATIC=1` | Embedded in binary               |

**Static linking for GGUF:**

```powershell
$env:LLAMA_STATIC = "1"
cargo build --release --features gguf
# Result: Single binary with embedded llama.cpp
```

### 2.3 Option C: Core Only (No Backends)

```powershell
# Build runtime only
cargo build --release

# Result: Smallest binary, no inference backends
```

| Component    | Included | Native Dependencies |
| ------------ | -------- | ------------------- |
| Core Runtime | Yes      | None                |
| ONNX Backend | No       | N/A                 |
| GGUF Backend | No       | N/A                 |

**Use Case:** For deployments that use external model servers.

---

## 3. Licensing Implications

### 3.1 License Compatibility Matrix

| Dependency License | Compatible with Apache 2.0 | Notes                          |
| ------------------ | -------------------------- | ------------------------------ |
| Apache-2.0         | ✅ Yes                     | Same license, fully compatible |
| MIT                | ✅ Yes                     | Permissive, compatible         |
| BSD-3-Clause       | ✅ Yes                     | Permissive, compatible         |
| MPL-2.0            | ✅ Yes                     | Weak copyleft, file-scope      |
| LGPL               | ⚠️ Conditional             | Dynamic linking recommended    |
| GPL                | ❌ No                      | Incompatible with Apache 2.0   |

### 3.2 GG-CORE Dependency Licenses - ALL PERMISSIVE

**IMPORTANT: GG-CORE has NO LGPL or GPL dependencies.**

All dependencies in this project use permissive licenses:

| Category                     | Licenses        | Compatibility    |
| ---------------------------- | --------------- | ---------------- |
| **Core Dependencies**        | MIT, Apache-2.0 | Fully compatible |
| **ONNX Backend (Candle)**    | MIT, Apache-2.0 | Fully compatible |
| **GGUF Backend (llama.cpp)** | MIT             | Fully compatible |

**Result:** All dependencies are Apache 2.0 or MIT, fully compatible with GG-CORE's Apache 2.0 license.

### 3.3 Why LGPL/GPL Are Listed (But Not Used)

The license compatibility matrix includes LGPL and GPL for **reference only** - these show what *would* happen if such dependencies were added:

| License | Why It's Listed | Current Status in GG-CORE |
| ------- | --------------- | ----------------------------- |
| **LGPL** | Shows copyleft implications | ❌ **NOT USED** - No LGPL dependencies |
| **GPL** | Shows incompatibility warning | ❌ **NOT USED** - No GPL dependencies |

**Why this matters:**

- **LGPL (Lesser GPL)**: If used, would require dynamic linking or provide object files for relinking. This adds deployment complexity.
- **GPL**: If used, would require the entire project to be GPL-licensed, which conflicts with Apache 2.0.

**GG-CORE's Policy:** Only MIT and Apache-2.0 licensed dependencies are permitted. This ensures:
1. No copyleft obligations
2. No license compatibility issues
3. Maximum flexibility for users and contributors
4. Single binary distribution without restrictions

### 3.3 Static vs Dynamic Linking Implications

| Linking Type             | License Impact               | Distribution  |
| ------------------------ | ---------------------------- | ------------- |
| **Static (Rust crates)** | No impact - MIT/Apache allow | Single binary |
| **Static (native libs)** | MIT allows, Apache allows    | Single binary |
| **Dynamic**              | No impact for MIT/Apache     | Binary + DLLs |

**Key Point:** MIT and Apache 2.0 licenses allow static linking without imposing additional requirements on the final binary.

### 3.4 Attribution Requirements

| License    | Requirement                            | Compliance              |
| ---------- | -------------------------------------- | ----------------------- |
| Apache-2.0 | Include license text, copyright notice | Include in LICENSE file |
| MIT        | Include copyright notice, license text | Include in NOTICE file  |

**Recommended:** Create a `NOTICE` file with all dependency attributions:

```
GG-CORE
Copyright 2024-2026 GG-CORE Contributors

This product includes software developed by third parties:

tokio - Copyright (c) Tokio Contributors
  Licensed under MIT

serde - Copyright (c) Serde Contributors
  Licensed under MIT OR Apache-2.0

[... full list of dependencies ...]
```

---

## 4. Distribution Options

### 4.1 Single Binary Distribution (Recommended)

```
GG-CORE.exe          # Windows
GG-CORE              # Linux
GG-CORE              # macOS
```

**Requirements:**

- Build with `--features onnx` (pure Rust)
- Or build with `LLAMA_STATIC=1` for GGUF

**Advantages:**

- No installation required
- No DLL hell
- Simple deployment
- Container-friendly

### 4.2 Binary + Native Libraries

```
GG-CORE.exe
llama.dll                # GGUF backend
```

**Requirements:**

- Build with `--features full` (default dynamic linking)

**Advantages:**

- Smaller binary size
- Shared library updates

### 4.3 Container Distribution

```dockerfile
FROM scratch
COPY GG-CORE /GG-CORE
ENTRYPOINT ["/GG-CORE"]
```

**Result:** Minimal container (~20-50 MB) with single binary.

---

## 5. Dependency Audit

### 5.1 Security Audit

| Check                | Status   | Notes                         |
| -------------------- | -------- | ----------------------------- |
| `cargo audit`        | Pass     | No known vulnerabilities      |
| Network dependencies | None     | No reqwest, hyper, etc.       |
| Unsafe code          | Minimal  | Only in native backends       |
| Supply chain         | Verified | Hash verification implemented |

### 5.2 Forbidden Dependencies

The following dependencies are **explicitly forbidden** per [`Cargo.toml`](core-runtime/Cargo.toml:124):

| Dependency    | Reason               |
| ------------- | -------------------- |
| `reqwest`     | Network access       |
| `hyper`       | HTTP server          |
| `tungstenite` | WebSocket            |
| `walkdir`     | Filesystem traversal |

**Security Implication:** No network capabilities can be accidentally introduced.

---

## 6. Recommendations

### 6.1 Single Binary Distribution - RECOMMENDED

**Yes, single binary distribution is recommended for GG-CORE.**

| Factor         | Assessment                                                  |
| -------------- | ----------------------------------------------------------- |
| **Security**   | Reduced attack surface (no DLL search path vulnerabilities) |
| **Deployment** | Copy and run - no installation required                     |
| **Operations** | No DLL version conflicts ("DLL hell")                       |
| **Containers** | Minimal image size (FROM scratch possible)                  |
| **Air-gapped** | Self-contained, no external dependencies                    |
| **Enterprise** | Simplified change management and auditing                   |

**Advantages:**

| Advantage                   | Impact                                 |
| --------------------------- | -------------------------------------- |
| **No Runtime Installation** | Reduces deployment complexity          |
| **No Dependency Conflicts** | Eliminates version mismatch issues     |
| **Security Isolation**      | No DLL injection attacks possible      |
| **Deterministic Behavior**  | Same binary = same behavior everywhere |
| **Simplified Rollback**     | Replace single file to rollback        |
| **Container Efficiency**    | Smaller images, faster startup         |

**Trade-offs:**

| Trade-off          | Impact                   | Mitigation                |
| ------------------ | ------------------------ | ------------------------- |
| **Larger Binary**  | 20-50 MB vs 10-20 MB     | Acceptable for enterprise |
| **Memory Sharing** | No shared library memory | Minimal impact            |
| **Updates**        | Full binary replacement  | Standard practice         |

**Recommendation:** For enterprise deployment, the security and operational benefits of single binary distribution significantly outweigh the modest increase in binary size.

### 6.2 Build Configuration for Single Binary

```powershell
# Recommended: Pure Rust build (ONNX backend)
cargo build --release --features onnx

# Alternative: Full build with static GGUF
$env:LLAMA_STATIC = "1"
cargo build --release --features full
```

### 6.3 Distribution Package

```
GG-CORE/
  GG-CORE.exe           # Single binary
  LICENSE                   # Apache 2.0
  NOTICE                    # Dependency attributions
  README.md                 # Quick start
  config/
    default.toml            # Default configuration
```

### 6.4 For Enterprise Distribution

1. **Use static linking** for all dependencies
2. **Build with pure Rust backends** (ONNX via Candle)
3. **Create NOTICE file** with attributions
4. **Provide pre-built binaries** to eliminate build requirement

### 6.2 For Maximum Compatibility

1. **Build separate binaries:**
   - `GG-CORE-onnx.exe` - Pure Rust, ONNX only
   - `GG-CORE-full.exe` - With GGUF backend
2. **Document build process** for custom builds
3. **Provide Docker images** for containerized deployment

### 6.3 For License Compliance

1. **Include LICENSE file** (Apache 2.0)
2. **Include NOTICE file** (dependency attributions)
3. **Run `cargo bundle-licenses`** for automated attribution
4. **Audit dependencies** before each release

---

## 7. Conclusion

### Can Dependencies Be Built In?

**Yes.** All Rust dependencies are statically linked by default. Native libraries (GGUF backend) can be statically linked with `LLAMA_STATIC=1`.

### Licensing Implications

**None.** All dependencies use MIT or Apache 2.0 licenses, which are fully compatible with GG-CORE's Apache 2.0 license and allow static linking.

### Recommended Distribution

**Single binary** with:

- Core runtime (always)
- ONNX backend (pure Rust, no native dependencies)
- GGUF backend (statically linked if needed)

This provides the simplest deployment experience with no external dependencies.

---

## Appendix: Full Dependency List

### A.1 Direct Dependencies

| Crate                 | Version | License        | Purpose              |
| --------------------- | ------- | -------------- | -------------------- |
| tokio                 | 1.35    | MIT            | Async runtime        |
| futures               | 0.3     | MIT/Apache-2.0 | Async utilities      |
| serde                 | 1.0     | MIT/Apache-2.0 | Serialization        |
| serde_json            | 1.0     | MIT/Apache-2.0 | JSON                 |
| bincode               | 1.3     | MIT            | Binary serialization |
| interprocess          | 2.0     | MIT            | IPC                  |
| candle-core           | 0.8     | MIT/Apache-2.0 | ML tensors           |
| candle-onnx           | 0.8     | MIT/Apache-2.0 | ONNX inference       |
| llama-cpp-2           | 0.1     | MIT            | GGUF inference       |
| thiserror             | 1.0     | MIT/Apache-2.0 | Error handling       |
| async-trait           | 0.1     | MIT/Apache-2.0 | Async traits         |
| sha2                  | 0.10    | MIT/Apache-2.0 | SHA-256              |
| hex                   | 0.4     | MIT/Apache-2.0 | Hex encoding         |
| rand                  | 0.8     | MIT/Apache-2.0 | CSPRNG               |
| regex                 | 1.10    | MIT/Apache-2.0 | Pattern matching     |
| unicode-normalization | 0.1     | MIT/Apache-2.0 | Unicode              |
| toml                  | 0.8     | MIT/Apache-2.0 | Config parsing       |
| memmap2               | 0.9     | MIT            | Memory mapping       |
| tracing               | 0.1     | MIT            | Logging              |
| tracing-subscriber    | 0.3     | MIT            | Logging              |
| metrics               | 0.22    | MIT            | Metrics              |
| parking_lot           | 0.12    | MIT/Apache-2.0 | Synchronization      |
| dashmap               | 6.0     | MIT            | Concurrent map       |
| num_cpus              | 1.16    | MIT/Apache-2.0 | CPU detection        |
| aho-corasick          | 1.1     | MIT/Apache-2.0 | Pattern matching     |
| aes                   | 0.8     | MIT/Apache-2.0 | AES encryption       |
| hostname              | 0.4     | MIT/Apache-2.0 | Hostname             |
| tempfile              | 3.10    | MIT/Apache-2.0 | Temp files           |
| windows-sys           | 0.52    | MIT/Apache-2.0 | Windows API          |

### A.2 Build Command Reference

```powershell
# Pure Rust build (recommended for distribution)
cargo build --release --features onnx

# Full build with static GGUF
$env:LLAMA_STATIC = "1"
cargo build --release --features full

# Core only (no backends)
cargo build --release
```

---

Copyright 2024-2026 GG-CORE Contributors  
Licensed under the Apache License, Version 2.0
