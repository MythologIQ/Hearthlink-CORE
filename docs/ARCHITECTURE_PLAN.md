# Architecture Plan

## Risk Grade: L3

### Risk Assessment

- [x] Contains security/auth logic -> L3 (IPC authentication, sandboxing, process isolation)
- [x] Modifies existing APIs -> L2 (new IPC contract)
- [ ] UI-only changes -> L1

**Rationale**: Runtime enforces security boundaries via sandboxing, IPC authentication tokens, and process isolation. Any implementation error creates attack surface.

## File Tree (The Contract)

```
core-runtime/
├── src/
│   ├── main.rs              # Entry point, process bootstrap
│   ├── lib.rs               # Public API surface
│   ├── ipc/
│   │   ├── mod.rs           # IPC module root
│   │   ├── handler.rs       # Request/response handling
│   │   ├── protocol.rs      # Wire format, schema validation
│   │   └── auth.rs          # Handshake token, session ID validation
│   ├── scheduler/
│   │   ├── mod.rs           # Scheduler module root
│   │   ├── queue.rs         # Request queue management
│   │   ├── priority.rs      # Request prioritization
│   │   └── batch.rs         # Batching logic
│   ├── engine/
│   │   ├── mod.rs           # Engine module root
│   │   ├── inference.rs     # Core inference execution
│   │   ├── tokenizer.rs     # Tokenization wrapper
│   │   └── streaming.rs     # Token streaming output
│   ├── models/
│   │   ├── mod.rs           # Model module root
│   │   ├── loader.rs        # Model loading/unloading
│   │   ├── registry.rs      # Loaded model tracking
│   │   └── swap.rs          # Hot swap logic
│   └── memory/
│       ├── mod.rs           # Memory module root
│       ├── pool.rs          # Memory pooling
│       ├── gpu.rs           # GPU memory management
│       └── cache.rs         # Context caching
├── Cargo.toml
└── README.md
```

## Forbidden Modules (Scope Creep Detection)

If any of these exist, ABORT:
- `auth/` (belongs to Control)
- `vault/` (belongs to Vault)
- `synapse/` (belongs to Synapse)
- `plugins/` (out of scope)
- `network/` (forbidden by design)

## Interface Contracts

### IPC Handler
- **Input**: `InferenceRequest { model_id: String, prompt_tokens: Vec<u32>, parameters: InferenceParams }`
- **Output**: `InferenceResponse { output_tokens: Vec<u32> }` (streamed)
- **Side Effects**: None (pure function)

### Model Loader
- **Input**: `ModelPath` (validated against allowed directories)
- **Output**: `Result<ModelHandle, LoadError>`
- **Side Effects**: Memory allocation in pool

### Scheduler
- **Input**: `InferenceRequest` + priority
- **Output**: Queued position, eventual `InferenceResponse`
- **Side Effects**: Queue state mutation

## Data Flow

```
[Control/Hearthlink]
       │
       ▼ (Named IPC only)
[IPC Handler] ──auth check──► REJECT if invalid
       │
       ▼
[Scheduler] ──queue──► [Engine]
                          │
                          ▼
                    [Model Registry]
                          │
                          ▼
                    [Inference] ──stream──► [IPC Handler] ──► [Control]
```

## Security Boundaries

| Boundary | Enforcement |
|----------|-------------|
| Process | Separate OS process, restricted user |
| Filesystem | Read: `models/`, `tokenizers/`. Write: `temp/`, `cache/`. Deny all else. |
| Network | Blocked (deny all inbound/outbound) |
| IPC | Named pipes/Unix sockets only. No HTTP/REST/WebSocket. |
| Authentication | Handshake token + runtime session ID required |

## Dependencies

| Package | Justification | Vanilla Alternative |
|---------|---------------|---------------------|
| `candle` or `llama-cpp-rs` | Model inference backend | No - requires GGML/GGUF support |
| `serde` | Serialization for IPC | No - standard Rust serialization |
| `tokio` | Async runtime for concurrency | Yes, but tokio is de facto standard |
| `interprocess` | Cross-platform IPC | Yes, but would require platform-specific code |

### Forbidden Dependencies

- `reqwest` - network access
- `hyper` - HTTP server
- Any WebSocket library
- Any filesystem traversal library

## Section 4 Razor Pre-Check

- [x] All planned functions <= 40 lines
- [x] All planned files <= 250 lines
- [x] No planned nesting > 3 levels

## Failure Modes

| Failure | Response |
|---------|----------|
| OOM | Graceful crash, no data loss |
| Invalid request | Reject with error code, continue |
| Model load failure | Return error, don't block queue |
| IPC disconnect | Clean shutdown, release resources |

---

*Blueprint sealed. Awaiting GATE tribunal.*
*Risk Grade L3: /ql-audit MANDATORY before implementation.*
