/// Async command execution with progress display
///
/// This module integrates the async executor with the CLI loop,
/// providing non-blocking command execution with visual progress feedback.

use nettoolskit_commands::{
    AsyncCommandExecutor, CommandProgress, CommandResult,
};
use nettoolskit_ui::PRIMARY_COLOR;
use owo_colors::OwoColorize;
use std::io::{self, Write};
use tokio::sync::mpsc;

/// Execute a command asynchronously with progress display
///
/// This function spawns the command in the background and displays
/// progress updates while the command executes. The CLI remains
/// responsive during execution.
pub async fn execute_with_progress<F, Fut>(
    executor: &mut AsyncCommandExecutor,
    _command_name: &str,
    factory: F,
) -> CommandResult
where
    F: FnOnce(mpsc::UnboundedSender<CommandProgress>) -> Fut + Send + 'static,
    Fut: std::future::Future<Output = CommandResult> + Send + 'static,
{
    let (handle, mut progress_rx) = executor.spawn_with_progress(factory);

    // Display progress updates
    let progress_task = tokio::spawn(async move {
        while let Some(progress) = progress_rx.recv().await {
            display_progress(&progress);
        }
    });

    // Wait for command completion
    let result = match handle.wait().await {
        Ok(result) => result,
        Err(_) => Err("Command execution failed".into()),
    };

    // Wait for progress display to finish
    let _ = progress_task.await;

    result
}

/// Execute a cancellable command with Ctrl+C support
///
/// This function spawns a command that can be cancelled by the user.
/// It monitors for cancellation signals and terminates the command
/// gracefully if requested.
pub async fn execute_cancellable<F>(
    executor: &mut AsyncCommandExecutor,
    future: F,
) -> CommandResult
where
    F: std::future::Future<Output = CommandResult> + Send + 'static,
{
    let handle = executor.spawn_cancellable(future);

    // TODO: Wire up to actual Ctrl+C handler
    // For now, just wait for completion
    match handle.wait().await {
        Ok(result) => result,
        Err(_) => Err("Command execution failed".into()),
    }
}

/// Execute a simple async command without progress
pub async fn execute_simple<F>(executor: &mut AsyncCommandExecutor, future: F) -> CommandResult
where
    F: std::future::Future<Output = CommandResult> + Send + 'static,
{
    let handle = executor.spawn(future);

    match handle.wait().await {
        Ok(result) => result,
        Err(_) => Err("Command execution failed".into()),
    }
}

/// Display a progress update to the user
fn display_progress(progress: &CommandProgress) {
    // Clear current line
    print!("\r{}", " ".repeat(80));
    print!("\r");

    // Display message
    print!("{}", progress.message.color(PRIMARY_COLOR));

    // Display percentage if available
    if let Some(percent) = progress.percent {
        print!(" {}%", percent);
    }

    // Display task counts if available
    if let (Some(completed), Some(total)) = (progress.completed, progress.total) {
        print!(" ({}/{})", completed, total);
    }

    // Flush to ensure immediate display
    let _ = io::stdout().flush();
}

/// Clear the progress line
pub fn clear_progress_line() {
    print!("\r{}\r", " ".repeat(80));
    let _ = io::stdout().flush();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_execute_simple() {
        let mut executor = AsyncCommandExecutor::new();

        let result = execute_simple(&mut executor, async {
            sleep(Duration::from_millis(10)).await;
            Ok("Test completed".to_string())
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Test completed");
    }

    #[tokio::test]
    async fn test_execute_with_progress() {
        let mut executor = AsyncCommandExecutor::new();

        let result = execute_with_progress(&mut executor, "test", |progress| async move {
            progress
                .send(CommandProgress::message("Starting..."))
                .ok();
            sleep(Duration::from_millis(10)).await;

            progress
                .send(CommandProgress::percent("Processing...", 50))
                .ok();
            sleep(Duration::from_millis(10)).await;

            progress
                .send(CommandProgress::message("Done!"))
                .ok();

            Ok("Test completed".to_string())
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Test completed");
    }

    #[tokio::test]
    async fn test_execute_cancellable() {
        let mut executor = AsyncCommandExecutor::new();

        let result = execute_cancellable(&mut executor, async {
            sleep(Duration::from_millis(10)).await;
            Ok("Test completed".to_string())
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Test completed");
    }
}
