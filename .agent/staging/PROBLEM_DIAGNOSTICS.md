# CURRENT PROBLEM DIAGNOSTICS

## 1. FilterConfig Field Mismatch

- **Location**: `tests/integration_end_to_end_test.rs`
- **Error**: `no field max_output_length on type FilterConfig`
- **Diagnosis**: The test is attempting to use `max_output_length`, but the actual struct definition in `src/engine/filter.rs` uses `max_output_chars`.
- **Recommendation**: Rename the field in the test to `max_output_chars`.

## 2. GgufConfig Field Mismatches

- **Location**: `tests/integration_gguf_test.rs`
- **Errors**:
  - `no field context_size` (should be `n_ctx`)
  - `no field threads` (should be `n_threads`)
  - `no field use_gpu` (should be `n_gpu_layers`)
- **Diagnosis**: The tests use high-level descriptive names, while the `GgufConfig` struct uses `llama-cpp` specific field names.
- **Recommendation**: Align the test assertions with the `llama-cpp` naming convention used in the implementation.

## 3. OnnxConfig Field Mismatches

- **Location**: `tests/integration_onnx_test.rs`
- **Errors**:
  - `no field threads` (not present in `OnnxConfig`)
  - `no field use_gpu` (should be represented via `device` field)
- **Diagnosis**: The `OnnxConfig` struct has a simplified definition (`max_batch_size`, `device`) compared to what the tests are checking.
- **Recommendation**: Update the tests to use the `device` field (e.g., `OnnxDevice::Cpu`) and remove assertions for non-existent `threads` field.

## 4. Unused Variable: runtime

- **Location**: `src/main.rs:33`
- **Diagnosis**: The `runtime` object is passed to `run_ipc_server` but is only referenced in a comment, not in the actual executable code of the loop.
- **Recommendation**: Implement the IPC handler logic or prefix the variable with an underscore if it's intentionally a placeholder.

## 5. Unused Variable: new_handle

- **Location**: `src/models/swap.rs:58`
- **Diagnosis**: `new_handle` is passed to `execute_swap` but not used. The logic currently only handles unregistering the `old_handle`.
- **Recommendation**: Implement the registration of `new_handle` or remove it if not yet needed for the current swap logic.

## 6. Dead Code: collected_at

- **Location**: `tests/baseline_comparison_test.rs:12`
- **Diagnosis**: The `collected_at` field is parsed from JSON but never read in any test assertions.
- **Recommendation**: Use the field in a test (e.g., to verify valid date format) or suppress the warning if it's strictly for informational purposes in the fixture.
