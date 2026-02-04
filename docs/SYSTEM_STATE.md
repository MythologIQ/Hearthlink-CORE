# System State Snapshot

**Generated**: 2026-02-03T23:45:00+00:00
**Phase**: SUBSTANTIATE
**Status**: SEALED

## Physical Tree

```
core-runtime/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── ipc/
│   │   ├── mod.rs
│   │   ├── auth.rs
│   │   ├── handler.rs
│   │   └── protocol.rs
│   ├── scheduler/
│   │   ├── mod.rs
│   │   ├── priority.rs
│   │   ├── queue.rs
│   │   └── batch.rs
│   ├── engine/
│   │   ├── mod.rs
│   │   ├── inference.rs
│   │   ├── tokenizer.rs
│   │   └── streaming.rs
│   ├── models/
│   │   ├── mod.rs
│   │   ├── loader.rs
│   │   ├── registry.rs
│   │   └── swap.rs
│   └── memory/
│       ├── mod.rs
│       ├── pool.rs
│       ├── gpu.rs
│       └── cache.rs
└── tests/
    ├── auth_test.rs
    ├── protocol_test.rs
    ├── scheduler_test.rs
    └── memory_test.rs
```

## File Inventory

| Path | Lines | Purpose |
|------|-------|---------|
| `src/main.rs` | 49 | Entry point |
| `src/lib.rs` | 110 | Public API surface |
| `src/ipc/mod.rs` | 14 | IPC module root |
| `src/ipc/auth.rs` | 126 | Session authentication |
| `src/ipc/handler.rs` | 126 | Request/response handling |
| `src/ipc/protocol.rs` | 110 | Wire format, validation |
| `src/scheduler/mod.rs` | 11 | Scheduler module root |
| `src/scheduler/priority.rs` | 104 | Priority queue |
| `src/scheduler/queue.rs` | 98 | Request queue |
| `src/scheduler/batch.rs` | 98 | Batching logic |
| `src/engine/mod.rs` | 11 | Engine module root |
| `src/engine/inference.rs` | 102 | Core inference |
| `src/engine/tokenizer.rs` | 66 | Tokenization wrapper |
| `src/engine/streaming.rs` | 72 | Token streaming |
| `src/models/mod.rs` | 11 | Models module root |
| `src/models/loader.rs` | 89 | Model loading |
| `src/models/registry.rs` | 80 | Model registry |
| `src/models/swap.rs` | 95 | Hot swap logic |
| `src/memory/mod.rs` | 11 | Memory module root |
| `src/memory/pool.rs` | 90 | Memory pooling |
| `src/memory/gpu.rs` | 79 | GPU memory tracking |
| `src/memory/cache.rs` | 92 | Context caching |

**Total Source Lines**: 1,644
**Total Test Lines**: 354

## Module Dependencies

```
main.rs
└── lib.rs
    ├── ipc/
    │   ├── auth (standalone)
    │   ├── protocol (standalone)
    │   └── handler (-> auth, protocol, scheduler)
    ├── scheduler/
    │   ├── priority (standalone)
    │   ├── queue (-> priority, engine)
    │   └── batch (-> queue)
    ├── engine/
    │   ├── tokenizer (standalone)
    │   ├── inference (-> models)
    │   └── streaming (standalone)
    ├── models/
    │   ├── loader (standalone)
    │   ├── registry (standalone)
    │   └── swap (-> registry)
    └── memory/
        ├── pool (standalone)
        ├── gpu (standalone)
        └── cache (standalone)
```

## Forbidden Modules Status

| Module | Status |
|--------|--------|
| `auth/` | NOT PRESENT |
| `vault/` | NOT PRESENT |
| `synapse/` | NOT PRESENT |
| `plugins/` | NOT PRESENT |
| `network/` | NOT PRESENT |

## Dependencies

| Package | Version | Status |
|---------|---------|--------|
| tokio | 1.35 | APPROVED |
| serde | 1.0 | APPROVED |
| serde_json | 1.0 | APPROVED |
| interprocess | 2.0 | APPROVED |
| candle-core | 0.4 | APPROVED |
| candle-nn | 0.4 | APPROVED |
| candle-transformers | 0.4 | APPROVED |
| thiserror | 1.0 | APPROVED |
| sha2 | 0.10 | APPROVED |
| hex | 0.4 | APPROVED |

## Forbidden Dependencies Status

| Package | Status |
|---------|--------|
| reqwest | NOT PRESENT |
| hyper | NOT PRESENT |
| tungstenite | NOT PRESENT |
| tokio-tungstenite | NOT PRESENT |
| walkdir | NOT PRESENT |

---

_State verified and sealed by QoreLogic Judge_
