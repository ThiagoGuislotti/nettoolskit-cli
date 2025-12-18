//! Command processor implementation

use crate::models::{MainAction, ExitStatus};
use nettoolskit_core::CommandEntry;
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
    let parts: Vec<&str> = cmd.split_whitespace().collect();

    // If command is "/ help" (with space), parts = ["/", "help"], subcommand = parts[2]
    // If command is "/help list", parts = ["/help", "list"], subcommand = parts[1]
    let subcommand = if parts.first().copied() == Some("/") {
        parts.get(2).copied()
    } else {
        parts.get(1).copied()
    };

    // Parse command using full original string
    let parsed = crate::models::get_main_action(cmd);

    let result = match parsed {
        Some(MainAction::Help) => {
            use nettoolskit_ui::Color;
            println!("{}", "� NetToolsKit CLI - Help".color(Color::CYAN).bold());
            println!("\n{}", "Available Commands:".color(Color::WHITE).bold());
            println!();

            for command in MainAction::iter() {
                println!("  {} - {}", command.slash_static().color(Color::GREEN), command.description());
            }

            println!("\n{}", "Usage:".color(Color::WHITE).bold());
            println!("  • Type {} to open the command palette", "/".color(Color::GREEN));
            println!("  • Type a command directly (e.g., {})", "/help".color(Color::GREEN));
            println!("  • Use {} to navigate in the palette", "↑↓".color(Color::CYAN));
            println!("  • Press {} to select a command", "Enter".color(Color::CYAN));

            println!("\n{}", "Examples:".color(Color::WHITE).bold());
            println!("  {} - Show this help", "/help".color(Color::GREEN));
            println!("  {} - Manage manifests", "/manifest".color(Color::GREEN));
            println!("  {} - Exit the CLI", "/quit".color(Color::GREEN));

            ExitStatus::Success
        }
        Some(MainAction::Manifest) => {
            use nettoolskit_ui::Color;
            match subcommand {
                Some("list") => {
                    println!("{}", "📋 Discovering Manifests...".color(Color::CYAN).bold());
                    println!("\n{}", "ℹ️  Manifest discovery will list all available manifest files".color(Color::YELLOW));
                    ExitStatus::Success
                }
                Some("check") => {
                    println!("{}", "✅ Validating Manifest...".color(Color::CYAN).bold());
                    println!("\n{}", "ℹ️  Manifest validation will check structure and dependencies".color(Color::YELLOW));
                    ExitStatus::Success
                }
                Some("render") => {
                    println!("{}", "🎨 Rendering Preview...".color(Color::CYAN).bold());
                    println!("\n{}", "ℹ️  Manifest rendering will preview generated files".color(Color::YELLOW));
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
                            println!("{}", "⚠️  Missing manifest path".color(Color::RED).bold());
                            println!("\n{}", "Usage:".color(Color::WHITE).bold());
                            println!("  {} <PATH> [--dry-run] [--output DIR]", "/manifest apply".color(Color::GREEN));
                            println!("\n{}", "Examples:".color(Color::WHITE).bold());
                            println!("  {} manifest.yaml", "/manifest apply".color(Color::GREEN));
                            println!("  {} feature.manifest.yaml --dry-run", "/manifest apply".color(Color::GREEN));
                            println!("  {} domain.manifest.yaml --output ./src", "/manifest apply".color(Color::GREEN));
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
                    println!("{}", "📋 Manifest Commands".color(Color::CYAN).bold());
                    println!("\nAvailable subcommands:");
                    println!("  {} - Discover available manifests in the workspace", "/manifest list".color(Color::GREEN));
                    println!("  {} - Validate manifest structure and dependencies", "/manifest check".color(Color::GREEN));
                    println!("  {} - Preview generated files without creating them", "/manifest render".color(Color::GREEN));
                    println!("  {} - Apply manifest to generate/update project files", "/manifest apply".color(Color::GREEN));
                    println!("\n{}", "💡 Type a subcommand to continue or just type /manifest for interactive menu".color(Color::YELLOW));
                    ExitStatus::Success
                }
            }
        }
        Some(MainAction::Translate) => {
            use nettoolskit_ui::Color;
            println!("{}", "🔄 Translate Command".color(Color::CYAN).bold());
            println!("\n{}", "ℹ️  Translation feature is deferred to a future release".color(Color::YELLOW));
            ExitStatus::Success
        }
        Some(MainAction::Quit) => ExitStatus::Success, // Handled by CLI loop
        None => {
            use nettoolskit_ui::Color;
            tracing::warn!("Unknown command attempted: {}", cmd);
            metrics.increment_counter("unknown_command_attempts");
            println!("{}: {}", "Unknown command".color(Color::RED), cmd);
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
