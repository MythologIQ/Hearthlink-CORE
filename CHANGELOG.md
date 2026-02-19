# Changelog

All notable changes to Veritas SDR are documented in this file.

## [0.6.7] - 2026-02-19

### Production Safety Fixes

This release focuses on production safety and fail-fast behavior for the Hearthlink integration.

#### Fixed

- **Flash Attention Placeholder**: CUDA and Metal implementations now return explicit errors instead of zero vectors when kernel not implemented
- **Tokenizer Stub Behavior**: `encode()` and `decode()` now return `TokenizerError::NotLoaded` instead of silently returning empty results
- **Handler Metrics**: Fixed hardcoded `ModelHandle::new(0)` - now uses proper model lookup for metrics attribution
- **Telemetry Integration**: Handler now calls `telemetry::record_request_success()` and `record_request_failure()` for Prometheus-compatible metrics
- **FFI Streaming**: Updated to use model_id lookup; token-based API now fails fast with deprecation message
- **Benchmark Protocol**: Updated IPC throughput and scheduler benchmarks to use v0.6.5 text-based protocol

#### Added

- `InferenceEngine::get_handle()` method for model_id to ModelHandle resolution
- 8 new tests for InferenceEngine and InferenceParams validation
- Explicit version roadmap comments for unimplemented status --json fields (v0.7.0+)

#### Changed

- Tokenizer tests updated to expect `NotLoaded` errors instead of empty results
- Prompt fixtures updated to use text-based `prompt` field instead of `prompt_tokens`

### Breaking Changes

- FFI streaming with token arrays now returns `InvalidParams` error
- Stub tokenizer operations now fail instead of returning empty values

---

## [0.6.5] - 2026-02-18

### Text-Based IPC Protocol

- Eliminated mock data paths
- Changed IPC protocol from tokenized to text-based prompts
- Added chaos testing infrastructure

---

## [0.6.0] - 2026-02-17

### Functional GGUF Backend

- Functional GGUF inference via llama-cpp-2
- IPC server implementation
- Chaos testing framework

---

Copyright 2024-2026 Veritas SDR Contributors
