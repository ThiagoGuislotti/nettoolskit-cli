//! Command processor - Main dispatcher for CLI commands

use crate::definitions::{Command, ExitStatus};
use crate::handlers::list::discover_manifests;
use nettoolskit_otel::{Metrics, Timer};
use owo_colors::OwoColorize;
use std::path::PathBuf;
use std::str::FromStr;
use tracing::info;

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
/// Returns `ExitStatus` indicating the result of command execution
pub async fn process_command(cmd: &str) -> ExitStatus {
    let metrics = Metrics::new();
    let timer = Timer::start("command_execution", metrics.clone());

    // Log command usage with structured data
    info!(
        command = %cmd,
        command_type = %cmd.trim_start_matches('/'),
        "Processing CLI command"
    );
    metrics.increment_counter(format!("command_{}_usage", cmd.trim_start_matches('/')));

    // Parse and dispatch command
    let result = match Command::from_str(cmd).ok() {
        Some(Command::List) => {
            println!("{}", "📋 Listing Manifests...".cyan().bold());
            let manifests = discover_manifests(Some(PathBuf::from("."))).await;
            if manifests.is_empty() {
                println!("  {}", "No manifests found in workspace".yellow());
                println!("  Run '/new' to create a new manifest");
            } else {
                println!("  Found {} manifest(s):\n", manifests.len());
                for manifest in manifests {
                    println!("  • {}", manifest.path.display());
                }
            }
            ExitStatus::Success
        }
        Some(Command::New) => {
            println!("{}", "✨ Creating New Manifest...".cyan().bold());
            println!("\n{}", "ℹ️  Manifest creation to be implemented".yellow());
            ExitStatus::Success
        }
        Some(Command::Check) => {
            println!("{}", "🔍 Checking Manifest...".cyan().bold());
            println!("\n{}", "ℹ️  Manifest validation to be implemented".yellow());
            ExitStatus::Success
        }
        Some(Command::Render) => {
            println!("{}", "🎨 Rendering Preview...".cyan().bold());
            println!("\n{}", "ℹ️  Render preview to be implemented".yellow());
            ExitStatus::Success
        }
        Some(Command::Apply) => {
            println!("{}", "⚡ Applying Manifest...".cyan().bold());
            println!("\n{}", "ℹ️  Manifest application to be implemented".yellow());
            ExitStatus::Success
        }
        Some(Command::Quit) => ExitStatus::Success, // Handled by CLI loop
        None => {
            tracing::warn!("Unknown command attempted: {}", cmd);
            metrics.increment_counter("unknown_command_attempts");
            println!("{}: {}", "Unknown command".red(), cmd);
            ExitStatus::Error
        }
    };

    // Stop timer and log result with structured data
    let duration = timer.stop();

    // Log and convert result to CLI status
    let (status_str, counter_name) = match result {
        ExitStatus::Success => ("success", "successful_commands"),
        ExitStatus::Error => ("error", "failed_commands"),
        ExitStatus::Interrupted => ("interrupted", "interrupted_commands"),
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
    result
}

/// Process non-command text input from CLI
///
/// Handles regular text input that is not a slash command, providing
/// basic feedback to the user about their input.
///
/// # Arguments
///
/// * `text` - The input text to process
///
/// # Returns
///
/// Returns `ExitStatus` based on the processing result
pub async fn process_text(text: &str) -> ExitStatus {
    tracing::debug!("Processing text input: {}", text);
    println!("Text processing - to be implemented: {}", text);
    ExitStatus::Success
}
