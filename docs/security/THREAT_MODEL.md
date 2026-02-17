# Threat Model - Hearthlink CORE Runtime

**Version:** 0.6.0
**Document Date:** 2026-02-17
**Classification:** Internal / Audit Preparation
**Framework:** STRIDE + Attack Trees

---

## 1. System Overview

### 1.1 Purpose

Hearthlink CORE Runtime is a sandboxed, offline inference engine that performs model execution only. It operates as a pure compute service with no authority over data, tools, or system actions.

### 1.2 Design Principles (C.O.R.E.)

| Principle | Implementation |
|-----------|----------------|
| **Contained** | Separate OS process, restricted user, seccomp/AppContainer |
| **Offline** | Zero network access (inbound/outbound blocked) |
| **Restricted** | IPC-only communication with authenticated callers |
| **Execution** | Pure compute, no business logic or decision authority |

### 1.3 Trust Boundaries

```
┌─────────────────────────────────────────────────────────────────┐
│                        HOST SYSTEM                              │
│                                                                 │
│  ┌──────────────────┐          ┌──────────────────────────────┐│
│  │  Control Plane   │◄─────────►│       CORE Runtime          ││
│  │  (Trusted)       │   IPC    │     (Sandboxed)              ││
│  │                  │          │  ┌─────────────────────────┐ ││
│  │  - Auth decision │          │  │ Trust Boundary 2        │ ││
│  │  - Data policy   │          │  │                         │ ││
│  │  - Tool auth     │          │  │  Inference Engine       │ ││
│  └──────────────────┘          │  │  - Model loading        │ ││
│           │                    │  │  - Token generation     │ ││
│   Trust Boundary 1             │  │  - KV cache             │ ││
│           │                    │  └─────────────────────────┘ ││
│           ▼                    │                              ││
│  ┌──────────────────┐          │  Filesystem (Read-only):     ││
│  │  User Input      │          │  - models/                   ││
│  │  (Untrusted)     │          │  - tokenizers/               ││
│  └──────────────────┘          │                              ││
│                                └──────────────────────────────┘│
└─────────────────────────────────────────────────────────────────┘
```

**Trust Boundary 1 (TB1):** IPC channel between Control Plane and CORE
**Trust Boundary 2 (TB2):** Model execution environment

---

## 2. Protected Assets

| Asset | Sensitivity | Location | Protection Goal |
|-------|-------------|----------|-----------------|
| Model weights | HIGH | `models/` | Confidentiality, Integrity |
| User prompts | HIGH | IPC transit | Confidentiality |
| Inference outputs | HIGH | IPC transit | Integrity |
| Session tokens | HIGH | Memory | Confidentiality |
| Encryption keys | CRITICAL | Memory | Confidentiality |
| KV cache | MEDIUM | Memory | Availability |
| Telemetry data | LOW | Memory/disk | Integrity |

---

## 3. Threat Actors

### 3.1 External Threat Actors

| Actor | Capability | Motivation | Access |
|-------|------------|------------|--------|
| **Malicious User** | Crafted prompts | Data exfiltration, model abuse | Indirect via prompts |
| **Network Attacker** | N/A (offline) | N/A | None |
| **Supply Chain** | Compromised deps | Backdoor insertion | Build-time |

### 3.2 Internal Threat Actors

| Actor | Capability | Motivation | Access |
|-------|------------|------------|--------|
| **Rogue Process** | Local process | Privilege escalation | Same host |
| **Compromised Control Plane** | IPC access | Full system compromise | Direct IPC |

---

## 4. Attack Surfaces

### 4.1 IPC Protocol (Primary Attack Surface)

**Entry Points:**
- `decode_message()` - JSON parsing of incoming messages
- `decode_message_binary()` - Bincode deserialization
- Session handshake flow
- Streaming response channel

**Security Controls:**
- Size limits enforced (16MB max message)
- Authentication required for all requests
- Constant-time token comparison
- Rate limiting on authentication failures
- Protocol versioning

**Fuzz Targets:** `fuzz_ipc_json`, `fuzz_ipc_binary`

### 4.2 Prompt Processing

**Entry Points:**
- `InferenceRequest.prompt` field
- System prompt injection points
- Token streaming output

**Security Controls:**
- Prompt injection detection (Aho-Corasick pattern matching)
- Risk scoring (0-100 scale)
- Configurable blocking threshold
- PII detection and redaction
- Output sanitization

**Fuzz Targets:** `fuzz_prompt_injection`, `fuzz_pii_detection`, `fuzz_output_sanitizer`

### 4.3 Model Loading

**Entry Points:**
- Model file path specification
- GGUF/ONNX file parsing
- Tokenizer vocabulary loading

**Security Controls:**
- Path validation (no traversal)
- File format validation
- Size limits on model files
- Encrypted model support (AES-256-GCM)
- PBKDF2 key derivation (100K iterations)

### 4.4 Memory Management

**Entry Points:**
- KV cache allocation
- Token buffer management
- GPU memory allocation

**Security Controls:**
- Bounded memory pools
- Mutex poison recovery (graceful degradation)
- No unbounded allocations from untrusted input

---

## 5. STRIDE Analysis

### 5.1 Spoofing

| Threat | Impact | Mitigation | Status |
|--------|--------|------------|--------|
| Session hijacking | HIGH | CSPRNG session IDs, constant-time comparison | MITIGATED |
| Replay attacks | MEDIUM | Session timeouts, request IDs | MITIGATED |
| IPC impersonation | HIGH | Named pipe authentication, process verification | MITIGATED |

### 5.2 Tampering

| Threat | Impact | Mitigation | Status |
|--------|--------|------------|--------|
| Model file modification | HIGH | AES-256-GCM with auth tags | MITIGATED |
| IPC message tampering | HIGH | Message authentication (bincode CRC) | PARTIAL |
| Memory corruption | CRITICAL | Rust memory safety, bounds checking | MITIGATED |

### 5.3 Repudiation

| Threat | Impact | Mitigation | Status |
|--------|--------|------------|--------|
| Untracked model operations | MEDIUM | Audit logging module | MITIGATED |
| Missing access logs | MEDIUM | Security event logging | MITIGATED |

### 5.4 Information Disclosure

| Threat | Impact | Mitigation | Status |
|--------|--------|------------|--------|
| Prompt leakage | HIGH | No persistent storage, memory clearing | MITIGATED |
| Model extraction | HIGH | Process isolation, no network | MITIGATED |
| PII in outputs | MEDIUM | PII detection and redaction | MITIGATED |
| System prompt extraction | MEDIUM | Prompt injection filtering | MITIGATED |
| Timing side-channels | LOW | Constant-time auth comparison | MITIGATED |

### 5.5 Denial of Service

| Threat | Impact | Mitigation | Status |
|--------|--------|------------|--------|
| Message size bomb | HIGH | 16MB size limit, atomic validation | MITIGATED |
| Prompt length attack | MEDIUM | Token limits, configurable max | MITIGATED |
| Auth brute-force | MEDIUM | Rate limiting (5 attempts/30s block) | MITIGATED |
| Thread pool exhaustion | MEDIUM | Bounded thread pool, priority queuing | MITIGATED |
| KV cache exhaustion | MEDIUM | LRU eviction, bounded cache size | MITIGATED |

### 5.6 Elevation of Privilege

| Threat | Impact | Mitigation | Status |
|--------|--------|------------|--------|
| Sandbox escape | CRITICAL | seccomp/AppContainer, minimal syscalls | MITIGATED |
| File system traversal | HIGH | Path validation, allowlist directories | MITIGATED |
| Network access | HIGH | Zero network (blocked at OS level) | MITIGATED |
| Arbitrary code exec | CRITICAL | No plugin/script loading, no eval | MITIGATED |

---

## 6. Attack Trees

### 6.1 Model Extraction Attack

```
Goal: Extract proprietary model weights
├── Via Network Exfiltration
│   └── BLOCKED: No network access (OS-level deny)
├── Via File System Access
│   ├── Direct read of models/
│   │   └── BLOCKED: Sandboxed filesystem access
│   └── Path traversal
│       └── BLOCKED: Path validation, no ".." allowed
├── Via Memory Dump
│   ├── Process memory access
│   │   └── BLOCKED: Process isolation, restricted user
│   └── Core dump analysis
│       └── BLOCKED: Core dumps disabled in production
└── Via IPC Smuggling
    ├── Encode model in responses
    │   └── MITIGATED: Output size limits, sanitization
    └── Timing/side-channel
        └── PARTIAL: Constant-time ops for auth only
```

### 6.2 Prompt Injection Attack

```
Goal: Override system instructions or extract system prompt
├── Direct Instruction Override
│   ├── "Ignore previous instructions"
│   │   └── BLOCKED: Pattern matching, risk score
│   └── "You are now DAN"
│       └── BLOCKED: High-risk pattern detection
├── Indirect Injection
│   ├── Delimiter attacks (---, ```)
│   │   └── MITIGATED: Delimiter pattern matching
│   └── Encoding attacks (base64, rot13)
│       └── MITIGATED: Encoding pattern detection
├── System Prompt Extraction
│   ├── "Repeat your instructions"
│   │   └── BLOCKED: Extraction pattern matching
│   └── "What is your system prompt"
│       └── BLOCKED: Pattern matching
└── Context Manipulation
    ├── "This is only a test"
    │   └── MITIGATED: Context pattern detection
    └── "Hypothetically..."
        └── MITIGATED: Pattern matching
```

---

## 7. Security Controls Summary

### 7.1 Cryptographic Controls

| Control | Algorithm | Parameters | Standard |
|---------|-----------|------------|----------|
| Model encryption | AES-256-GCM | 96-bit nonce, 128-bit tag | NIST SP 800-38D |
| Key derivation | PBKDF2-HMAC-SHA256 | 100,000 iterations | OWASP minimum |
| Session ID generation | CSPRNG | 256-bit | OS random source |
| Token comparison | Constant-time | subtle crate | Timing-safe |

### 7.2 Access Controls

| Control | Implementation | Scope |
|---------|----------------|-------|
| IPC authentication | Session tokens + rate limiting | All requests |
| Filesystem access | Read: `models/`, `tokenizers/`. Write: `temp/`, `cache/` | Process-level |
| Network access | Deny all (OS-level) | Process-level |
| Memory access | Rust ownership + bounds checking | Language-level |

### 7.3 Input Validation

| Input | Validation | Location |
|-------|------------|----------|
| IPC messages | Size limits, format validation | `ipc/protocol.rs` |
| File paths | No traversal, allowlist | `models/loader.rs` |
| Prompts | Injection detection, PII scan | `security/prompt_injection.rs` |
| K8s CRDs | Path, image, model ID validation | `k8s/types.rs` |

### 7.4 Runtime Protection

| Control | Implementation | Recovery |
|---------|----------------|----------|
| Mutex poisoning | `unwrap_or_else(poison.into_inner())` | Graceful degradation |
| Thread panics | Catch unwind in thread pool | Worker respawn |
| Memory exhaustion | Bounded allocations, LRU eviction | Request rejection |

---

## 8. Residual Risks

### 8.1 Accepted Risks

| Risk | Severity | Rationale |
|------|----------|-----------|
| Side-channel attacks on inference | LOW | Constant-time only for auth; inference timing visible |
| Model format vulnerabilities | MEDIUM | Depends on upstream GGUF/ONNX parser security |
| Sophisticated prompt injection | MEDIUM | Pattern-based detection has bypass potential |

### 8.2 Risks Requiring External Validation

| Risk | Recommended Testing |
|------|---------------------|
| Sandbox escape vectors | Penetration testing |
| Memory corruption in unsafe blocks | Fuzzing + formal verification |
| Cryptographic implementation | Cryptographic audit |
| Supply chain dependencies | Dependency audit + SBOM |

---

## 9. Audit Recommendations

### 9.1 Priority Areas for External Audit

1. **IPC Protocol Parsing** (HIGH)
   - Files: `ipc/protocol.rs`, `ipc/handler.rs`
   - Focus: Deserialization safety, size validation, bounds checking

2. **Cryptographic Implementation** (HIGH)
   - Files: `security/encryption.rs`
   - Focus: Key derivation, nonce handling, error conditions

3. **Sandbox Boundaries** (HIGH)
   - Files: OS integration points
   - Focus: seccomp filters, AppContainer policies

4. **Unsafe Code Blocks** (MEDIUM)
   - Files: FFI boundaries, SIMD code
   - Focus: Memory safety invariants

5. **Prompt Injection Bypasses** (MEDIUM)
   - Files: `security/prompt_injection.rs`
   - Focus: Evasion techniques, Unicode normalization

### 9.2 Automated Testing Available

| Test Type | Location | Command |
|-----------|----------|---------|
| Unit tests | `src/**/tests.rs` | `cargo test` |
| Security tests | `tests/security_*.rs` | `cargo test --test security` |
| Fuzz tests | `fuzz/fuzz_targets/` | `cargo +nightly fuzz run <target>` |
| Benchmarks | `benches/` | `cargo bench` |

### 9.3 Documentation for Auditors

| Document | Location | Contents |
|----------|----------|----------|
| Concept | `docs/CONCEPT.md` | Design philosophy, anti-goals, security tax |
| Security Analysis | `docs/security/SECURITY_ANALYSIS_REPORT.md` | CVE remediations, test coverage |
| This Document | `docs/security/THREAT_MODEL.md` | Threat analysis, attack trees |
| Usage Guide | `docs/USAGE_GUIDE.md` | API reference and usage patterns |

---

## 10. Version History

| Version | Date | Changes |
|---------|------|---------|
| 0.6.0 | 2026-02-17 | Initial threat model, fuzz targets added |

---

## 11. Appendix: Security Test Coverage

### 11.1 Test Counts by Category

| Category | Count | Coverage |
|----------|-------|----------|
| Encryption | 20+ | PBKDF2, AES-GCM, edge cases |
| Authentication | 12 | Auth flows, timing, sessions |
| IPC Protocol | 16 | Versioning, encoding, limits |
| Prompt Injection | 10 | Patterns, sanitization, performance |
| PII Detection | 8 | Detection, redaction, consistency |
| K8s Validation | 15+ | Input validation |
| **Total** | 430+ | Security-critical paths |

### 11.2 Fuzz Target Coverage

| Target | Functions Covered | Priority |
|--------|-------------------|----------|
| `fuzz_ipc_json` | `decode_message()` | HIGH |
| `fuzz_ipc_binary` | `decode_message_binary()` | HIGH |
| `fuzz_prompt_injection` | `scan()`, `sanitize()` | HIGH |
| `fuzz_pii_detection` | `detect()`, `redact()` | MEDIUM |
| `fuzz_output_sanitizer` | `sanitize()`, `validate_format()` | MEDIUM |
