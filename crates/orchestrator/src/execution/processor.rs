//! Command processor implementation

use crate::models::{MainAction, ExitStatus};
use nettoolskit_otel::{Metrics, Timer};
use owo_colors::OwoColorize;
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

    // Parse command - pass full command string to get_command
    // It will handle "/ help", "/help", or "help" formats
    let parts: Vec<&str> = cmd.trim().split_whitespace().collect();

    // If command is "/ help" (with space), parts = ["/", "help"], subcommand = parts[2]
    // If command is "/help list", parts = ["/help", "list"], subcommand = parts[1]
    let subcommand = if parts.get(0) == Some(&"/") {
        parts.get(2).copied()
    } else {
        parts.get(1).copied()
    };

    // Parse command using full original string
    let parsed = crate::models::get_main_action(cmd);

    let result = match parsed {
        Some(MainAction::Help) => {
            println!("{}", "� NetToolsKit CLI - Help".cyan().bold());
            println!("\n{}", "Available Commands:".white().bold());
            println!();

            for command in MainAction::iter() {
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
        Some(MainAction::Manifest) => {
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
                    // Parse apply command arguments
                    // Format: /manifest apply <PATH> [--dry-run] [--output DIR]

                    let manifest_path = parts.get(2).map(std::path::PathBuf::from);
                    let dry_run = parts.contains(&"--dry-run");
                    let output_root = if let Some(idx) = parts.iter().position(|&p| p == "--output") {
                        parts.get(idx + 1).map(std::path::PathBuf::from)
                    } else {
                        None
                    };

                    match manifest_path {
                        Some(path) => {
                            // Execute apply handler
                            nettoolskit_manifest::execute_apply(
                                path,
                                output_root,
                                dry_run,
                            ).await
                        }
                        None => {
                            println!("{}", "⚠️  Missing manifest path".red().bold());
                            println!("\n{}", "Usage:".white().bold());
                            println!("  {} <PATH> [--dry-run] [--output DIR]", "/manifest apply".green());
                            println!("\n{}", "Examples:".white().bold());
                            println!("  {} manifest.yaml", "/manifest apply".green());
                            println!("  {} feature.manifest.yaml --dry-run", "/manifest apply".green());
                            println!("  {} domain.manifest.yaml --output ./src", "/manifest apply".green());
                            ExitStatus::Error
                        }
                    }
                }
                None => {
                    // No subcommand provided - show interactive menu from manifest crate
                    info!("Opening manifest interactive menu (no subcommand)");
                    nettoolskit_manifest::show_menu().await
                }
                _ => {
                    println!("{}", "📋 Manifest Commands".cyan().bold());
                    println!("\nAvailable subcommands:");
                    println!("  {} - Discover available manifests in the workspace", "/manifest list".green());
                    println!("  {} - Validate manifest structure and dependencies", "/manifest check".green());
                    println!("  {} - Preview generated files without creating them", "/manifest render".green());
                    println!("  {} - Apply manifest to generate/update project files", "/manifest apply".green());
                    println!("\n{}", "💡 Type a subcommand to continue or just type /manifest for interactive menu".yellow());
                    ExitStatus::Success
                }
            }
        }
        Some(MainAction::Translate) => {
            println!("{}", "🔄 Translate Command".cyan().bold());
            println!("\n{}", "ℹ️  Translation feature is deferred to a future release".yellow());
            ExitStatus::Success
        }
        Some(MainAction::Quit) => ExitStatus::Success, // Handled by CLI loop
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
