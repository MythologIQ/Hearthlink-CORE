//! TDD-Light tests for authentication module.

use std::time::Duration;

use gg_core::ipc::{AuthError, SessionAuth};

#[tokio::test]
async fn valid_token_creates_session() {
    let auth = SessionAuth::new("secret-token", Duration::from_secs(60));

    let result = auth.authenticate("secret-token").await;

    assert!(result.is_ok(), "Valid token should create session");
}

#[tokio::test]
async fn invalid_token_rejected() {
    let auth = SessionAuth::new("secret-token", Duration::from_secs(60));

    let result = auth.authenticate("wrong-token").await;

    assert!(matches!(result, Err(AuthError::InvalidToken)));
}

#[tokio::test]
async fn session_validates_successfully() {
    let auth = SessionAuth::new("secret-token", Duration::from_secs(60));
    let session = auth.authenticate("secret-token").await.unwrap();

    let result = auth.validate(&session).await;

    assert!(result.is_ok(), "Valid session should validate");
}

#[tokio::test]
async fn unknown_session_rejected() {
    let auth = SessionAuth::new("secret-token", Duration::from_secs(60));
    let session = auth.authenticate("secret-token").await.unwrap();

    // Create a different auth manager (no sessions)
    let other_auth = SessionAuth::new("secret-token", Duration::from_secs(60));
    let result = other_auth.validate(&session).await;

    assert!(matches!(result, Err(AuthError::SessionNotFound)));
}
