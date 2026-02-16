//! Tests for ModelRouter - atomic model_id → handle routing table.

use veritas_sdr::models::{ModelHandle, ModelRouter, RouterError};

#[tokio::test]
async fn test_add_route_success() {
    let router = ModelRouter::new();
    let handle = ModelHandle::new(1);

    let result = router.add_route("test-model", handle).await;
    assert!(result.is_ok());

    let resolved = router.resolve("test-model").await;
    assert_eq!(resolved, Some(handle));
}

#[tokio::test]
async fn test_add_route_duplicate_fails() {
    let router = ModelRouter::new();
    let handle1 = ModelHandle::new(1);
    let handle2 = ModelHandle::new(2);

    router.add_route("test-model", handle1).await.unwrap();
    let result = router.add_route("test-model", handle2).await;

    assert!(matches!(result, Err(RouterError::RouteExists(_))));
}

#[tokio::test]
async fn test_swap_route_atomic() {
    let router = ModelRouter::new();
    let old_handle = ModelHandle::new(1);
    let new_handle = ModelHandle::new(2);

    router.add_route("test-model", old_handle).await.unwrap();

    let returned = router.swap_route("test-model", new_handle).await;
    assert_eq!(returned, Some(old_handle));

    let resolved = router.resolve("test-model").await;
    assert_eq!(resolved, Some(new_handle));
}

#[tokio::test]
async fn test_swap_route_nonexistent_creates() {
    let router = ModelRouter::new();
    let handle = ModelHandle::new(1);

    let returned = router.swap_route("new-model", handle).await;
    assert_eq!(returned, None);

    let resolved = router.resolve("new-model").await;
    assert_eq!(resolved, Some(handle));
}

#[tokio::test]
async fn test_remove_route() {
    let router = ModelRouter::new();
    let handle = ModelHandle::new(1);

    router.add_route("test-model", handle).await.unwrap();

    let removed = router.remove_route("test-model").await;
    assert_eq!(removed, Some(handle));

    let resolved = router.resolve("test-model").await;
    assert_eq!(resolved, None);
}

#[tokio::test]
async fn test_resolve_nonexistent() {
    let router = ModelRouter::new();

    let resolved = router.resolve("nonexistent").await;
    assert_eq!(resolved, None);
}

#[tokio::test]
async fn test_list_routes() {
    let router = ModelRouter::new();
    let handle1 = ModelHandle::new(1);
    let handle2 = ModelHandle::new(2);

    router.add_route("model-a", handle1).await.unwrap();
    router.add_route("model-b", handle2).await.unwrap();

    let routes = router.list_routes().await;
    assert_eq!(routes.len(), 2);
    assert!(routes.contains(&("model-a".to_string(), handle1)));
    assert!(routes.contains(&("model-b".to_string(), handle2)));
}

#[tokio::test]
async fn test_concurrent_operations() {
    use std::sync::Arc;

    let router = Arc::new(ModelRouter::new());
    let mut handles = Vec::new();

    for i in 0..10 {
        let r = router.clone();
        let h = tokio::spawn(async move {
            let handle = ModelHandle::new(i);
            r.add_route(&format!("model-{}", i), handle).await.unwrap();
            r.resolve(&format!("model-{}", i)).await
        });
        handles.push(h);
    }

    for h in handles {
        let result = h.await.unwrap();
        assert!(result.is_some());
    }

    let routes = router.list_routes().await;
    assert_eq!(routes.len(), 10);
}
