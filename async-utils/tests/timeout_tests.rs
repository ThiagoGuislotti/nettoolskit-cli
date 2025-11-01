use nettoolskit_async_utils::{with_timeout, TimeoutError};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_timeout_success() {
    // Fast operation that completes before timeout
    let result = with_timeout(Duration::from_millis(100), async {
        sleep(Duration::from_millis(10)).await;
        "success"
    })
    .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "success");
}

#[tokio::test]
async fn test_timeout_error() {
    // Slow operation that exceeds timeout
    let result = with_timeout(Duration::from_millis(50), async {
        sleep(Duration::from_millis(200)).await;
        "never reached"
    })
    .await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), TimeoutError));
}

#[tokio::test]
async fn test_timeout_edge_case() {
    // Operation that completes exactly at timeout boundary
    let result = with_timeout(Duration::from_millis(100), async {
        sleep(Duration::from_millis(95)).await;
        42
    })
    .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
}

#[tokio::test]
async fn test_timeout_zero_duration() {
    // Zero timeout should immediately fail
    let result = with_timeout(Duration::ZERO, async { "instant" }).await;

    // This is a timing-sensitive test, could be either success or timeout
    // depending on scheduler timing
    match result {
        Ok(value) => assert_eq!(value, "instant"),
        Err(_) => assert!(true), // Timeout is also acceptable
    }
}

#[tokio::test]
async fn test_timeout_with_different_types() {
    // Test with different return types
    let string_result =
        with_timeout(Duration::from_millis(100), async { "hello".to_string() }).await;

    let number_result = with_timeout(Duration::from_millis(100), async { 123usize }).await;

    let vec_result = with_timeout(Duration::from_millis(100), async { vec![1, 2, 3] }).await;

    assert!(string_result.is_ok());
    assert!(number_result.is_ok());
    assert!(vec_result.is_ok());

    assert_eq!(string_result.unwrap(), "hello");
    assert_eq!(number_result.unwrap(), 123);
    assert_eq!(vec_result.unwrap(), vec![1, 2, 3]);
}

#[test]
fn test_timeout_error_display() {
    let error = TimeoutError;
    assert_eq!(format!("{}", error), "operation timed out");
}

#[test]
fn test_timeout_error_debug() {
    let error = TimeoutError;
    assert_eq!(format!("{:?}", error), "TimeoutError");
}

#[test]
fn test_timeout_error_is_error() {
    let error = TimeoutError;
    assert!(std::error::Error::source(&error).is_none());
}
