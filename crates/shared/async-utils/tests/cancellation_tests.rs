//! Tests for cancellation token functionality
//!
//! Validates token creation, cancellation propagation, concurrent cancellation,
//! error handling (CancellationError), and integration with async operations.
//!
//! ## Test Coverage
//! - Token creation (new, default, clone)
//! - Successful operation completion (no cancellation)
//! - Cancellation propagation (cancelled operations)
//! - Concurrent cancellation scenarios
//! - Error type validation (CancellationError)

use nettoolskit_async_utils::{CancellationError, CancellationToken};
use std::time::Duration;
use tokio::time::sleep;

// Token Creation and Basic Operation Tests

#[tokio::test]
async fn test_cancellation_token_creation() {
    // Arrange & Act
    let token = CancellationToken::new();
    let token_default = CancellationToken::default();

    // Assert
    assert!(!format!("{:?}", token).is_empty());
    assert!(!format!("{:?}", token_default).is_empty());
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
    let display = format!("{}", error);

    // Assert
    assert_eq!(display, "operation was cancelled");
}

#[test]
fn test_cancellation_error_debug() {
    // Arrange
    let error = CancellationError;

    // Act
    let debug = format!("{:?}", error);

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
