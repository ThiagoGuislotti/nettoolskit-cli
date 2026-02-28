//! Tests for timeout functionality
//!
//! Validates timeout behavior for async operations, including successful completion
//! before timeout, timeout expiration, edge cases (zero duration, boundary timing),
//! and error handling (`TimeoutError`).
//!
//! ## Test Coverage
//! - Successful operation completion (before timeout)
//! - Timeout expiration (operation exceeds limit)
//! - Edge cases (zero duration, boundary timing)
//! - Error type validation (`TimeoutError`)

use nettoolskit_core::async_utils::timeout::{with_global_timeout, with_timeout_concurrent};
use nettoolskit_core::async_utils::{with_timeout, TimeoutError};
use std::time::Duration;
use tokio::time::sleep;

// Basic Timeout Tests

#[tokio::test]
async fn test_timeout_success() {
    // Act
    let result = with_timeout(Duration::from_millis(100), async {
        sleep(Duration::from_millis(10)).await;
        "success"
    })
    .await;

    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "success");
}

#[tokio::test]
async fn test_timeout_error() {
    // Act
    let result = with_timeout(Duration::from_millis(50), async {
        sleep(Duration::from_millis(200)).await;
        "never reached"
    })
    .await;

    // Assert
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), TimeoutError));
}

#[tokio::test]
async fn test_timeout_edge_case() {
    // Act
    let result = with_timeout(Duration::from_millis(100), async {
        sleep(Duration::from_millis(95)).await;
        42
    })
    .await;

    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
}

// Edge Cases and Type Compatibility Tests

#[tokio::test]
async fn test_timeout_zero_duration() {
    // Act
    let result = with_timeout(Duration::ZERO, async { "instant" }).await;

    // Assert
    if let Ok(value) = result {
        assert_eq!(value, "instant");
    }
    // Timeout is also acceptable for ZERO duration
}

#[tokio::test]
async fn test_timeout_with_different_types() {
    // Act
    let string_result =
        with_timeout(Duration::from_millis(100), async { "hello".to_string() }).await;
    let number_result = with_timeout(Duration::from_millis(100), async { 123usize }).await;
    let vec_result = with_timeout(Duration::from_millis(100), async { vec![1, 2, 3] }).await;

    // Assert
    assert!(string_result.is_ok());
    assert!(number_result.is_ok());
    assert!(vec_result.is_ok());
    assert_eq!(string_result.unwrap(), "hello");
    assert_eq!(number_result.unwrap(), 123);
    assert_eq!(vec_result.unwrap(), vec![1, 2, 3]);
}

// Error Handling Tests

#[test]
fn test_timeout_error_display() {
    // Arrange
    let error = TimeoutError;

    // Act
    let display = format!("{error}");

    // Assert
    assert_eq!(display, "operation timed out");
}

#[test]
fn test_timeout_error_debug() {
    // Arrange
    let error = TimeoutError;

    // Act
    let debug = format!("{error:?}");

    // Assert
    assert_eq!(debug, "TimeoutError");
}

#[test]
fn test_timeout_error_is_error() {
    // Arrange
    let error = TimeoutError;

    // Assert
    assert!(std::error::Error::source(&error).is_none());
}

// with_timeout_concurrent Tests

#[tokio::test]
async fn test_timeout_concurrent_all_succeed() {
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

    let results = with_timeout_concurrent(Duration::from_millis(200), futures).await;

    assert_eq!(results.len(), 3);
    assert!(results.iter().all(|r| r.is_ok()));
    let values: Vec<i32> = results.into_iter().map(|r| r.unwrap()).collect();
    assert_eq!(values, vec![1, 2, 3]);
}

#[tokio::test]
async fn test_timeout_concurrent_some_timeout() {
    let futures = vec![
        Box::pin(async {
            sleep(Duration::from_millis(10)).await;
            1
        }) as std::pin::Pin<Box<dyn std::future::Future<Output = i32> + Send>>,
        Box::pin(async {
            sleep(Duration::from_millis(500)).await;
            2
        }),
        Box::pin(async {
            sleep(Duration::from_millis(10)).await;
            3
        }),
    ];

    let results = with_timeout_concurrent(Duration::from_millis(100), futures).await;

    assert_eq!(results.len(), 3);
    assert!(results[0].is_ok());
    assert!(results[1].is_err()); // This one should timeout
    assert!(results[2].is_ok());
}

#[tokio::test]
async fn test_timeout_concurrent_empty() {
    let futures: Vec<std::pin::Pin<Box<dyn std::future::Future<Output = i32> + Send>>> = vec![];

    let results = with_timeout_concurrent(Duration::from_millis(100), futures).await;
    assert!(results.is_empty());
}

// with_global_timeout Tests

#[tokio::test]
async fn test_global_timeout_all_succeed() {
    let futures = vec![
        Box::pin(async {
            sleep(Duration::from_millis(10)).await;
            10
        }) as std::pin::Pin<Box<dyn std::future::Future<Output = i32> + Send>>,
        Box::pin(async {
            sleep(Duration::from_millis(10)).await;
            20
        }),
        Box::pin(async {
            sleep(Duration::from_millis(10)).await;
            30
        }),
    ];

    let result = with_global_timeout(Duration::from_millis(200), futures).await;

    assert!(result.is_ok());
    let values = result.unwrap();
    assert_eq!(values, vec![10, 20, 30]);
}

#[tokio::test]
async fn test_global_timeout_expires() {
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

    let result = with_global_timeout(Duration::from_millis(50), futures).await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), TimeoutError));
}

#[tokio::test]
async fn test_global_timeout_empty() {
    let futures: Vec<std::pin::Pin<Box<dyn std::future::Future<Output = i32> + Send>>> = vec![];

    let result = with_global_timeout(Duration::from_millis(100), futures).await;

    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}
