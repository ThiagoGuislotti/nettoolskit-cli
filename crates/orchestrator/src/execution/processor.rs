//! Command processor implementation

use crate::models::{ExitStatus, MainAction};
use nettoolskit_core::{AppConfig, ColorMode, CommandEntry, UnicodeMode};
use nettoolskit_otel::{Metrics, Timer};
use owo_colors::OwoColorize;
use std::path::Path;
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
                println!(
                    "  {} - {}",
                    command.slash_static().color(Color::GREEN),
                    command.description()
                );
            }

            println!("\n{}", "Usage:".color(Color::WHITE).bold());
            println!(
                "  • Type {} to open the command palette",
                "/".color(Color::GREEN)
            );
            println!(
                "  • Type a command directly (e.g., {})",
                "/help".color(Color::GREEN)
            );
            println!(
                "  • Use {} to navigate in the palette",
                "↑↓".color(Color::CYAN)
            );
            println!(
                "  • Press {} to select a command",
                "Enter".color(Color::CYAN)
            );

            println!("\n{}", "Examples:".color(Color::WHITE).bold());
            println!("  {} - Show this help", "/help".color(Color::GREEN));
            println!("  {} - Manage manifests", "/manifest".color(Color::GREEN));
            println!(
                "  {} - View or edit configuration",
                "/config".color(Color::GREEN)
            );
            println!("  {} - Exit the CLI", "/quit".color(Color::GREEN));

            ExitStatus::Success
        }
        Some(MainAction::Manifest) => {
            use nettoolskit_ui::Color;
            match subcommand {
                Some("list") => {
                    println!(
                        "{}",
                        "📋 Discovering Manifests...".color(Color::CYAN).bold()
                    );
                    println!(
                        "\n{}",
                        "ℹ️  Manifest discovery will list all available manifest files"
                            .color(Color::YELLOW)
                    );
                    ExitStatus::Success
                }
                Some("check") => {
                    println!("{}", "✅ Validating Manifest...".color(Color::CYAN).bold());
                    println!(
                        "\n{}",
                        "ℹ️  Manifest validation will check structure and dependencies"
                            .color(Color::YELLOW)
                    );
                    ExitStatus::Success
                }
                Some("render") => {
                    println!("{}", "🎨 Rendering Preview...".color(Color::CYAN).bold());
                    println!(
                        "\n{}",
                        "ℹ️  Manifest rendering will preview generated files".color(Color::YELLOW)
                    );
                    ExitStatus::Success
                }
                Some("apply") => {
                    // Parse apply command arguments
                    // Format: /manifest apply <PATH> [--dry-run] [--output DIR]

                    let manifest_path = parts.get(2).map(std::path::PathBuf::from);
                    let dry_run = parts.contains(&"--dry-run");
                    let output_root = if let Some(idx) = parts.iter().position(|&p| p == "--output")
                    {
                        parts.get(idx + 1).map(std::path::PathBuf::from)
                    } else {
                        None
                    };

                    match manifest_path {
                        Some(path) => {
                            // Execute apply handler
                            nettoolskit_manifest::execute_apply(path, output_root, dry_run).await
                        }
                        None => {
                            println!("{}", "⚠️  Missing manifest path".color(Color::RED).bold());
                            println!("\n{}", "Usage:".color(Color::WHITE).bold());
                            println!(
                                "  {} <PATH> [--dry-run] [--output DIR]",
                                "/manifest apply".color(Color::GREEN)
                            );
                            println!("\n{}", "Examples:".color(Color::WHITE).bold());
                            println!("  {} manifest.yaml", "/manifest apply".color(Color::GREEN));
                            println!(
                                "  {} feature.manifest.yaml --dry-run",
                                "/manifest apply".color(Color::GREEN)
                            );
                            println!(
                                "  {} domain.manifest.yaml --output ./src",
                                "/manifest apply".color(Color::GREEN)
                            );
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
                    println!(
                        "  {} - Discover available manifests in the workspace",
                        "/manifest list".color(Color::GREEN)
                    );
                    println!(
                        "  {} - Validate manifest structure and dependencies",
                        "/manifest check".color(Color::GREEN)
                    );
                    println!(
                        "  {} - Preview generated files without creating them",
                        "/manifest render".color(Color::GREEN)
                    );
                    println!(
                        "  {} - Apply manifest to generate/update project files",
                        "/manifest apply".color(Color::GREEN)
                    );
                    println!("\n{}", "💡 Type a subcommand to continue or just type /manifest for interactive menu".color(Color::YELLOW));
                    ExitStatus::Success
                }
            }
        }
        Some(MainAction::Translate) => {
            use nettoolskit_ui::Color;
            println!("{}", "🔄 Translate Command".color(Color::CYAN).bold());
            println!(
                "\n{}",
                "ℹ️  Translation feature is deferred to a future release".color(Color::YELLOW)
            );
            ExitStatus::Success
        }
        Some(MainAction::Config) => process_config_command(&parts),
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

fn process_config_command(parts: &[&str]) -> ExitStatus {
    use nettoolskit_ui::Color;

    let Some(config_path) = AppConfig::default_config_path() else {
        println!(
            "{}",
            "Could not determine configuration path on this system."
                .color(Color::RED)
                .bold()
        );
        return ExitStatus::Error;
    };

    match parts.get(1).copied() {
        None | Some("show") => {
            print_effective_config(&config_path);
            ExitStatus::Success
        }
        Some("path") => {
            println!("{}", "📍 Configuration Path".color(Color::CYAN).bold());
            println!(
                "  {}",
                config_path.display().to_string().color(Color::GREEN)
            );
            println!(
                "  Exists: {}",
                if config_path.exists() {
                    "yes".color(Color::GREEN)
                } else {
                    "no".color(Color::YELLOW)
                }
            );
            ExitStatus::Success
        }
        Some("init") => {
            if config_path.exists() {
                println!(
                    "{}",
                    "Configuration file already exists."
                        .color(Color::YELLOW)
                        .bold()
                );
                println!("  {}", config_path.display());
                return ExitStatus::Success;
            }

            let config = AppConfig::default();
            match config.save_to(&config_path) {
                Ok(()) => {
                    println!(
                        "{}",
                        "✅ Configuration initialized".color(Color::GREEN).bold()
                    );
                    println!("  {}", config_path.display());
                    ExitStatus::Success
                }
                Err(err) => {
                    println!(
                        "{}: {}",
                        "Failed to initialize config".color(Color::RED),
                        err
                    );
                    ExitStatus::Error
                }
            }
        }
        Some("set") => {
            if parts.len() < 4 {
                print_config_usage();
                return ExitStatus::Error;
            }

            let key = parts[2];
            let value = parts[3..].join(" ");
            let mut config = load_persisted_or_default(&config_path);

            match set_config_value(&mut config, key, &value) {
                Ok(()) => match config.save_to(&config_path) {
                    Ok(()) => {
                        println!(
                            "{} {}={}",
                            "✅ Updated".color(Color::GREEN).bold(),
                            key.color(Color::CYAN),
                            value.color(Color::WHITE)
                        );
                        println!("  {}", config_path.display());
                        ExitStatus::Success
                    }
                    Err(err) => {
                        println!("{}: {}", "Failed to save config".color(Color::RED), err);
                        ExitStatus::Error
                    }
                },
                Err(err) => {
                    println!("{}: {}", "Invalid config value".color(Color::RED), err);
                    print_supported_config_keys();
                    ExitStatus::Error
                }
            }
        }
        Some("unset") => {
            if parts.len() < 3 {
                print_config_usage();
                return ExitStatus::Error;
            }

            let key = parts[2];
            let mut config = load_persisted_or_default(&config_path);

            match unset_config_value(&mut config, key) {
                Ok(()) => match config.save_to(&config_path) {
                    Ok(()) => {
                        println!(
                            "{} {}",
                            "✅ Reset".color(Color::GREEN).bold(),
                            key.color(Color::CYAN)
                        );
                        println!("  {}", config_path.display());
                        ExitStatus::Success
                    }
                    Err(err) => {
                        println!("{}: {}", "Failed to save config".color(Color::RED), err);
                        ExitStatus::Error
                    }
                },
                Err(err) => {
                    println!("{}: {}", "Invalid config key".color(Color::RED), err);
                    print_supported_config_keys();
                    ExitStatus::Error
                }
            }
        }
        Some("reset") => {
            let config = AppConfig::default();
            match config.save_to(&config_path) {
                Ok(()) => {
                    println!(
                        "{}",
                        "✅ Configuration reset to defaults"
                            .color(Color::GREEN)
                            .bold()
                    );
                    println!("  {}", config_path.display());
                    ExitStatus::Success
                }
                Err(err) => {
                    println!("{}: {}", "Failed to reset config".color(Color::RED), err);
                    ExitStatus::Error
                }
            }
        }
        Some("help") => {
            print_config_usage();
            ExitStatus::Success
        }
        Some(_) => {
            println!("{}", "Unknown /config subcommand".color(Color::RED).bold());
            print_config_usage();
            ExitStatus::Error
        }
    }
}

fn print_effective_config(config_path: &Path) {
    use nettoolskit_ui::Color;

    let effective = AppConfig::load();
    println!(
        "{}",
        "⚙️  NetToolsKit Configuration".color(Color::CYAN).bold()
    );
    println!(
        "  File: {} ({})",
        config_path.display().to_string().color(Color::GREEN),
        if config_path.exists() {
            "exists".color(Color::GREEN)
        } else {
            "not found, using defaults/env".color(Color::YELLOW)
        }
    );
    println!();
    println!("{}", "[general]".color(Color::WHITE).bold());
    println!("  verbose = {}", effective.general.verbose);
    println!("  log_level = {}", effective.general.log_level);
    println!("{}", "[display]".color(Color::WHITE).bold());
    println!("  color = {}", effective.display.color);
    println!("  unicode = {}", effective.display.unicode);
    println!("{}", "[templates]".color(Color::WHITE).bold());
    println!(
        "  directory = {}",
        effective
            .templates
            .directory
            .as_deref()
            .unwrap_or("(default)")
    );
    println!("{}", "[shell]".color(Color::WHITE).bold());
    println!(
        "  default_shell = {}",
        effective.shell.default_shell.as_deref().unwrap_or("(none)")
    );
    println!();
    print_config_usage();
}

fn print_supported_config_keys() {
    use nettoolskit_ui::Color;
    println!("{}", "Supported keys:".color(Color::WHITE).bold());
    println!("  {}", "verbose".color(Color::CYAN));
    println!("  {}", "log_level".color(Color::CYAN));
    println!("  {}", "color".color(Color::CYAN));
    println!("  {}", "unicode".color(Color::CYAN));
    println!("  {}", "template_dir".color(Color::CYAN));
    println!("  {}", "shell".color(Color::CYAN));
}

fn print_config_usage() {
    use nettoolskit_ui::Color;
    println!("{}", "Usage:".color(Color::WHITE).bold());
    println!("  {}", "/config [show]".color(Color::GREEN));
    println!("  {}", "/config path".color(Color::GREEN));
    println!("  {}", "/config init".color(Color::GREEN));
    println!("  {}", "/config set <key> <value>".color(Color::GREEN));
    println!("  {}", "/config unset <key>".color(Color::GREEN));
    println!("  {}", "/config reset".color(Color::GREEN));
    println!();
    print_supported_config_keys();
}

fn load_persisted_or_default(config_path: &Path) -> AppConfig {
    if config_path.exists() {
        AppConfig::load_from(config_path).unwrap_or_default()
    } else {
        AppConfig::default()
    }
}

fn set_config_value(config: &mut AppConfig, key: &str, value: &str) -> Result<(), String> {
    let normalized = key.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "verbose" | "general.verbose" => {
            let parsed = parse_bool(value).ok_or_else(|| {
                "verbose must be one of: true, false, 1, 0, yes, no, on, off".to_string()
            })?;
            config.general.verbose = parsed;
            Ok(())
        }
        "log_level" | "log-level" | "general.log_level" | "general.log-level" => {
            config.general.log_level = value.trim().to_string();
            Ok(())
        }
        "color" | "display.color" => {
            let parsed = parse_color_mode(value)?;
            config.display.color = parsed;
            Ok(())
        }
        "unicode" | "display.unicode" => {
            let parsed = parse_unicode_mode(value)?;
            config.display.unicode = parsed;
            Ok(())
        }
        "template_dir" | "template-dir" | "templates.directory" => {
            config.templates.directory = Some(value.trim().to_string());
            Ok(())
        }
        "shell" | "default_shell" | "default-shell" | "shell.default_shell" => {
            config.shell.default_shell = Some(value.trim().to_string());
            Ok(())
        }
        _ => Err(format!("unsupported key '{key}'")),
    }
}

fn unset_config_value(config: &mut AppConfig, key: &str) -> Result<(), String> {
    let normalized = key.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "verbose" | "general.verbose" => {
            config.general.verbose = false;
            Ok(())
        }
        "log_level" | "log-level" | "general.log_level" | "general.log-level" => {
            config.general.log_level = "info".to_string();
            Ok(())
        }
        "color" | "display.color" => {
            config.display.color = ColorMode::Auto;
            Ok(())
        }
        "unicode" | "display.unicode" => {
            config.display.unicode = UnicodeMode::Auto;
            Ok(())
        }
        "template_dir" | "template-dir" | "templates.directory" => {
            config.templates.directory = None;
            Ok(())
        }
        "shell" | "default_shell" | "default-shell" | "shell.default_shell" => {
            config.shell.default_shell = None;
            Ok(())
        }
        _ => Err(format!("unsupported key '{key}'")),
    }
}

fn parse_bool(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn parse_color_mode(value: &str) -> Result<ColorMode, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "auto" => Ok(ColorMode::Auto),
        "always" => Ok(ColorMode::Always),
        "never" => Ok(ColorMode::Never),
        _ => Err("color must be one of: auto, always, never".to_string()),
    }
}

fn parse_unicode_mode(value: &str) -> Result<UnicodeMode, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "auto" => Ok(UnicodeMode::Auto),
        "always" => Ok(UnicodeMode::Always),
        "never" => Ok(UnicodeMode::Never),
        _ => Err("unicode must be one of: auto, always, never".to_string()),
    }
}

/// Process non-command text input from CLI
///
/// Handles regular text input that is not a slash command.
/// Since NetToolsKit CLI is a command-driven tool, free-text input
/// is treated as unrecognized and the user is guided toward
/// available slash commands.
///
/// # Arguments
///
/// * `text` - The input text to process
///
/// # Returns
///
/// * `ExitStatus::Success` for empty/whitespace-only input (silently ignored)
/// * `ExitStatus::Success` for non-empty text (hint displayed)
pub async fn process_text(text: &str) -> ExitStatus {
    let trimmed = text.trim();

    // Silently ignore empty or whitespace-only input
    if trimmed.is_empty() {
        tracing::trace!("Empty text input ignored");
        return ExitStatus::Success;
    }

    tracing::debug!(input = %trimmed, "Unrecognized text input");

    use nettoolskit_ui::Color;
    use owo_colors::OwoColorize;

    println!(
        "{}: {}",
        "Unrecognized input".color(Color::YELLOW),
        trimmed.color(Color::GRAY)
    );
    println!(
        "  {} Type {} to see available commands, or {} to open the palette.",
        "💡".color(Color::CYAN),
        "/help".color(Color::GREEN),
        "/".color(Color::GREEN)
    );

    ExitStatus::Success
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_config_value_updates_known_keys() {
        let mut config = AppConfig::default();
        assert!(set_config_value(&mut config, "verbose", "true").is_ok());
        assert!(config.general.verbose);

        assert!(set_config_value(&mut config, "color", "never").is_ok());
        assert_eq!(config.display.color, ColorMode::Never);

        assert!(set_config_value(&mut config, "template_dir", "/tmp/x").is_ok());
        assert_eq!(config.templates.directory.as_deref(), Some("/tmp/x"));
    }

    #[test]
    fn set_config_value_rejects_unknown_key() {
        let mut config = AppConfig::default();
        let result = set_config_value(&mut config, "unknown", "x");
        assert!(result.is_err());
    }

    #[test]
    fn unset_config_value_resets_known_keys() {
        let mut config = AppConfig::default();
        config.general.verbose = true;
        config.display.color = ColorMode::Always;
        config.templates.directory = Some("/tmp/x".to_string());

        assert!(unset_config_value(&mut config, "verbose").is_ok());
        assert!(unset_config_value(&mut config, "color").is_ok());
        assert!(unset_config_value(&mut config, "template_dir").is_ok());

        assert!(!config.general.verbose);
        assert_eq!(config.display.color, ColorMode::Auto);
        assert_eq!(config.templates.directory, None);
    }

    #[test]
    fn parse_bool_handles_supported_values() {
        assert_eq!(parse_bool("true"), Some(true));
        assert_eq!(parse_bool("1"), Some(true));
        assert_eq!(parse_bool("on"), Some(true));
        assert_eq!(parse_bool("false"), Some(false));
        assert_eq!(parse_bool("0"), Some(false));
        assert_eq!(parse_bool("off"), Some(false));
        assert_eq!(parse_bool("maybe"), None);
    }
}
