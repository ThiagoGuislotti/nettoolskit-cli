//! Command processor - Main dispatcher for CLI commands

use crate::definitions::{Command, ExitStatus};
use nettoolskit_otel::{Metrics, Timer};
use owo_colors::OwoColorize;
use std::str::FromStr;
use strum::IntoEnumIterator;
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

    // Parse command and potential subcommand
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    let base_cmd = parts.get(0).unwrap_or(&"");
    let subcommand = parts.get(1).copied();

    // Parse and dispatch command
    let result = match Command::from_str(base_cmd).ok() {
        Some(Command::Help) => {
            println!("{}", "� NetToolsKit CLI - Help".cyan().bold());
            println!("\n{}", "Available Commands:".white().bold());
            println!();

            for command in Command::iter() {
                println!("  {} - {}", command.slash().green(), command.description());
            }

            println!("\n{}", "Usage:".white().bold());
            println!("  • Type {} to open the command palette", "/".green());
            println!("  • Type a command directly (e.g., {})", "/help".green());
            println!("  • Use {} to navigate in the palette", "↑↓".cyan());
            println!("  • Press {} to select a command", "Enter".cyan());

            println!("\n{}", "Examples:".white().bold());
            println!("  {} - Show this help", "/help".green());
            println!("  {} - Manage manifests", "/manifest".green());
            println!("  {} - Exit the CLI", "/quit".green());

            ExitStatus::Success
        }
        Some(Command::Manifest) => {
            match subcommand {
                Some("list") => {
                    println!("{}", "📋 Discovering Manifests...".cyan().bold());
                    println!("\n{}", "ℹ️  Manifest discovery will list all available manifest files".yellow());
                    ExitStatus::Success
                }
                Some("check") => {
                    println!("{}", "✅ Validating Manifest...".cyan().bold());
                    println!("\n{}", "ℹ️  Manifest validation will check structure and dependencies".yellow());
                    ExitStatus::Success
                }
                Some("render") => {
                    println!("{}", "🎨 Rendering Preview...".cyan().bold());
                    println!("\n{}", "ℹ️  Manifest rendering will preview generated files".yellow());
                    ExitStatus::Success
                }
                Some("apply") => {
                    println!("{}", "⚡ Applying Manifest...".cyan().bold());
                    println!("\n{}", "ℹ️  Manifest application will generate/update project files".yellow());
                    ExitStatus::Success
                }
                _ => {
                    println!("{}", "📋 Manifest Commands".cyan().bold());
                    println!("\nAvailable subcommands:");
                    println!("  {} - Discover available manifests in the workspace", "/manifest list".green());
                    println!("  {} - Validate manifest structure and dependencies", "/manifest check".green());
                    println!("  {} - Preview generated files without creating them", "/manifest render".green());
                    println!("  {} - Apply manifest to generate/update project files", "/manifest apply".green());
                    println!("\n{}", "💡 Type a subcommand to continue".yellow());
                    ExitStatus::Success
                }
            }
        }
        Some(Command::Translate) => {
            println!("{}", "🔄 Translate Command".cyan().bold());
            println!("\n{}", "ℹ️  Translation feature is deferred to a future release".yellow());
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
