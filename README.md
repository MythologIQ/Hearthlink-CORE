# Hearthlink CORE Runtime

Hearthlink CORE is the local inference runtime for Hearthlink. It is a
sandboxed, offline execution engine that converts tokens to tokens and nothing
else. It exists to run models deterministically while enforcing strict
boundaries around data, network access, and authority.

This repository contains the CORE Runtime implementation and its
security-first architecture.

## Architecture

The authoritative design document lives at:

- `docs/architecture/CORE_RUNTIME_ARCHITECTURE.md`

That document is implementation-ready and defines the runtime’s scope,
trust model, IPC contract, and non-goals. If there is any conflict between
code and the architecture, the architecture wins.

## Design Pillars

The runtime is defined by four non-negotiables:

- Contained: isolated process with minimal privileges
- Offline: no inbound or outbound network access
- Restricted: authenticated IPC only
- Execution: inference only, no governance or tooling

## Repository Layout

- `core-runtime/` Rust implementation of the runtime
- `docs/` Architecture and system documentation
- `.claude/` Claude Code governance framework and templates
- `.agent/` Project agent artifacts and staging outputs

## Build

This project uses Rust. Build the runtime from its crate directory:

```bash
cd core-runtime
cargo build
```

For an optimized build:

```bash
cd core-runtime
cargo build --release
```

## Security Notes

- The runtime is intentionally isolated from Vault, Synapse, plugins, and any
  external network.
- IPC is the only allowed communication path.
- Models are treated as untrusted data blobs and never gain privileges.

## License

MIT. See `LICENSE`.
