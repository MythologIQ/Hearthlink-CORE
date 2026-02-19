//! TDD-Light tests for IPC protocol module.

use veritas_sdr::ipc::protocol::{
    decode_message, encode_message, InferenceRequest, InferenceResponse,
    IpcMessage, RequestId,
};
use veritas_sdr::engine::InferenceParams;

#[test]
fn handshake_roundtrip() {
    let message = IpcMessage::Handshake {
        token: "test-token".to_string(),
        protocol_version: None,
    };

    let encoded = encode_message(&message).unwrap();
    let decoded = decode_message(&encoded).unwrap();

    match decoded {
        IpcMessage::Handshake { token, .. } => assert_eq!(token, "test-token"),
        _ => panic!("Expected Handshake message"),
    }
}

#[test]
fn inference_request_roundtrip() {
    let request = InferenceRequest {
        request_id: RequestId(42),
        model_id: "test-model".to_string(),
        prompt: "Hello, world!".to_string(),
        parameters: InferenceParams::default(),
    };

    let message = IpcMessage::InferenceRequest(request);
    let encoded = encode_message(&message).unwrap();
    let decoded = decode_message(&encoded).unwrap();

    match decoded {
        IpcMessage::InferenceRequest(req) => {
            assert_eq!(req.request_id, RequestId(42));
            assert_eq!(req.model_id, "test-model");
            assert_eq!(req.prompt, "Hello, world!");
        }
        _ => panic!("Expected InferenceRequest message"),
    }
}

#[test]
fn inference_response_success() {
    let response = InferenceResponse::success(
        RequestId(1),
        "Generated text output".to_string(),
        5,
        true,
    );

    assert_eq!(response.request_id, RequestId(1));
    assert_eq!(response.output, "Generated text output");
    assert_eq!(response.tokens_generated, 5);
    assert!(response.finished);
    assert!(response.error.is_none());
}

#[test]
fn inference_response_error() {
    let response = InferenceResponse::error(RequestId(2), "Test error".to_string());

    assert_eq!(response.request_id, RequestId(2));
    assert!(response.output.is_empty());
    assert_eq!(response.tokens_generated, 0);
    assert!(response.finished);
    assert_eq!(response.error, Some("Test error".to_string()));
}

#[test]
fn request_validation_requires_model_id() {
    let request = InferenceRequest {
        request_id: RequestId(1),
        model_id: String::new(), // Empty
        prompt: "test".to_string(),
        parameters: InferenceParams::default(),
    };

    assert!(request.validate().is_err());
}

#[test]
fn request_validation_requires_prompt() {
    let request = InferenceRequest {
        request_id: RequestId(1),
        model_id: "model".to_string(),
        prompt: String::new(), // Empty
        parameters: InferenceParams::default(),
    };

    assert!(request.validate().is_err());
}
