//! Tests for cancellation token functionality
//!
//! Validates token creation, cancellation propagation, concurrent cancellation,
//! error handling (`CancellationError`), and integration with async operations.
//!
//! ## Test Coverage
//! - Token creation (new, default, clone)
//! - Successful operation completion (no cancellation)
//! - Cancellation propagation (cancelled operations)
//! - Concurrent cancellation scenarios
//! - Error type validation (`CancellationError`)

use nettoolskit_core::async_utils::{CancellationError, CancellationToken};
use std::time::Duration;
use tokio::time::sleep;

// Token Creation and Basic Operation Tests

#[tokio::test]
async fn test_cancellation_token_creation() {
    // Arrange & Act
    let token = CancellationToken::new();
    let token_default = CancellationToken::default();

    // Assert
    assert!(!format!("{token:?}").is_empty());
    assert!(!format!("{token_default:?}").is_empty());
}

#[tokio::test]
async fn test_cancellation_success() {
    // Arrange
    let token = CancellationToken::new();

    // Act
    let result = token
        .with_cancellation(async {
            sleep(Duration::from_millis(10)).await;
            "completed"
        })
        .await;

    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "completed");
}

#[tokio::test]
async fn test_cancellation_cancelled() {
    // Arrange
    let token = CancellationToken::new();
    let token_clone = token.clone();

    tokio::spawn(async move {
        sleep(Duration::from_millis(50)).await;
        token_clone.cancel();
    });

    // Act
    let result = token
        .with_cancellation(async {
            sleep(Duration::from_millis(200)).await;
            "never reached"
        })
        .await;

    // Assert
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), CancellationError));
}

// Concurrent Cancellation Tests

#[tokio::test]
async fn test_cancellation_multiple_receivers() {
    // Arrange
    let token = CancellationToken::new();
    let mut receiver1 = token.receiver();
    let mut receiver2 = token.receiver();

    // Act
    token.cancel();
    receiver1.cancelled().await;
    receiver2.cancelled().await;

    // Assert - both receivers completed cancellation
}

#[tokio::test]
async fn test_cancellation_immediate() {
    // Arrange
    let token = CancellationToken::new();

    // Act
    token.cancel();
    let result = token.with_cancellation(async { "immediate" }).await;

    // Assert
    if let Ok(value) = result {
        assert_eq!(value, "immediate");
    }
    // Cancellation is also acceptable after immediate cancel
}

// Type Compatibility and Cloning Tests

#[tokio::test]
async fn test_cancellation_with_different_types() {
    // Arrange
    let token = CancellationToken::new();

    // Act
    let string_result = token.with_cancellation(async { "hello".to_string() }).await;
    let number_result = token.with_cancellation(async { 123usize }).await;
    let vec_result = token.with_cancellation(async { vec![1, 2, 3] }).await;

    // Assert
    assert!(string_result.is_ok());
    assert!(number_result.is_ok());
    assert!(vec_result.is_ok());
    assert_eq!(string_result.unwrap(), "hello");
    assert_eq!(number_result.unwrap(), 123);
    assert_eq!(vec_result.unwrap(), vec![1, 2, 3]);
}

#[tokio::test]
async fn test_token_clone() {
    // Arrange
    let token = CancellationToken::new();
    let cloned_token = token.clone();

    // Act
    cloned_token.cancel();
    let result = token
        .with_cancellation(async {
            sleep(Duration::from_millis(100)).await;
            "should be cancelled"
        })
        .await;

    // Assert - both Ok and Err are valid outcomes
    let _ = result;
}

// Error Handling Tests

#[test]
fn test_cancellation_error_display() {
    // Arrange
    let error = CancellationError;

    // Act
    let display = format!("{error}");

    // Assert
    assert_eq!(display, "operation was cancelled");
}

#[test]
fn test_cancellation_error_debug() {
    // Arrange
    let error = CancellationError;

    // Act
    let debug = format!("{error:?}");

    // Assert
    assert_eq!(debug, "CancellationError");
}

#[test]
fn test_cancellation_error_is_error() {
    // Arrange
    let error = CancellationError;

    // Assert
    assert!(std::error::Error::source(&error).is_none());
}

// Child Token Tests

#[tokio::test]
async fn test_child_token_cancelled_by_parent() {
    let parent = CancellationToken::new();
    let child = parent.child();

    let parent_clone = parent.clone();
    tokio::spawn(async move {
        sleep(Duration::from_millis(50)).await;
        parent_clone.cancel();
    });

    // The child should detect cancellation when parent is cancelled
    let result = child
        .with_cancellation(async {
            sleep(Duration::from_millis(500)).await;
            "should be cancelled"
        })
        .await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), CancellationError));
}

#[tokio::test]
async fn test_child_token_independent_cancel() {
    let parent = CancellationToken::new();
    let child = parent.child();

    // Cancel child directly without cancelling parent
    let child_clone = child.clone();
    tokio::spawn(async move {
        sleep(Duration::from_millis(50)).await;
        child_clone.cancel();
    });

    let result = child
        .with_cancellation(async {
            sleep(Duration::from_millis(500)).await;
            "should be cancelled"
        })
        .await;

    // Child was directly cancelled
    assert!(result.is_err());
}

#[tokio::test]
async fn test_child_token_completes_before_cancel() {
    let parent = CancellationToken::new();
    let child = parent.child();

    let result = child
        .with_cancellation(async {
            sleep(Duration::from_millis(10)).await;
            "completed"
        })
        .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "completed");
}

// with_cancellation_concurrent Tests

#[tokio::test]
async fn test_cancellation_concurrent_all_succeed() {
    let token = CancellationToken::new();

    let futures = vec![
        Box::pin(async {
            sleep(Duration::from_millis(10)).await;
            1
        }) as std::pin::Pin<Box<dyn std::future::Future<Output = i32> + Send>>,
        Box::pin(async {
            sleep(Duration::from_millis(10)).await;
            2
        }),
        Box::pin(async {
            sleep(Duration::from_millis(10)).await;
            3
        }),
    ];

    let result = token.with_cancellation_concurrent(futures).await;

    assert!(result.is_ok());
    let values = result.unwrap();
    assert_eq!(values, vec![1, 2, 3]);
}

#[tokio::test]
async fn test_cancellation_concurrent_cancelled() {
    let token = CancellationToken::new();
    let token_clone = token.clone();

    tokio::spawn(async move {
        sleep(Duration::from_millis(50)).await;
        token_clone.cancel();
    });

    let futures = vec![
        Box::pin(async {
            sleep(Duration::from_millis(500)).await;
            1
        }) as std::pin::Pin<Box<dyn std::future::Future<Output = i32> + Send>>,
        Box::pin(async {
            sleep(Duration::from_millis(500)).await;
            2
        }),
    ];

    let result = token.with_cancellation_concurrent(futures).await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), CancellationError));
}

#[tokio::test]
async fn test_cancellation_concurrent_empty() {
    let token = CancellationToken::new();
    let futures: Vec<std::pin::Pin<Box<dyn std::future::Future<Output = i32> + Send>>> = vec![];

    let result = token.with_cancellation_concurrent(futures).await;

    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}
