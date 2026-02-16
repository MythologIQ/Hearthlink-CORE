//! Integration tests for ONNX model operations.
//!
//! Tests ONNX model configuration and inference output structures.

use veritas_sdr::engine::{
    ClassificationResult, EmbeddingResult, InferenceInput, InferenceOutput,
    OnnxConfig,
};
use veritas_sdr::engine::onnx::OnnxDevice;
use veritas_sdr::models::ModelLoader;

fn create_test_loader() -> ModelLoader {
    let base = std::env::temp_dir().join("core_runtime_onnx_test");
    std::fs::create_dir_all(base.join("models")).ok();
    ModelLoader::new(base)
}

#[test]
fn onnx_config_defaults_valid() {
    let config = OnnxConfig::default();

    assert!(config.max_batch_size > 0, "Should have positive batch size");
    assert_eq!(config.device, OnnxDevice::Cpu, "CPU device by default");
}

#[test]
fn onnx_classifier_requires_valid_model_path() {
    let loader = create_test_loader();

    // Attempting to load non-existent model should fail
    let result = loader.validate_path("models/nonexistent.onnx");
    assert!(result.is_err(), "Should fail for non-existent model");
}

#[test]
fn onnx_embedder_requires_valid_model_path() {
    let loader = create_test_loader();

    // Attempting to load non-existent model should fail
    let result = loader.validate_path("models/nonexistent_embedder.onnx");
    assert!(result.is_err(), "Should fail for non-existent embedder model");
}

#[test]
fn classification_output_structure_valid() {
    // Test that ClassificationResult has expected structure
    let result = ClassificationResult {
        label: "positive".to_string(),
        confidence: 0.95,
        all_labels: vec![
            ("positive".to_string(), 0.95),
            ("negative".to_string(), 0.05),
        ],
    };

    assert_eq!(result.label, "positive");
    assert!(result.confidence > 0.9);
    assert_eq!(result.all_labels.len(), 2);
}

#[test]
fn embedding_output_structure_valid() {
    // Test that EmbeddingResult has expected structure
    let result = EmbeddingResult {
        vector: vec![0.1, 0.2, 0.3, 0.4, 0.5],
        dimensions: 5,
    };

    assert_eq!(result.vector.len(), result.dimensions);
    assert_eq!(result.dimensions, 5);
}

#[test]
fn inference_output_classification_identification() {
    let classification = ClassificationResult {
        label: "test".to_string(),
        confidence: 0.8,
        all_labels: vec![],
    };
    let output = InferenceOutput::Classification(classification);

    assert!(output.is_classification());
    assert!(!output.is_generation());
    assert!(!output.is_embedding());
}

#[test]
fn inference_output_embedding_identification() {
    let embedding = EmbeddingResult {
        vector: vec![0.0; 384],
        dimensions: 384,
    };
    let output = InferenceOutput::Embedding(embedding);

    assert!(!output.is_classification());
    assert!(!output.is_generation());
    assert!(output.is_embedding());
}

#[test]
fn inference_input_text_validation() {
    let input = InferenceInput::Text("Hello, world!".to_string());
    let result = input.validate();

    assert!(result.is_ok(), "Valid text input should pass validation");
}

#[test]
fn inference_input_batch_validation() {
    let input = InferenceInput::TextBatch(vec![
        "First text".to_string(),
        "Second text".to_string(),
    ]);
    let result = input.validate();

    assert!(result.is_ok(), "Valid batch input should pass validation");
}
