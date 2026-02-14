# Hearthlink CORE Runtime

<p align="center">
  <img src="HEARTHLINK-CORE.png" alt="Hearthlink CORE" width="220"/>
</p>

Hearthlink CORE is a sandboxed, offline inference engine for Hearthlink. It is a
headless execution engine that converts tokens to tokens and nothing else.
It exists to run models deterministically while enforcing strict
boundaries around data, network access, and authority.

This repository contains the CORE Runtime implementation and its
security-first architecture.

---

## Architecture

The authoritative design documents live in the `docs/` directory:

- `docs/CONCEPT.md` - Project concept, design philosophy, and performance goals
- `docs/META_LEDGER.md` - Merkle-chained decision log with audit trail
- `docs/SYSTEM_STATE.md` - System state snapshot and compliance tracking
- `docs/TANDEM_EXPERIMENTS_PROPOSAL.md` - Experimental framework proposals

Additional planning and architecture documents are in `PRIVATE/docs/` (excluded from version control).

## Design Pillars

The runtime is defined by four non-negotiables:

- **Contained**: Isolated process with minimal privileges
- **Offline**: No inbound or outbound network access
- **Restricted**: Authenticated IPC only
- **Execution**: Inference only, no governance or tooling

## Repository Layout

- `core-runtime/` - Rust implementation of runtime
- `docs/` - Architecture and system documentation
- `PRIVATE/docs/` - Internal planning and proprietary documents (gitignored)
- `.claude/` - Claude Code governance framework and templates
- `.agent/` - Project agent artifacts and staging outputs

## Build

This project uses Rust. Build runtime from its crate directory:

```bash
cd core-runtime
cargo build
```

For an optimized release build:

```bash
cd core-runtime
cargo build --release
```

## Test

Run the test suite:

```bash
cd core-runtime
cargo test
```

Run benchmarks (requires criterion):

```bash
cd core-runtime
cargo bench
```

## Security Notes

- The runtime is intentionally isolated from Vault, Synapse, plugins, and any
  external network.
- IPC is the only allowed communication path.
- Models are treated as untrusted data blobs and never gain privileges.
- All inference runs in a sandboxed process with OS-level isolation.

## QoreLogic A.E.G.I.S. Governance

This project uses the QoreLogic A.E.G.I.S. (Aligned Execution Governance for Intelligent Systems)
lifecycle framework for development governance.

### Lifecycle Phases

| Phase        | Command            | Purpose                                  |
| ------------ | ------------------ | ---------------------------------------- |
| ALIGN/ENCODE | `/ql-bootstrap`    | Initialize project DNA                   |
| GATE         | `/ql-audit`        | Security review for L2/L3 risk grades    |
| IMPLEMENT    | `/ql-implement`    | Execute blueprint                        |
| VERIFY       | `/ql-substantiate` | Validate implementation matches contract |

### Current Status

See `docs/META_LEDGER.md` for the complete lifecycle history and current phase.

## Performance Targets

The runtime targets three performance tiers accounting for "security tax" (40-88% overhead vs unsandboxed runtimes):

| Metric             | Tier 1    | Tier 2    | Tier 3    |
| ------------------ | --------- | --------- | --------- |
| Generation         | >10 tok/s | >25 tok/s | >50 tok/s |
| Classification P95 | <100ms    | <20ms     | <5ms      |
| Memory Ratio       | <1.5x     | <1.35x    | <1.25x    |

See `docs/CONCEPT.md` for detailed performance goals and competitive analysis.

## License

MIT. See [`LICENSE`](LICENSE).
