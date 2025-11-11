/// Tests for async command executor
///
/// Validates command handle behavior, progress tracking,
/// cancellation support, and concurrent execution patterns.

use nettoolskit_commands::{CommandProgress, AsyncCommandExecutor};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_command_progress_message() {
    let progress = CommandProgress::message("Testing");
    assert_eq!(progress.message, "Testing");
    assert_eq!(progress.percent, None);
    assert_eq!(progress.completed, None);
    assert_eq!(progress.total, None);
}

#[tokio::test]
async fn test_command_progress_percent() {
    let progress = CommandProgress::percent("Downloading", 75);
    assert_eq!(progress.message, "Downloading");
    assert_eq!(progress.percent, Some(75));
}

#[tokio::test]
async fn test_command_progress_steps() {
    let progress = CommandProgress::steps("Processing", 3, 10);
    assert_eq!(progress.message, "Processing");
    assert_eq!(progress.completed, Some(3));
    assert_eq!(progress.total, Some(10));
}

#[tokio::test]
async fn test_async_executor_spawn_simple() {
    let mut executor = AsyncCommandExecutor::new();

    let handle = executor.spawn(async {
        Ok("success".to_string())
    });

    let result = handle.wait().await.unwrap();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "success");
}

#[tokio::test]
async fn test_async_executor_spawn_with_delay() {
    let mut executor = AsyncCommandExecutor::new();

    let handle = executor.spawn(async {
        sleep(Duration::from_millis(50)).await;
        Ok("delayed".to_string())
    });

    let result = handle.wait().await.unwrap();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "delayed");
}

#[tokio::test]
async fn test_async_executor_spawn_error() {
    let mut executor = AsyncCommandExecutor::new();

    let handle = executor.spawn(async {
        Err("command failed".into())
    });

    let result = handle.wait().await.unwrap();
    assert!(result.is_err());
}

#[tokio::test]
async fn test_async_executor_spawn_cancellable() {
    let mut executor = AsyncCommandExecutor::new();

    let handle = executor.spawn_cancellable(async {
        sleep(Duration::from_millis(100)).await;
        Ok("completed".to_string())
    });

    // Don't cancel, let it complete
    let result = handle.wait().await.unwrap();
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_async_executor_cancel_command() {
    let mut executor = AsyncCommandExecutor::new();

    let mut handle = executor.spawn_cancellable(async {
        sleep(Duration::from_secs(10)).await;
        Ok("should not reach here".to_string())
    });

    // Cancel immediately
    let cancelled = handle.cancel().await;
    assert!(cancelled);

    let result = handle.wait().await.unwrap();
    assert!(result.is_err());
}

#[tokio::test]
async fn test_async_executor_spawn_with_progress() {
    let mut executor = AsyncCommandExecutor::new();

    let (handle, mut progress_rx) = executor.spawn_with_progress(|progress_tx| async move {
        // Send progress updates
        let _ = progress_tx.send(CommandProgress::percent("Step 1", 25));
        sleep(Duration::from_millis(10)).await;

        let _ = progress_tx.send(CommandProgress::percent("Step 2", 50));
        sleep(Duration::from_millis(10)).await;

        let _ = progress_tx.send(CommandProgress::percent("Step 3", 75));
        sleep(Duration::from_millis(10)).await;

        let _ = progress_tx.send(CommandProgress::percent("Complete", 100));

        Ok("done".to_string())
    });

    // Collect progress updates
    let mut updates = Vec::new();
    while let Some(progress) = progress_rx.recv().await {
        updates.push(progress);
    }

    // Wait for completion
    let result = handle.wait().await.unwrap();
    assert!(result.is_ok());

    // Verify progress updates
    assert_eq!(updates.len(), 4);
    assert_eq!(updates[0].percent, Some(25));
    assert_eq!(updates[1].percent, Some(50));
    assert_eq!(updates[2].percent, Some(75));
    assert_eq!(updates[3].percent, Some(100));
}

#[tokio::test]
async fn test_async_executor_concurrent_commands() {
    let mut executor = AsyncCommandExecutor::new();

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

    // Wait for all in parallel
    let (r1, r2, r3) = tokio::join!(
        handle1.wait(),
        handle2.wait(),
        handle3.wait()
    );

    assert!(r1.is_ok());
    assert!(r2.is_ok());
    assert!(r3.is_ok());

    assert_eq!(r1.unwrap().unwrap(), "cmd1");
    assert_eq!(r2.unwrap().unwrap(), "cmd2");
    assert_eq!(r3.unwrap().unwrap(), "cmd3");
}

#[tokio::test]
async fn test_command_handle_try_result_not_ready() {
    let mut executor = AsyncCommandExecutor::new();

    let mut handle = executor.spawn(async {
        sleep(Duration::from_millis(100)).await;
        Ok("result".to_string())
    });

    // Should not be ready immediately
    assert!(handle.try_result().is_none());

    // Wait and try again
    sleep(Duration::from_millis(150)).await;
    assert!(handle.try_result().is_some());
}

#[tokio::test]
async fn test_async_executor_multiple_progress_updates() {
    let mut executor = AsyncCommandExecutor::new();

    let (handle, mut progress_rx) = executor.spawn_with_progress(|progress_tx| async move {
        for i in 0..10 {
            let _ = progress_tx.send(CommandProgress::steps("Processing", i, 10));
            sleep(Duration::from_millis(5)).await;
        }
        Ok("complete".to_string())
    });

    // Spawn task to collect progress
    let collector = tokio::spawn(async move {
        let mut messages = Vec::new();
        while let Some(progress) = progress_rx.recv().await {
            messages.push(progress);
        }
        messages
    });

    // Wait for command
    let result = handle.wait().await;
    assert!(result.is_ok());

    // Collect progress
    let progress_messages = collector.await.unwrap();    // Should have received 10 progress updates
    assert_eq!(progress_messages.len(), 10);
    assert_eq!(progress_messages[0].completed, Some(0));
    assert_eq!(progress_messages[9].completed, Some(9));
}

#[tokio::test]
async fn test_async_executor_error_propagation() {
    let mut executor = AsyncCommandExecutor::new();

    let handle = executor.spawn(async {
        sleep(Duration::from_millis(10)).await;
        Err("intentional error".into())
    });

    let result = handle.wait().await;
    assert!(result.is_ok()); // Receiver should work

    let cmd_result = result.unwrap();
    assert!(cmd_result.is_err()); // But command should have failed
}

#[tokio::test]
async fn test_command_handle_cancel_not_cancellable() {
    let mut executor = AsyncCommandExecutor::new();

    let mut handle = executor.spawn(async {
        Ok("result".to_string())
    });

    // Should return false for non-cancellable commands
    let cancelled = handle.cancel().await;
    assert!(!cancelled);
}