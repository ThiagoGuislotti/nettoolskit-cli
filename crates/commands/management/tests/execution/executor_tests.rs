//! Async Command Executor Tests
//!
//! Tests for async command executor validating command handle behavior,
//! progress tracking, cancellation support, and concurrent execution patterns.

use nettoolskit_management::{AsyncCommandExecutor, CommandProgress};
use std::time::Duration;
use tokio::time::sleep;

// CommandProgress Tests

#[tokio::test]
async fn test_command_progress_message() {
    // Act
    let progress = CommandProgress::message("Testing");

    // Assert
    assert_eq!(progress.message, "Testing");
    assert_eq!(progress.percent, None);
    assert_eq!(progress.completed, None);
    assert_eq!(progress.total, None);
}

#[tokio::test]
async fn test_command_progress_percent() {
    // Act
    let progress = CommandProgress::percent("Downloading", 75);

    // Assert
    assert_eq!(progress.message, "Downloading");
    assert_eq!(progress.percent, Some(75));
}

#[tokio::test]
async fn test_command_progress_steps() {
    // Act
    let progress = CommandProgress::steps("Processing", 3, 10);

    // Assert
    assert_eq!(progress.message, "Processing");
    assert_eq!(progress.completed, Some(3));
    assert_eq!(progress.total, Some(10));
}

// AsyncCommandExecutor Basic Tests

#[tokio::test]
async fn test_async_executor_spawn_simple() {
    // Arrange
    let mut executor = AsyncCommandExecutor::new();

    // Act
    let handle = executor.spawn(async { Ok("success".to_string()) });

    let result = handle.wait().await.unwrap();

    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "success");
}

#[tokio::test]
async fn test_async_executor_spawn_with_delay() {
    // Arrange
    let mut executor = AsyncCommandExecutor::new();

    // Act
    let handle = executor.spawn(async {
        sleep(Duration::from_millis(50)).await;
        Ok("delayed".to_string())
    });

    let result = handle.wait().await.unwrap();

    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "delayed");
}

#[tokio::test]
async fn test_async_executor_spawn_error() {
    // Arrange
    let mut executor = AsyncCommandExecutor::new();

    // Act
    let handle = executor.spawn(async { Err("command failed".into()) });

    let result = handle.wait().await.unwrap();

    // Assert
    assert!(result.is_err());
}

// Cancellation Tests

#[tokio::test]
async fn test_async_executor_spawn_cancellable() {
    // Arrange
    let mut executor = AsyncCommandExecutor::new();

    // Act
    let handle = executor.spawn_cancellable(async {
        sleep(Duration::from_millis(100)).await;
        Ok("completed".to_string())
    });

    let result = handle.wait().await.unwrap();

    // Assert
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_async_executor_cancel_command() {
    // Arrange
    let mut executor = AsyncCommandExecutor::new();

    let mut handle = executor.spawn_cancellable(async {
        sleep(Duration::from_secs(10)).await;
        Ok("should not reach here".to_string())
    });

    // Act
    let cancelled = handle.cancel().await;
    let result = handle.wait().await.unwrap();

    // Assert
    assert!(cancelled);
    assert!(result.is_err());
}

// Progress Tracking Tests

#[tokio::test]
async fn test_async_executor_spawn_with_progress() {
    // Arrange
    let mut executor = AsyncCommandExecutor::new();

    // Act
    let (handle, mut progress_rx) = executor.spawn_with_progress(|progress_tx| async move {
        let _ = progress_tx.send(CommandProgress::percent("Step 1", 25));
        sleep(Duration::from_millis(10)).await;

        let _ = progress_tx.send(CommandProgress::percent("Step 2", 50));
        sleep(Duration::from_millis(10)).await;

        let _ = progress_tx.send(CommandProgress::percent("Step 3", 75));
        sleep(Duration::from_millis(10)).await;

        let _ = progress_tx.send(CommandProgress::percent("Complete", 100));

        Ok("done".to_string())
    });

    let mut updates = Vec::new();
    while let Some(progress) = progress_rx.recv().await {
        updates.push(progress);
    }

    let result = handle.wait().await.unwrap();

    // Assert
    assert!(result.is_ok());
    assert_eq!(updates.len(), 4);
    assert_eq!(updates[0].percent, Some(25));
    assert_eq!(updates[1].percent, Some(50));
    assert_eq!(updates[2].percent, Some(75));
    assert_eq!(updates[3].percent, Some(100));
}

// Concurrent Execution Tests

#[tokio::test]
async fn test_async_executor_concurrent_commands() {
    // Arrange
    let mut executor = AsyncCommandExecutor::new();

    // Act
    let handle1 = executor.spawn(async {
        sleep(Duration::from_millis(20)).await;
        Ok("cmd1".to_string())
    });

    let handle2 = executor.spawn(async {
        sleep(Duration::from_millis(30)).await;
        Ok("cmd2".to_string())
    });

    let handle3 = executor.spawn(async {
        sleep(Duration::from_millis(10)).await;
        Ok("cmd3".to_string())
    });

    let (r1, r2, r3) = tokio::join!(handle1.wait(), handle2.wait(), handle3.wait());

    // Assert
    assert!(r1.is_ok());
    assert!(r2.is_ok());
    assert!(r3.is_ok());

    assert_eq!(r1.unwrap().unwrap(), "cmd1");
    assert_eq!(r2.unwrap().unwrap(), "cmd2");
    assert_eq!(r3.unwrap().unwrap(), "cmd3");
}

#[tokio::test]
async fn test_command_handle_try_result_not_ready() {
    // Arrange
    let mut executor = AsyncCommandExecutor::new();

    let mut handle = executor.spawn(async {
        sleep(Duration::from_millis(100)).await;
        Ok("result".to_string())
    });

    // Act & Assert - Not ready immediately
    assert!(handle.try_result().is_none());

    // Act - Wait and check again
    sleep(Duration::from_millis(150)).await;

    // Assert - Should be ready now
    assert!(handle.try_result().is_some());
}

#[tokio::test]
async fn test_async_executor_multiple_progress_updates() {
    // Arrange
    let mut executor = AsyncCommandExecutor::new();

    // Act
    let (handle, mut progress_rx) = executor.spawn_with_progress(|progress_tx| async move {
        for i in 0..10 {
            let _ = progress_tx.send(CommandProgress::steps("Processing", i, 10));
            sleep(Duration::from_millis(5)).await;
        }
        Ok("complete".to_string())
    });

    let collector = tokio::spawn(async move {
        let mut messages = Vec::new();
        while let Some(progress) = progress_rx.recv().await {
            messages.push(progress);
        }
        messages
    });

    let result = handle.wait().await;
    let progress_messages = collector.await.unwrap();

    // Assert
    assert!(result.is_ok());
    assert_eq!(progress_messages.len(), 10);
    assert_eq!(progress_messages[0].completed, Some(0));
    assert_eq!(progress_messages[9].completed, Some(9));
}

// Error Handling Tests

#[tokio::test]
async fn test_async_executor_error_propagation() {
    // Arrange
    let mut executor = AsyncCommandExecutor::new();

    // Act
    let handle = executor.spawn(async {
        sleep(Duration::from_millis(10)).await;
        Err("intentional error".into())
    });

    let result = handle.wait().await;

    // Assert
    assert!(result.is_ok()); // Receiver should work
    let cmd_result = result.unwrap();
    assert!(cmd_result.is_err()); // But command should have failed
}

#[tokio::test]
async fn test_command_handle_cancel_not_cancellable() {
    // Arrange
    let mut executor = AsyncCommandExecutor::new();

    let mut handle = executor.spawn(async { Ok("result".to_string()) });

    // Act
    let cancelled = handle.cancel().await;

    // Assert
    assert!(!cancelled);
}
