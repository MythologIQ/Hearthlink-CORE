//! Streaming execution helpers for the worker loop.

use crate::engine::InferenceEngine;
use crate::memory::ResourceLimits;
use crate::telemetry;
use super::streaming_queue::StreamingQueuedRequest;

/// Execute a streaming inference request with resource control.
pub(crate) async fn execute(
    engine: &InferenceEngine,
    resource_limits: Option<&ResourceLimits>,
    request: StreamingQueuedRequest,
) {
    let model_id = request.model_id.clone();

    let _guard = match super::worker::acquire_guard(
        engine, resource_limits, &model_id,
    ).await {
        Ok(g) => g,
        Err(msg) => {
            telemetry::record_admission_rejection(&model_id, &msg);
            let _ = send_error(&request.token_sender).await;
            return;
        }
    };

    let start = std::time::Instant::now();
    let result = run_stream(engine, &model_id, request.prompt, request.config, request.token_sender).await;
    let latency_ms = start.elapsed().as_millis() as u64;

    match result {
        Ok(Ok(())) => {
            telemetry::record_request_success(&model_id, latency_ms, 0);
        }
        Ok(Err(e)) => {
            telemetry::record_request_failure(&model_id, &e.to_string());
        }
        Err(e) => {
            telemetry::record_request_failure(&model_id, &e.to_string());
        }
    }
}

/// Run streaming inference on a blocking thread.
async fn run_stream(
    engine: &InferenceEngine,
    model_id: &str,
    prompt: String,
    config: crate::engine::InferenceConfig,
    sender: crate::engine::TokenStreamSender,
) -> Result<Result<(), crate::engine::inference::InferenceError>, tokio::task::JoinError> {
    #[cfg(feature = "gguf")]
    {
        let engine_ptr = engine as *const InferenceEngine;
        let mid = model_id.to_string();
        tokio::task::spawn_blocking(move || {
            // SAFETY: engine lives in the worker loop which awaits this.
            let engine = unsafe { &*engine_ptr };
            engine.run_stream_sync(&mid, &prompt, &config, sender)
        })
        .await
    }
    #[cfg(not(feature = "gguf"))]
    {
        let _ = (engine, model_id, prompt, config, sender);
        Ok(Err(crate::engine::inference::InferenceError::ExecutionFailed(
            "streaming requires gguf feature".into(),
        )))
    }
}

async fn send_error(sender: &crate::engine::TokenStreamSender) -> Result<(), ()> {
    sender.send(0, true).await.map_err(|_| ())
}
