# Fuzz Testing for Veritas SDR

This directory contains fuzz testing targets for security-critical parsing code.

## Prerequisites

Install cargo-fuzz (requires nightly Rust):

```bash
rustup install nightly
cargo +nightly install cargo-fuzz
```

## Available Fuzz Targets

| Target | Description | Priority |
|--------|-------------|----------|
| `fuzz_ipc_json` | IPC JSON message decoding | High |
| `fuzz_ipc_binary` | IPC bincode message decoding | High |
| `fuzz_prompt_injection` | Prompt injection detection | High |
| `fuzz_pii_detection` | PII detection and redaction | Medium |
| `fuzz_output_sanitizer` | Output sanitization | Medium |

## Running Fuzz Tests

Run a specific target:

```bash
cd core-runtime
cargo +nightly fuzz run fuzz_ipc_json
```

Run with a time limit (recommended for CI):

```bash
cargo +nightly fuzz run fuzz_ipc_json -- -max_total_time=300
```

Run all targets sequentially:

```bash
for target in fuzz_ipc_json fuzz_ipc_binary fuzz_prompt_injection fuzz_pii_detection fuzz_output_sanitizer; do
    cargo +nightly fuzz run $target -- -max_total_time=60
done
```

## Interpreting Results

- **No crashes**: Target is robust against fuzzing
- **Crash found**: A bug was discovered, see `fuzz/artifacts/<target>/` for the input
- **Timeout**: Normal, fuzzing continues until you stop it

## Crash Artifacts

When a crash is found, the input that caused it is saved in:
```
fuzz/artifacts/<target_name>/crash-<hash>
```

To reproduce a crash:
```bash
cargo +nightly fuzz run fuzz_ipc_json fuzz/artifacts/fuzz_ipc_json/crash-<hash>
```

## Coverage

Generate coverage report:

```bash
cargo +nightly fuzz coverage fuzz_ipc_json
```

## Adding New Targets

1. Create a new file in `fuzz_targets/`
2. Add the binary entry to `Cargo.toml`
3. Follow the pattern in existing targets

## Security Notes

- Run fuzzing in a sandboxed environment
- Report any crashes found to the security team
- Do not commit crash artifacts containing sensitive data
