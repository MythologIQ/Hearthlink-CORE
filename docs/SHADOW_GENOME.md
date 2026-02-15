# Shadow Genome

This document records failure modes from GATE TRIBUNAL vetoes to prevent repetition of similar errors.

---

## Failure Entry #1

**Date**: 2026-02-14T16:00:00+00:00
**Verdict ID**: Entry #65 (META_LEDGER.md)
**Failure Mode**: HALLUCINATION

### What Failed

Pre-Testing Hardening Bundle - Phase 2 (IPC Binary Encoding Integration)

### Why It Failed

The Governor proposed adding V2 encoder tests to `tests/encoding_roundtrip_test.rs` that already exist in the codebase:

| Proposed Test | Already Exists At |
|---------------|------------------|
| v2_encode_empty | line 107 |
| v2_encode_single | line 115 |
| v2_roundtrip | line 128 |
| v2_decode_truncated | line 137 |
| v2_decode_length_mismatch | line 145 |
| v2_smaller_than_v1 | line 155 (as v2_vs_v1_size_comparison) |

The plan stated "V2 binary encoder exists but may not be wired into the handler. Verify integration and add benchmark comparison." but then specified tests that were already implemented.

### Pattern to Avoid

**Before proposing new tests**:
1. Use `Grep` or `Read` to verify the test file doesn't already contain the proposed tests
2. Check existing test coverage before planning new tests
3. If tests exist, acknowledge them and scope the plan to only what's missing

### Remediation Attempted

Governor must revise plan to:
1. Remove duplicate test specifications
2. Acknowledge existing V2 encoder test coverage (12 tests at lines 104-189)
3. Limit Phase 2 to benchmark comparison only (if needed)

---

_Shadow Genome tracks failures to prevent repetition. Each entry is a lesson._
