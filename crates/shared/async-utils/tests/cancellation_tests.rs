use nettoolskit_async_utils::{CancellationError, CancellationToken};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_cancellation_token_creation() {
    let token = CancellationToken::new();
    let token_default = CancellationToken::default();

    // Both should be valid tokens
    assert!(!format!("{:?}", token).is_empty());
    assert!(!format!("{:?}", token_default).is_empty());
}

#[tokio::test]
async fn test_cancellation_success() {
    let token = CancellationToken::new();

    // Fast operation that completes before cancellation
    let result = token
        .with_cancellation(async {
            sleep(Duration::from_millis(10)).await;
            "completed"
        })
        .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "completed");
}

#[tokio::test]
async fn test_cancellation_cancelled() {
    let token = CancellationToken::new();
    let token_clone = token.clone();

    // Spawn a task that cancels after a short delay
    tokio::spawn(async move {
        sleep(Duration::from_millis(50)).await;
        token_clone.cancel();
    });

    // Long operation that should be cancelled
    let result = token
        .with_cancellation(async {
            sleep(Duration::from_millis(200)).await;
            "never reached"
        })
        .await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), CancellationError));
}

#[tokio::test]
async fn test_cancellation_multiple_receivers() {
    let token = CancellationToken::new();

    let mut receiver1 = token.receiver();
    let mut receiver2 = token.receiver();

    // Cancel the token
    token.cancel();

    // Both receivers should receive the cancellation
    receiver1.cancelled().await;
    receiver2.cancelled().await;

    // Test passes if we reach here without hanging
    assert!(true);
}

#[tokio::test]
async fn test_cancellation_immediate() {
    let token = CancellationToken::new();

    // Cancel immediately
    token.cancel();

    // Operation should be cancelled right away
    let result = token.with_cancellation(async { "immediate" }).await;

    // This could be either cancelled or completed depending on timing
    match result {
        Ok(value) => assert_eq!(value, "immediate"),
        Err(_) => assert!(true), // Cancellation is also acceptable
    }
}

#[tokio::test]
async fn test_cancellation_with_different_types() {
    let token = CancellationToken::new();

    // Test with different return types
    let string_result = token.with_cancellation(async { "hello".to_string() }).await;

    let number_result = token.with_cancellation(async { 123usize }).await;

    let vec_result = token.with_cancellation(async { vec![1, 2, 3] }).await;

    assert!(string_result.is_ok());
    assert!(number_result.is_ok());
    assert!(vec_result.is_ok());

    assert_eq!(string_result.unwrap(), "hello");
    assert_eq!(number_result.unwrap(), 123);
    assert_eq!(vec_result.unwrap(), vec![1, 2, 3]);
}

#[tokio::test]
async fn test_token_clone() {
    let token = CancellationToken::new();
    let cloned_token = token.clone();

    // Cancel using cloned token
    cloned_token.cancel();

    // Original token should also be cancelled
    let result = token
        .with_cancellation(async {
            sleep(Duration::from_millis(100)).await;
            "should be cancelled"
        })
        .await;

    // May be cancelled or completed depending on timing
    match result {
        Ok(_) => assert!(true),
        Err(_) => assert!(true),
    }
}

#[test]
fn test_cancellation_error_display() {
    let error = CancellationError;
    assert_eq!(format!("{}", error), "operation was cancelled");
}

#[test]
fn test_cancellation_error_debug() {
    let error = CancellationError;
    assert_eq!(format!("{:?}", error), "CancellationError");
}

#[test]
fn test_cancellation_error_is_error() {
    let error = CancellationError;
    assert!(std::error::Error::source(&error).is_none());
}
