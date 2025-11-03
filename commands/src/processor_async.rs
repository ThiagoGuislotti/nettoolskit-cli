/// Async command processor with progress support
///
/// This module provides a wrapper around command execution that uses
/// the async executor for non-blocking operations with progress feedback.
use crate::{async_executor::CommandProgress, check, list};
use nettoolskit_otel::{Metrics, Timer};
use owo_colors::OwoColorize;
use std::path::PathBuf;
use tokio::sync::mpsc;
use tracing::info;

/// Process async commands with progress display
///
/// This is similar to `process_command` but uses the async executor
/// for commands that benefit from progress reporting.
///
/// # Arguments
///
/// * `cmd` - The slash command string (e.g., "/check-async")
///
/// # Returns
///
/// Returns `crate::Result<String>` with the command output
pub async fn process_async_command(cmd: &str) -> crate::Result<String> {
    let metrics = Metrics::new();
    let timer = Timer::start("async_command_execution", metrics.clone());

    info!(
        command = %cmd,
        command_type = %cmd.trim_start_matches('/'),
        "Processing async CLI command"
    );

    // Execute the appropriate async command
    let result = match cmd {
        "/check-async" => {
            // Create a sample path for testing
            let args = check::CheckArgs {
                path: PathBuf::from("Cargo.toml"),
                strict: false,
            };

            // Create progress channel
            let (progress_tx, mut progress_rx) = mpsc::unbounded_channel();

            // Spawn progress display task
            let progress_handle = tokio::spawn(async move {
                while let Some(progress) = progress_rx.recv().await {
                    display_async_progress(&progress);
                }
            });

            // Execute command
            let result = check::run_async(args, progress_tx).await;

            // Wait for progress display to complete
            let _ = progress_handle.await;

            // Clear progress line
            clear_progress_line();

            result
        }
        "/list-async" => {
            // Get filter and tech from command if provided
            // For now, use defaults
            let args = list::ListArgs {
                filter: None,
                tech: None,
            };

            // Create progress channel
            let (progress_tx, mut progress_rx) = mpsc::unbounded_channel();

            // Spawn progress display task
            let progress_handle = tokio::spawn(async move {
                while let Some(progress) = progress_rx.recv().await {
                    display_async_progress(&progress);
                }
            });

            // Execute command
            let result = list::run_async(args, progress_tx).await;

            // Wait for progress display to complete
            let _ = progress_handle.await;

            // Clear progress line
            clear_progress_line();

            result
        }
        _ => {
            tracing::warn!("Unknown async command: {}", cmd);
            Err(format!("Unknown async command: {}", cmd).into())
        }
    };

    // Stop timer and log result
    let duration = timer.stop();
    info!(
        command = %cmd,
        duration_ms = duration.as_millis(),
        success = result.is_ok(),
        "Async command execution completed"
    );

    result
}

/// Display progress update for async commands
fn display_async_progress(progress: &CommandProgress) {
    use std::io::Write;

    // Clear current line
    print!("\r{}", " ".repeat(80));
    print!("\r");

    // Display message with color
    print!("{}", progress.message.bright_cyan());

    // Display percentage if available
    if let Some(percent) = progress.percent {
        print!(" {}%", percent.to_string().bright_green());
    }

    // Display task counts if available
    if let (Some(completed), Some(total)) = (progress.completed, progress.total) {
        print!(" ({}/{})", completed, total);
    }

    // Flush to ensure immediate display
    let _ = std::io::stdout().flush();
}

/// Clear the progress line
fn clear_progress_line() {
    use std::io::Write;
    print!("\r{}\r", " ".repeat(80));
    let _ = std::io::stdout().flush();
}
