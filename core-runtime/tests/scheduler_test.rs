//! TDD-Light tests for scheduler module.

use core_runtime::engine::InferenceParams;
use core_runtime::scheduler::{
    BatchConfig, BatchProcessor, Priority, PriorityQueue, RequestQueue,
    RequestQueueConfig,
};

#[test]
fn priority_queue_orders_by_priority() {
    let mut queue: PriorityQueue<&str> = PriorityQueue::new();

    queue.push("low", Priority::Low);
    queue.push("critical", Priority::Critical);
    queue.push("normal", Priority::Normal);

    assert_eq!(queue.pop(), Some("critical"));
    assert_eq!(queue.pop(), Some("normal"));
    assert_eq!(queue.pop(), Some("low"));
}

#[test]
fn priority_queue_fifo_within_same_priority() {
    let mut queue: PriorityQueue<&str> = PriorityQueue::new();

    queue.push("first", Priority::Normal);
    queue.push("second", Priority::Normal);
    queue.push("third", Priority::Normal);

    assert_eq!(queue.pop(), Some("first"));
    assert_eq!(queue.pop(), Some("second"));
    assert_eq!(queue.pop(), Some("third"));
}

#[tokio::test]
async fn request_queue_enqueue_dequeue() {
    let queue = RequestQueue::new(RequestQueueConfig::default());

    let (id, position) = queue
        .enqueue(
            "model".to_string(),
            vec![1, 2, 3],
            InferenceParams::default(),
            Priority::Normal,
        )
        .await
        .unwrap();

    assert_eq!(id, 1);
    assert_eq!(position, 0);

    let request = queue.dequeue().await.unwrap();
    assert_eq!(request.id, 1);
    assert_eq!(request.model_id, "model");
}

#[test]
fn batch_processor_respects_size_limit() {
    let config = BatchConfig {
        max_batch_size: 2,
        max_total_tokens: 1000,
    };
    let processor = BatchProcessor::new(config);

    let requests = vec![
        create_test_request(1, 10),
        create_test_request(2, 10),
        create_test_request(3, 10),
    ];

    let batches = processor.create_batches(requests);

    assert_eq!(batches.len(), 2);
    assert_eq!(batches[0].len(), 2);
    assert_eq!(batches[1].len(), 1);
}

#[test]
fn batch_processor_respects_token_limit() {
    let config = BatchConfig {
        max_batch_size: 10,
        max_total_tokens: 25,
    };
    let processor = BatchProcessor::new(config);

    let requests = vec![
        create_test_request(1, 10),
        create_test_request(2, 10),
        create_test_request(3, 10),
    ];

    let batches = processor.create_batches(requests);

    assert_eq!(batches.len(), 2);
    assert_eq!(batches[0].total_tokens, 20);
    assert_eq!(batches[1].total_tokens, 10);
}

fn create_test_request(
    id: u64,
    token_count: usize,
) -> core_runtime::scheduler::QueuedRequest {
    core_runtime::scheduler::QueuedRequest {
        id,
        model_id: "test".to_string(),
        prompt_tokens: vec![0; token_count],
        params: InferenceParams::default(),
    }
}
