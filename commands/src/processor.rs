use crate::{ExitStatus, list, new, check, render, apply};
use owo_colors::OwoColorize;
use nettoolskit_ui::PRIMARY_COLOR;
use nettoolskit_otel::Metrics;

/// CLI Exit Status type for compatibility with CLI module
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CliExitStatus {
    Success,
    Error,
    Interrupted,
}

/// Process slash commands from CLI and return appropriate status
///
/// This function handles the mapping between CLI slash commands and the actual
/// command implementations, providing telemetry and logging for all operations.
/// It serves as the main dispatcher for interactive CLI commands.
///
/// # Arguments
///
/// * `cmd` - The slash command string (e.g., "/list", "/new", etc.)
///
/// # Returns
///
/// Returns `CliExitStatus` indicating the result of command execution
pub async fn process_command(cmd: &str) -> CliExitStatus {
    let mut metrics = Metrics::new();

    // Log command usage
    tracing::info!("Processing command: {}", cmd);
    metrics.increment_counter(&format!("command_{}_usage", cmd.trim_start_matches('/')));

    // Execute the appropriate command
    let result = match cmd {
        "/quit" => {
            tracing::info!("User requested quit");
            println!("{}", "👋 Goodbye!".color(PRIMARY_COLOR));
            ExitStatus::Success
        }
        "/list" => {
            tracing::debug!("Executing list command");
            let args = list::ListArgs::default();
            list::run(args).await
        }
        "/new" => {
            tracing::debug!("Executing new command");
            let args = new::NewArgs::default();
            new::run(args).await
        }
        "/check" => {
            tracing::debug!("Executing check command");
            let args = check::CheckArgs::default();
            check::run(args).await
        }
        "/render" => {
            tracing::debug!("Executing render command");
            let args = render::RenderArgs::default();
            render::run(args).await
        }
        "/apply" => {
            tracing::debug!("Executing apply command");
            let args = apply::ApplyArgs::default();
            apply::run(args).await
        }
        _ => {
            tracing::warn!("Unknown command attempted: {}", cmd);
            metrics.increment_counter("unknown_command_attempts");
            println!("{}: {}", "Unknown command".red(), cmd);
            ExitStatus::Error
        }
    };

    // Log result
    match result {
        ExitStatus::Success => {
            tracing::debug!("Command {} completed successfully", cmd);
            metrics.increment_counter("successful_commands");
        }
        ExitStatus::Error => {
            tracing::error!("Command {} failed", cmd);
            metrics.increment_counter("failed_commands");
        }
        ExitStatus::Interrupted => {
            tracing::warn!("Command {} was interrupted", cmd);
            metrics.increment_counter("interrupted_commands");
        }
    }

    // Convert to CLI ExitStatus
    match result {
        ExitStatus::Success => CliExitStatus::Success,
        ExitStatus::Error => CliExitStatus::Error,
        ExitStatus::Interrupted => CliExitStatus::Interrupted,
    }
}

/// Process non-command text input from CLI
///
/// Handles regular text input that is not a slash command, providing
/// basic feedback to the user about their input.
///
/// # Arguments
///
/// * `text` - The text input from the user
pub fn process_text(text: &str) {
    if !text.trim().is_empty() {
        println!("{}: {}", "You typed".color(PRIMARY_COLOR), text);
    }
}