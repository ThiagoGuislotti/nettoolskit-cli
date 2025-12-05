//! AsyncCommandExecutor tests
//!
//! Tests for async command execution, cancellation, progress reporting,
//! and concurrent command management.

use nettoolskit_orchestrator::execution::AsyncCommandExecutor;
use std::time::Duration;

#[tokio::test]
async fn test_spawn_command() {
    // Arrange
    let mut executor = AsyncCommandExecutor::new();

    // Act
    let handle = executor.spawn(async {
        tokio::time::sleep(Duration::from_millis(10)).await;
        Ok("completed".to_string())
    });

    let result = handle.wait().await.unwrap();

    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "completed");
}

#[tokio::test]
async fn test_spawn_cancellable() {
    // Arrange
    let mut executor = AsyncCommandExecutor::new();

    // Act
    let mut handle = executor.spawn_cancellable(async {
        tokio::time::sleep(Duration::from_secs(10)).await;
        Ok("should not complete".to_string())
    });

    // Cancel immediately
    assert!(handle.cancel().await);

    let result = handle.wait().await.unwrap();

    // Assert
    assert!(result.is_err());
}

#[tokio::test]
async fn test_progress_reporting() {
    use nettoolskit_orchestrator::execution::CommandProgress;

    // Arrange
    let mut executor = AsyncCommandExecutor::new();

    // Act
    let (handle, mut progress_rx) = executor.spawn_with_progress(|progress_tx| async move {
        progress_tx
            .send(CommandProgress::percent("Step 1", 33))
            .ok();
        tokio::time::sleep(Duration::from_millis(10)).await;

        progress_tx
            .send(CommandProgress::percent("Step 2", 66))
            .ok();
        tokio::time::sleep(Duration::from_millis(10)).await;

        progress_tx
            .send(CommandProgress::percent("Step 3", 100))
            .ok();

        Ok("completed with progress".to_string())
    });

    // Collect progress updates
    let mut updates = Vec::new();
    while let Some(update) = progress_rx.recv().await {
        updates.push(update);
    }

    // Assert
    assert_eq!(updates.len(), 3);
    assert_eq!(updates[0].percent, Some(33));
    assert_eq!(updates[1].percent, Some(66));
    assert_eq!(updates[2].percent, Some(100));

    let result = handle.wait().await.unwrap();
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_concurrent_commands() {
    // Arrange
    let mut executor = AsyncCommandExecutor::with_limit(5);

    // Act
    let mut handles = Vec::new();
    for i in 0..5 {
        let handle = executor.spawn(async move {
            tokio::time::sleep(Duration::from_millis(10)).await;
            Ok(format!("command_{}", i))
        });
        handles.push(handle);
    }

    // Assert
    assert_eq!(executor.running_count(), 5);

    // Wait for all
    for handle in handles {
        let result = handle.wait().await.unwrap();
        assert!(result.is_ok());
    }
}