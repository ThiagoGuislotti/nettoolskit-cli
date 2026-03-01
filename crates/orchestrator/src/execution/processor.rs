//! Command processor implementation

use crate::models::{ExitStatus, MainAction};
use nettoolskit_core::file_search::{search_files, SearchConfig};
use nettoolskit_core::{AppConfig, ColorMode, CommandEntry, UnicodeMode};
use nettoolskit_otel::{next_correlation_id, Metrics, Timer};
use owo_colors::OwoColorize;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::Duration;
use strum::IntoEnumIterator;
use tracing::{info, info_span};

static RUNTIME_METRICS: OnceLock<Metrics> = OnceLock::new();

fn runtime_metrics() -> &'static Metrics {
    RUNTIME_METRICS.get_or_init(Metrics::new)
}

fn sanitize_metric_component(input: &str) -> String {
    let mut normalized = String::with_capacity(input.len());
    let mut previous_was_separator = false;

    for ch in input.trim().to_ascii_lowercase().chars() {
        if ch.is_ascii_alphanumeric() {
            normalized.push(ch);
            previous_was_separator = false;
        } else if !previous_was_separator {
            normalized.push('_');
            previous_was_separator = true;
        }
    }

    let trimmed = normalized.trim_matches('_');
    if trimmed.is_empty() {
        "unknown".to_string()
    } else {
        trimmed.to_string()
    }
}

fn command_metric_key(parsed: Option<MainAction>, subcommand: Option<&str>, cmd: &str) -> String {
    match parsed {
        Some(MainAction::Manifest) => {
            if let Some(sub) = subcommand {
                format!("manifest_{}", sanitize_metric_component(sub))
            } else {
                "manifest_menu".to_string()
            }
        }
        Some(action) => sanitize_metric_component(action.slash_static().trim_start_matches('/')),
        None => {
            let token = cmd
                .trim()
                .trim_start_matches('/')
                .split_whitespace()
                .next()
                .unwrap_or("unknown");
            format!("unknown_{}", sanitize_metric_component(token))
        }
    }
}

fn record_command_outcome_metrics(
    metrics: &Metrics,
    command_key: &str,
    status: ExitStatus,
) -> &'static str {
    let status_label = match status {
        ExitStatus::Success => "success",
        ExitStatus::Error => "error",
        ExitStatus::Interrupted => "interrupted",
    };

    metrics.increment_counter(format!("runtime_commands_{status_label}_total"));
    metrics.increment_counter(format!(
        "runtime_command_{command_key}_{status_label}_total"
    ));
    status_label
}

fn update_runtime_rate_gauges(metrics: &Metrics) {
    let total = metrics.get_counter("runtime_commands_total");
    if total == 0 {
        return;
    }

    let total_f64 = total as f64;
    let successes = metrics.get_counter("runtime_commands_success_total") as f64;
    let errors = metrics.get_counter("runtime_commands_error_total") as f64;
    let interrupted = metrics.get_counter("runtime_commands_interrupted_total") as f64;

    metrics.set_gauge(
        "runtime_command_success_rate_pct",
        (successes / total_f64) * 100.0,
    );
    metrics.set_gauge(
        "runtime_command_error_rate_pct",
        (errors / total_f64) * 100.0,
    );
    metrics.set_gauge(
        "runtime_command_cancellation_rate_pct",
        (interrupted / total_f64) * 100.0,
    );
}

fn update_runtime_latency_gauges(
    metrics: &Metrics,
    command_key: &str,
    command_timing_name: &str,
    duration: Duration,
) {
    metrics.record_timing("runtime_command_latency_all", duration);
    metrics.set_gauge(
        "runtime_last_command_duration_ms",
        duration.as_secs_f64() * 1000.0,
    );

    if let Some(avg_all) = metrics.get_average_timing("runtime_command_latency_all") {
        metrics.set_gauge(
            "runtime_command_avg_latency_ms",
            avg_all.as_secs_f64() * 1000.0,
        );
    }

    if let Some(avg_cmd) = metrics.get_average_timing(command_timing_name) {
        metrics.set_gauge(
            format!("runtime_command_{command_key}_avg_latency_ms"),
            avg_cmd.as_secs_f64() * 1000.0,
        );
    }
}

fn has_flag(parts: &[&str], flag: &str) -> bool {
    parts.iter().any(|part| *part == flag)
}

fn first_manifest_positional_path(parts: &[&str]) -> Option<PathBuf> {
    parts
        .iter()
        .skip(2)
        .find(|part| !part.starts_with("--"))
        .map(|part| PathBuf::from(*part))
}

fn parse_output_root(parts: &[&str]) -> Option<PathBuf> {
    parts.windows(2).find_map(|window| {
        if window[0] == "--output" && !window[1].starts_with("--") {
            Some(PathBuf::from(window[1]))
        } else {
            None
        }
    })
}

fn discover_manifest_files(root: &Path) -> Result<Vec<PathBuf>, String> {
    let config = SearchConfig {
        include_patterns: vec![
            "*manifest*.yml".to_string(),
            "*manifest*.yaml".to_string(),
            "ntk-*.yml".to_string(),
            "ntk-*.yaml".to_string(),
        ],
        exclude_patterns: vec!["**/target/**".to_string(), "**/node_modules/**".to_string()],
        max_depth: Some(8),
        follow_links: false,
        include_hidden: false,
    };

    let mut manifests =
        search_files(root, &config).map_err(|err| format!("manifest discovery failed: {err}"))?;
    manifests.sort();
    manifests.dedup();
    Ok(manifests)
}

fn relative_path_for_display(root: &Path, path: &Path) -> String {
    match path.strip_prefix(root) {
        Ok(relative) => relative.display().to_string(),
        Err(_) => path.display().to_string(),
    }
}

fn print_manifest_validation(
    path: &Path,
    validation: &nettoolskit_manifest::handlers::ValidationResult,
) {
    use nettoolskit_ui::Color;

    let location_text = path.display().to_string();
    let location = location_text.color(Color::CYAN);
    if validation.is_valid() {
        println!(
            "{} {} {}",
            "✓".color(Color::GREEN),
            location.bold(),
            "is valid".color(Color::GREEN)
        );
    } else {
        println!(
            "{} {} {}",
            "✗".color(Color::RED),
            location.bold(),
            "has validation errors".color(Color::RED)
        );

        for error in &validation.errors {
            if let Some(line) = error.line {
                println!(
                    "  {} [line {}] {}",
                    "error".color(Color::RED),
                    line,
                    error.message
                );
            } else {
                println!("  {} {}", "error".color(Color::RED), error.message);
            }
        }
    }

    if !validation.warnings.is_empty() {
        for warning in &validation.warnings {
            if let Some(line) = warning.line {
                println!(
                    "  {} [line {}] {}",
                    "warning".color(Color::YELLOW),
                    line,
                    warning.message
                );
            } else {
                println!("  {} {}", "warning".color(Color::YELLOW), warning.message);
            }
        }
    }

    println!(
        "  errors: {}, warnings: {}",
        validation.error_count(),
        validation.warning_count()
    );
    println!();
}

fn resolve_manifest_target_path(parts: &[&str], action_label: &str) -> Result<PathBuf, ExitStatus> {
    use nettoolskit_ui::Color;

    if let Some(path) = first_manifest_positional_path(parts) {
        return Ok(path);
    }

    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let manifests = match discover_manifest_files(&cwd) {
        Ok(found) => found,
        Err(err) => {
            println!("{} {}", "✗".color(Color::RED), err.color(Color::RED));
            return Err(ExitStatus::Error);
        }
    };

    match manifests.len() {
        0 => {
            println!(
                "{}",
                format!("No manifest files found for {action_label}.").color(Color::YELLOW)
            );
            println!(
                "{}",
                format!("Provide an explicit path: /manifest {action_label} <manifest-file>")
                    .color(Color::YELLOW)
            );
            Err(ExitStatus::Error)
        }
        1 => Ok(manifests[0].clone()),
        _ => {
            println!(
                "{}",
                format!(
                    "Multiple manifests detected ({}). Specify which one to use:",
                    manifests.len()
                )
                .color(Color::YELLOW)
            );
            for path in manifests {
                println!("  - {}", relative_path_for_display(&cwd, &path));
            }
            println!();
            println!(
                "{}",
                format!(
                    "Use: /manifest {action_label} <manifest-file> [--dry-run] [--output <dir>]"
                )
                .color(Color::YELLOW)
            );
            Err(ExitStatus::Error)
        }
    }
}

fn print_execution_summary(summary: &nettoolskit_manifest::core::models::ExecutionSummary) {
    use nettoolskit_ui::Color;

    if !summary.created.is_empty() {
        println!(
            "{}",
            format!("Files to create: {}", summary.created.len()).color(Color::GREEN)
        );
        for path in &summary.created {
            println!("  + {}", path.display());
        }
        println!();
    }

    if !summary.updated.is_empty() {
        println!(
            "{}",
            format!("Files to update: {}", summary.updated.len()).color(Color::GREEN)
        );
        for path in &summary.updated {
            println!("  ~ {}", path.display());
        }
        println!();
    }

    if !summary.skipped.is_empty() {
        println!(
            "{}",
            format!("Files to skip: {}", summary.skipped.len()).color(Color::YELLOW)
        );
        for (path, reason) in &summary.skipped {
            println!("  - {} ({reason})", path.display());
        }
        println!();
    }

    if !summary.notes.is_empty() {
        println!("{}", "Notes:".color(Color::CYAN));
        for note in &summary.notes {
            println!("  • {note}");
        }
        println!();
    }

    println!(
        "Total artifacts: {}",
        summary.created.len() + summary.updated.len()
    );
}

fn parse_translate_request(
    parts: &[&str],
) -> Result<nettoolskit_translate::TranslateRequest, String> {
    let mut from: Option<String> = None;
    let mut to: Option<String> = None;
    let mut path: Option<String> = None;
    let mut index = 1;

    while index < parts.len() {
        match parts[index] {
            "--from" => {
                let value = parts
                    .get(index + 1)
                    .ok_or_else(|| "missing value for --from".to_string())?;
                if value.starts_with("--") {
                    return Err("missing value for --from".to_string());
                }
                from = Some((*value).to_string());
                index += 2;
            }
            "--to" => {
                let value = parts
                    .get(index + 1)
                    .ok_or_else(|| "missing value for --to".to_string())?;
                if value.starts_with("--") {
                    return Err("missing value for --to".to_string());
                }
                to = Some((*value).to_string());
                index += 2;
            }
            other if other.starts_with("--") => {
                return Err(format!("unknown flag '{other}'"));
            }
            positional => {
                if path.is_some() {
                    return Err(format!(
                        "unexpected positional argument '{positional}' (only one template path is allowed)"
                    ));
                }
                path = Some(positional.to_string());
                index += 1;
            }
        }
    }

    let from = from.ok_or_else(|| "missing --from <language>".to_string())?;
    let to = to.ok_or_else(|| "missing --to <language>".to_string())?;
    let path = path.ok_or_else(|| "missing template path".to_string())?;

    Ok(nettoolskit_translate::TranslateRequest { from, to, path })
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
/// Returns `ExitStatus` indicating the result of command execution
pub async fn process_command(cmd: &str) -> ExitStatus {
    let correlation_id = next_correlation_id("cmd");
    let execution_span =
        info_span!("orchestrator.command", correlation_id = %correlation_id, command = %cmd);
    let _execution_scope = execution_span.enter();

    let metrics = runtime_metrics().clone();

    // Log command usage with structured data
    info!(
        correlation_id = %correlation_id,
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
    let command_key = command_metric_key(parsed, subcommand, cmd);
    let command_timing_name = format!("runtime_command_latency_{command_key}");
    let timer = Timer::start(command_timing_name.clone(), metrics.clone());

    metrics.increment_counter("runtime_commands_total");
    metrics.increment_counter(format!("runtime_command_{command_key}_total"));

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
                    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
                    println!("{}", "📋 Manifest Discovery".color(Color::CYAN).bold());
                    println!("Root: {}", cwd.display().to_string().color(Color::WHITE));
                    println!();

                    match discover_manifest_files(&cwd) {
                        Ok(found) if found.is_empty() => {
                            println!("{}", "No manifest files found.".color(Color::YELLOW));
                            ExitStatus::Error
                        }
                        Ok(found) => {
                            println!(
                                "{}",
                                format!("Found {} manifest file(s):", found.len())
                                    .color(Color::GREEN)
                            );
                            for path in found {
                                println!("  - {}", relative_path_for_display(&cwd, &path));
                            }
                            ExitStatus::Success
                        }
                        Err(err) => {
                            println!("{} {err}", "✗ Discovery failed:".color(Color::RED).bold());
                            ExitStatus::Error
                        }
                    }
                }
                Some("check") => {
                    let is_template = has_flag(&parts, "--template");
                    if is_template && first_manifest_positional_path(&parts).is_none() {
                        println!(
                            "{}",
                            "Template validation requires a file path: /manifest check <file> --template"
                                .color(Color::RED)
                                .bold()
                        );
                        ExitStatus::Error
                    } else {
                        match resolve_manifest_target_path(&parts, "check") {
                            Ok(target_path) => {
                                println!("{}", "✅ Validating Manifest".color(Color::CYAN).bold());
                                println!("Target: {}", target_path.display());
                                println!();

                                match nettoolskit_manifest::handlers::check::check_file(
                                    &target_path,
                                    is_template,
                                )
                                .await
                                {
                                    Ok(validation) => {
                                        print_manifest_validation(&target_path, &validation);
                                        if validation.is_valid() {
                                            ExitStatus::Success
                                        } else {
                                            ExitStatus::Error
                                        }
                                    }
                                    Err(err) => {
                                        println!(
                                            "{} {err}",
                                            "✗ Manifest validation failed:"
                                                .color(Color::RED)
                                                .bold()
                                        );
                                        ExitStatus::Error
                                    }
                                }
                            }
                            Err(status) => status,
                        }
                    }
                }
                Some("render") => match resolve_manifest_target_path(&parts, "render") {
                    Ok(manifest_path) => {
                        let output_root = parse_output_root(&parts).unwrap_or_else(|| {
                            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
                        });
                        let dry_run = true;

                        println!("{}", "🎨 Rendering Preview".color(Color::CYAN).bold());
                        println!("Manifest: {}", manifest_path.display());
                        println!("Output root: {}", output_root.display());
                        println!(
                            "{}",
                            "DRY-RUN mode enabled (preview only)".color(Color::YELLOW)
                        );
                        println!();

                        let config = nettoolskit_manifest::ExecutionConfig {
                            manifest_path,
                            output_root,
                            dry_run,
                        };

                        let executor = nettoolskit_manifest::ManifestExecutor::new();
                        match executor.execute(config).await {
                            Ok(summary) => {
                                println!("{}", "✓ Render preview completed".color(Color::GREEN));
                                println!();
                                print_execution_summary(&summary);
                                ExitStatus::Success
                            }
                            Err(err) => {
                                println!(
                                    "{} {err}",
                                    "✗ Render preview failed:".color(Color::RED).bold()
                                );
                                ExitStatus::Error
                            }
                        }
                    }
                    Err(status) => status,
                },
                Some("apply") => {
                    // Parse apply command arguments
                    // Format: /manifest apply <PATH> [--dry-run] [--output DIR]

                    let manifest_path = first_manifest_positional_path(&parts);
                    let dry_run = has_flag(&parts, "--dry-run");
                    let output_root = parse_output_root(&parts);

                    match manifest_path {
                        Some(path) => {
                            // Execute apply handler
                            nettoolskit_manifest::execute_apply(path, output_root, dry_run).await
                        }
                        None => {
                            info!(
                                "Manifest apply called without path; opening interactive apply flow"
                            );
                            nettoolskit_manifest::show_apply_menu().await
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
            match parse_translate_request(&parts) {
                Ok(request) => nettoolskit_translate::handle_translate(request).await,
                Err(reason) => {
                    println!("{}", "🔄 Translate Command".color(Color::CYAN).bold());
                    println!();
                    println!(
                        "{} {reason}",
                        "✗ Invalid translate arguments:".color(Color::RED).bold()
                    );
                    println!();
                    println!("{}", "Usage:".color(Color::WHITE).bold());
                    println!(
                        "  {} --from <language> --to <language> <template-path>",
                        "/translate".color(Color::GREEN)
                    );
                    println!();
                    println!("{}", "Examples:".color(Color::WHITE).bold());
                    println!(
                        "  {} --from dotnet --to rust ./templates/entity.cs.hbs",
                        "/translate".color(Color::GREEN)
                    );
                    println!(
                        "  {} --from python --to java ./templates/service.py.hbs",
                        "/translate".color(Color::GREEN)
                    );
                    println!();
                    println!(
                        "{}",
                        "Supported languages: dotnet, java, go, python, rust, clojure, typescript"
                            .color(Color::YELLOW)
                    );
                    println!(
                        "{}",
                        "Note: targets clojure/typescript are currently beta-limited."
                            .color(Color::YELLOW)
                    );
                    ExitStatus::Error
                }
            }
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
    update_runtime_latency_gauges(&metrics, &command_key, &command_timing_name, duration);

    // Log and convert result to CLI status
    let counter_name = match result {
        ExitStatus::Success => "successful_commands",
        ExitStatus::Error => "failed_commands",
        ExitStatus::Interrupted => "interrupted_commands",
    };
    let status_str = record_command_outcome_metrics(&metrics, &command_key, result);
    update_runtime_rate_gauges(&metrics);

    info!(
        correlation_id = %correlation_id,
        command = %cmd,
        command_key = %command_key,
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

fn infer_command_from_text(text: &str) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }

    if trimmed.starts_with('/') {
        return Some(trimmed.to_string());
    }

    let tokens: Vec<&str> = trimmed.split_whitespace().collect();
    if tokens.is_empty() {
        return None;
    }

    let first = tokens[0].to_ascii_lowercase();
    let lowered: Vec<String> = tokens
        .iter()
        .map(|token| token.to_ascii_lowercase())
        .collect();
    let has = |keyword: &str| lowered.iter().any(|token| token == keyword);

    match first.as_str() {
        // direct command aliases (without slash)
        "help" | "ajuda" => Some("/help".to_string()),
        "manifest" | "manifests" => Some(format!("/{}", trimmed)),
        "translate" => Some(format!("/{}", trimmed)),
        "traduzir" => {
            if tokens.len() > 1 {
                Some(format!("/translate {}", tokens[1..].join(" ")))
            } else {
                Some("/translate".to_string())
            }
        }
        "config" | "configuracao" | "configuração" | "settings" => Some(format!("/{}", trimmed)),
        _ => {
            // keyword-based intent routing
            if has("help") || has("ajuda") {
                Some("/help".to_string())
            } else if has("manifest") || has("manifests") || has("manifests:") {
                if has("list") || has("listar") || has("discover") {
                    Some("/manifest list".to_string())
                } else if has("check") || has("validate") || has("validar") {
                    Some("/manifest check".to_string())
                } else if has("render") || has("preview") || has("previa") || has("prévia") {
                    Some("/manifest render".to_string())
                } else if has("apply") || has("aplicar") {
                    Some("/manifest apply".to_string())
                } else {
                    Some("/manifest".to_string())
                }
            } else if has("config") || has("configuracao") || has("configuração") || has("settings")
            {
                Some("/config".to_string())
            } else {
                None
            }
        }
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
    let metrics = runtime_metrics().clone();
    metrics.increment_counter("runtime_text_inputs_total");

    let trimmed = text.trim();

    // Silently ignore empty or whitespace-only input
    if trimmed.is_empty() {
        metrics.increment_counter("runtime_text_inputs_ignored_total");
        tracing::trace!("Empty text input ignored");
        return ExitStatus::Success;
    }

    if let Some(resolved_command) = infer_command_from_text(trimmed) {
        metrics.increment_counter("runtime_text_inputs_routed_total");
        tracing::info!(
            input = %trimmed,
            resolved_command = %resolved_command,
            "Routed free-text input to command"
        );
        return process_command(&resolved_command).await;
    }

    metrics.increment_counter("runtime_text_inputs_unrecognized_total");
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
    fn sanitize_metric_component_normalizes_symbols() {
        assert_eq!(
            sanitize_metric_component(" Manifest Apply "),
            "manifest_apply"
        );
        assert_eq!(sanitize_metric_component("/x-y.z"), "x_y_z");
        assert_eq!(sanitize_metric_component("___"), "unknown");
    }

    #[test]
    fn command_metric_key_resolves_manifest_subcommands() {
        let key = command_metric_key(
            Some(MainAction::Manifest),
            Some("apply"),
            "/manifest apply a.yaml",
        );
        assert_eq!(key, "manifest_apply");

        let menu_key = command_metric_key(Some(MainAction::Manifest), None, "/manifest");
        assert_eq!(menu_key, "manifest_menu");
    }

    #[test]
    fn command_metric_key_resolves_unknown_commands() {
        let key = command_metric_key(None, None, "/custom-op --x");
        assert_eq!(key, "unknown_custom_op");
    }

    #[test]
    fn update_runtime_rate_gauges_computes_percentages() {
        let metrics = Metrics::new();
        metrics.increment_counter("runtime_commands_total");
        metrics.increment_counter("runtime_commands_total");
        metrics.increment_counter("runtime_commands_total");
        metrics.increment_counter("runtime_commands_total");
        metrics.increment_counter("runtime_commands_success_total");
        metrics.increment_counter("runtime_commands_success_total");
        metrics.increment_counter("runtime_commands_error_total");
        metrics.increment_counter("runtime_commands_interrupted_total");

        update_runtime_rate_gauges(&metrics);

        assert_eq!(
            metrics.get_gauge("runtime_command_success_rate_pct"),
            Some(50.0)
        );
        assert_eq!(
            metrics.get_gauge("runtime_command_error_rate_pct"),
            Some(25.0)
        );
        assert_eq!(
            metrics.get_gauge("runtime_command_cancellation_rate_pct"),
            Some(25.0)
        );
    }

    #[test]
    fn record_command_outcome_metrics_updates_counters() {
        let metrics = Metrics::new();
        let label = record_command_outcome_metrics(&metrics, "help", ExitStatus::Error);
        assert_eq!(label, "error");
        assert_eq!(metrics.get_counter("runtime_commands_error_total"), 1);
        assert_eq!(metrics.get_counter("runtime_command_help_error_total"), 1);
    }

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

    #[test]
    fn infer_command_from_text_routes_direct_aliases() {
        assert_eq!(infer_command_from_text("help").as_deref(), Some("/help"));
        assert_eq!(infer_command_from_text("ajuda").as_deref(), Some("/help"));
        assert_eq!(
            infer_command_from_text("manifest check sample.yaml").as_deref(),
            Some("/manifest check sample.yaml")
        );
        assert_eq!(
            infer_command_from_text("translate --from dotnet --to rust a.cs.hbs").as_deref(),
            Some("/translate --from dotnet --to rust a.cs.hbs")
        );
    }

    #[test]
    fn infer_command_from_text_routes_keyword_intents() {
        assert_eq!(
            infer_command_from_text("listar manifests").as_deref(),
            Some("/manifest list")
        );
        assert_eq!(
            infer_command_from_text("please validate manifest").as_deref(),
            Some("/manifest check")
        );
        assert_eq!(
            infer_command_from_text("quero preview do manifest").as_deref(),
            Some("/manifest render")
        );
        assert_eq!(
            infer_command_from_text("open settings").as_deref(),
            Some("/config")
        );
    }

    #[test]
    fn infer_command_from_text_returns_none_for_unrelated_text() {
        assert_eq!(infer_command_from_text(""), None);
        assert_eq!(infer_command_from_text("   "), None);
        assert_eq!(infer_command_from_text("hello world"), None);
    }

    #[test]
    fn first_manifest_positional_path_extracts_apply_path() {
        let parts = vec!["/manifest", "apply", "feature.manifest.yaml", "--dry-run"];
        let path = first_manifest_positional_path(&parts);
        assert_eq!(
            path.as_deref(),
            Some(std::path::Path::new("feature.manifest.yaml"))
        );
    }

    #[test]
    fn parse_output_root_extracts_output_value() {
        let parts = vec![
            "/manifest",
            "apply",
            "feature.manifest.yaml",
            "--output",
            "./src",
        ];
        let output = parse_output_root(&parts);
        assert_eq!(output.as_deref(), Some(std::path::Path::new("./src")));
    }
}
