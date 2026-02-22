//! Tests for request queue and streaming admission guard.

#[cfg(test)]
mod tests {
    use crate::engine::InferenceParams;
    use crate::scheduler::{Priority, RequestQueue, RequestQueueConfig};

    fn small_queue() -> RequestQueue {
        RequestQueue::new(RequestQueueConfig {
            max_pending: 2,
            max_context_tokens: 4096,
        })
    }

    #[tokio::test]
    async fn enqueue_with_response_returns_receiver() {
        let q = small_queue();
        let result = q
            .enqueue_with_response(
                "model".into(),
                "hello".into(),
                InferenceParams::default(),
                Priority::Normal,
            )
            .await;
        assert!(result.is_ok());
        let (id, _rx) = result.unwrap();
        assert!(id > 0);
    }

    #[tokio::test]
    async fn queue_full_rejects_enqueue() {
        let q = small_queue();
        for _ in 0..2 {
            q.enqueue("m".into(), "p".into(), InferenceParams::default(), Priority::Normal)
                .await
                .unwrap();
        }
        let err = q
            .enqueue("m".into(), "p".into(), InferenceParams::default(), Priority::Normal)
            .await;
        assert!(err.is_err());
    }

    #[tokio::test]
    async fn streaming_guard_drops_releases_slot() {
        let q = small_queue();
        assert_eq!(q.streaming_count(), 0);

        let guard = q.admit_streaming("short prompt").await.unwrap();
        assert_eq!(q.streaming_count(), 1);

        drop(guard);
        assert_eq!(q.streaming_count(), 0);
    }

    #[tokio::test]
    async fn streaming_slots_count_toward_capacity() {
        let q = small_queue(); // max_pending = 2

        // Reserve 2 streaming slots => queue should be full
        let g1 = q.admit_streaming("a").await.unwrap();
        let g2 = q.admit_streaming("b").await.unwrap();

        // Enqueue should fail — streaming slots fill capacity
        let err = q
            .enqueue("m".into(), "p".into(), InferenceParams::default(), Priority::Normal)
            .await;
        assert!(err.is_err());

        // Third streaming admission should also fail
        let err = q.admit_streaming("c").await;
        assert!(err.is_err());

        // Drop one guard => one slot freed
        drop(g1);
        assert_eq!(q.streaming_count(), 1);

        // Now enqueue should succeed
        let ok = q
            .enqueue("m".into(), "p".into(), InferenceParams::default(), Priority::Normal)
            .await;
        assert!(ok.is_ok());

        drop(g2);
    }

    #[tokio::test]
    async fn concurrent_streaming_respects_capacity() {
        let q = small_queue(); // max_pending = 2

        // Admit two streaming requests, holding guards alive
        let g1 = q.admit_streaming("prompt").await;
        let g2 = q.admit_streaming("prompt").await;
        assert!(g1.is_ok());
        assert!(g2.is_ok());
        assert_eq!(q.streaming_count(), 2);

        // Third admission must fail while guards are held
        let g3 = q.admit_streaming("prompt").await;
        assert!(g3.is_err());

        // Enqueue must also fail (streaming fills capacity)
        let enq = q
            .enqueue("m".into(), "p".into(), InferenceParams::default(), Priority::Normal)
            .await;
        assert!(enq.is_err());

        // Drop one guard, now one slot available
        drop(g1.unwrap());
        assert_eq!(q.streaming_count(), 1);

        let g4 = q.admit_streaming("prompt").await;
        assert!(g4.is_ok());

        drop(g2.unwrap());
        drop(g4.unwrap());
    }
}
