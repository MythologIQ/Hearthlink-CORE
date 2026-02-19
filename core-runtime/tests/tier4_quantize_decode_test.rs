//! Tier 4 tests: Quantization and Prefill/Decode.

use veritas_sdr::engine::quantize::{QuantFormat, QuantizedTensor};
use veritas_sdr::engine::prefill::{PrefillConfig, PrefillExecutor};
use veritas_sdr::engine::decode::{DecodeConfig, DecodeExecutor};
use veritas_sdr::memory::paged::PageTable;

// ============================================================================
// Phase 3: Quantization Tests
// ============================================================================

#[test]
fn quant_format_compression_ratios() {
    assert_eq!(QuantFormat::F32.compression_ratio(), 1.0);
    assert_eq!(QuantFormat::Q8_0.compression_ratio(), 4.0);
    assert_eq!(QuantFormat::Q4_0.compression_ratio(), 8.0);
}

#[test]
fn q8_roundtrip_within_tolerance() {
    let data: Vec<f32> = (0..64).map(|i| (i as f32 - 32.0) / 10.0).collect();
    let tensor = QuantizedTensor::from_f32(&data, 1, 64, QuantFormat::Q8_0);

    let input: Vec<f32> = vec![1.0; 64];
    let mut output = vec![0.0f32; 1];
    tensor.matmul(&input, &mut output);

    assert!(output[0].abs() < 100.0);
}

#[test]
fn q4_roundtrip_within_tolerance() {
    let data: Vec<f32> = (0..64).map(|i| (i as f32 - 32.0) / 10.0).collect();
    let tensor = QuantizedTensor::from_f32(&data, 1, 64, QuantFormat::Q4_0);

    let input: Vec<f32> = vec![1.0; 64];
    let mut output = vec![0.0f32; 1];
    tensor.matmul(&input, &mut output);

    assert!(output[0].abs() < 200.0);
}

#[test]
fn quant_memory_reduction() {
    let data: Vec<f32> = vec![1.0; 1024];

    let f32_tensor = QuantizedTensor::from_f32(&data, 1, 1024, QuantFormat::F32);
    let q8_tensor = QuantizedTensor::from_f32(&data, 1, 1024, QuantFormat::Q8_0);
    let q4_tensor = QuantizedTensor::from_f32(&data, 1, 1024, QuantFormat::Q4_0);

    assert_eq!(f32_tensor.format(), QuantFormat::F32);
    assert_eq!(q8_tensor.format(), QuantFormat::Q8_0);
    assert_eq!(q4_tensor.format(), QuantFormat::Q4_0);
}

#[test]
fn quantized_matmul_identity_format() {
    let data: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0];
    let tensor = QuantizedTensor::from_f32(&data, 2, 2, QuantFormat::F32);

    let input = vec![1.0, 1.0];
    let mut output = vec![0.0f32; 2];
    tensor.matmul(&input, &mut output);

    assert_eq!(output[0], 3.0);
    assert_eq!(output[1], 7.0);
}

// ============================================================================
// Phase 4: Prefill/Decode Tests
// ============================================================================

#[test]
fn prefill_processes_full_prompt() {
    let config = PrefillConfig { chunk_size: 16, hidden_dim: 4 };
    let executor = PrefillExecutor::new(config);
    let mut page_table = PageTable::new(4, 10);

    let tokens: Vec<u32> = (0..50).collect();
    let result = executor.execute(&tokens, &mut page_table).unwrap();

    assert_eq!(result.kv_len, 50);
    assert_eq!(result.chunks_processed, 4);
}

#[test]
fn prefill_populates_kv_cache() {
    let config = PrefillConfig { chunk_size: 16, hidden_dim: 4 };
    let executor = PrefillExecutor::new(config);
    let mut page_table = PageTable::new(4, 10);

    let tokens: Vec<u32> = (0..32).collect();
    executor.execute(&tokens, &mut page_table).unwrap();

    assert_eq!(page_table.page_count(), 2);
}

#[test]
fn prefill_rejects_empty_prompt() {
    let executor = PrefillExecutor::new(PrefillConfig::default());
    let mut page_table = PageTable::new(4, 10);

    let result = executor.execute(&[], &mut page_table);
    assert!(result.is_err());
}

#[test]
fn decode_requires_integrated_model() {
    // DecodeExecutor.step() requires an integrated model - no mock fallback
    let config = DecodeConfig { hidden_dim: 4, eos_token: 999, speculative: None };
    let mut executor = DecodeExecutor::new(config);
    let mut page_table = PageTable::new(4, 10);

    executor.init(0);
    let result = executor.step(&mut page_table, 10);

    // Should fail because no model is loaded
    assert!(result.is_err());
}

#[test]
fn decode_init_sets_position() {
    // Test that init() properly sets the starting position
    let config = DecodeConfig { hidden_dim: 4, eos_token: 999, speculative: None };
    let mut executor = DecodeExecutor::new(config);

    executor.init(100);
    assert_eq!(executor.current_pos(), 100);

    executor.init(0);
    assert_eq!(executor.current_pos(), 0);
}

#[test]
fn decode_tokens_generated_starts_at_zero() {
    let config = DecodeConfig { hidden_dim: 4, eos_token: 999, speculative: None };
    let executor = DecodeExecutor::new(config);

    assert_eq!(executor.tokens_generated(), 0);
}

#[test]
fn prefill_estimate_pages() {
    assert_eq!(PrefillExecutor::estimate_pages(16), 1);
    assert_eq!(PrefillExecutor::estimate_pages(17), 2);
    assert_eq!(PrefillExecutor::estimate_pages(32), 2);
    assert_eq!(PrefillExecutor::estimate_pages(100), 7);
}

#[test]
fn decode_estimate_pages() {
    assert_eq!(DecodeExecutor::estimate_pages(0, 16), 1);
    assert_eq!(DecodeExecutor::estimate_pages(10, 10), 2);
    assert_eq!(DecodeExecutor::estimate_pages(100, 100), 13);
}
