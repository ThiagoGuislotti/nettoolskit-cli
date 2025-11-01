use crate::{apply, check, list, new, render, ExitStatus};
use nettoolskit_otel::{Metrics, Timer};
use nettoolskit_ui::PRIMARY_COLOR;
use owo_colors::OwoColorize;
use tracing::info;

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
    let metrics = Metrics::new();
    let timer = Timer::start("command_execution", metrics.clone());

    // Log command usage with structured data
    info!(
        command = %cmd,
        command_type = %cmd.trim_start_matches('/'),
        "Processing CLI command"
    );
    metrics.increment_counter(format!("command_{}_usage", cmd.trim_start_matches('/')));

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

    // Stop timer and log result with structured data
    let duration = timer.stop();

    // Log and convert result to CLI status
    let (status_str, counter_name, cli_status) = match result {
        ExitStatus::Success => ("success", "successful_commands", CliExitStatus::Success),
        ExitStatus::Error => ("error", "failed_commands", CliExitStatus::Error),
        ExitStatus::Interrupted => (
            "interrupted",
            "interrupted_commands",
            CliExitStatus::Interrupted,
        ),
    };

    info!(
        command = %cmd,
        duration_ms = duration.as_millis(),
        status = status_str,
        "Command execution completed"
    );
    metrics.increment_counter(counter_name);

    // Log metrics summary for this command
    metrics.log_summary();
    cli_status
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
