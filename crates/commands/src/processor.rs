use crate::{CommandRegistry, ExitStatus};
use nettoolskit_manifest::{ExecutionConfig, ManifestExecutor};
use nettoolskit_otel::{Metrics, Timer};
use nettoolskit_ui::PRIMARY_COLOR;
use owo_colors::OwoColorize;
use std::path::PathBuf;
use tracing::{error, info};

/// Process slash commands from CLI and return appropriate status
///
/// This function handles the mapping between CLI slash commands and the actual
/// command implementations, providing telemetry and logging for all operations.
/// It serves as the main dispatcher for interactive CLI commands using dynamic registry.
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

    // Build registry with all available commands
    let registry = build_command_registry();

    // Execute command through registry (dynamic dispatch)
    let result = match registry.execute(cmd, vec![]).await {
        Ok(status) => status,
        Err(_) => {
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

/// Build and populate the command registry with all available commands
///
/// This function creates a new CommandRegistry and registers all slash commands
/// with their corresponding handler functions. Each handler is wrapped to match
/// the registry's signature (accepting Vec<String> args, returning Result<ExitStatus>).
///
/// # Returns
///
/// Returns a fully populated CommandRegistry ready for command execution
fn build_command_registry() -> CommandRegistry {
    let mut registry = CommandRegistry::new();

    // Register all commands with their handlers
    // Note: Handlers are wrapped to match registry signature (args: Vec<String>)
    registry.register("/quit", |_args| async move {
        tracing::info!("User requested quit");
        println!("{}", "👋 Goodbye!".color(PRIMARY_COLOR));
        Ok(ExitStatus::Success)
    });

    registry.register("/list", |_args| async move { Ok(handle_list().await) });

    registry.register("/new", |_args| async move { Ok(handle_new().await) });

    registry.register("/check", |_args| async move { Ok(handle_check().await) });

    registry.register("/render", |_args| async move { Ok(handle_render().await) });

    registry.register("/apply", |_args| async move { Ok(handle_apply().await) });

    registry
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

// ═══════════════════════════════════════════════════════════════
//  Command Handlers - Integration with manifest feature
// ═══════════════════════════════════════════════════════════════

/// Handle /list command - List available manifests
async fn handle_list() -> ExitStatus {
    tracing::debug!("Executing list command");

    println!("{}", "📋 Available Manifests:".color(PRIMARY_COLOR));
    println!("  • Scanning current directory for ntk-manifest.yml files...");

    // TODO: Implement manifest discovery
    // Should scan current directory and subdirectories for manifest files
    // For now, provide placeholder feedback

    println!(
        "\n{}",
        "ℹ️  Manifest discovery to be fully implemented".yellow()
    );
    println!("   Expected: Search for ntk-manifest.yml in current directory");

    ExitStatus::Success
}

/// Handle /new command - Create new manifest interactively
async fn handle_new() -> ExitStatus {
    tracing::debug!("Executing new command");

    println!("{}", "✨ Create New Manifest".color(PRIMARY_COLOR));
    println!("  Interactive manifest creation wizard...");

    // TODO: Implement interactive manifest creation
    // Should guide user through:
    // 1. Manifest kind selection (domain, feature, layer, artifact)
    // 2. Solution/Project name
    // 3. Target language (.NET, Java, Go, Python)
    // 4. Artifacts to generate (entities, repositories, use cases, etc.)

    println!(
        "\n{}",
        "ℹ️  Interactive creation to be fully implemented".yellow()
    );
    println!("   Expected: Step-by-step wizard for manifest creation");

    ExitStatus::Success
}

/// Handle /check command - Validate manifest file
async fn handle_check() -> ExitStatus {
    tracing::debug!("Executing check command");

    println!("{}", "🔍 Checking Manifest...".color(PRIMARY_COLOR));

    // TODO: Implement manifest validation
    // Should:
    // 1. Find manifest file (ntk-manifest.yml in current directory)
    // 2. Parse YAML
    // 3. Validate structure
    // 4. Check for errors and warnings
    // 5. Display results

    let manifest_path = PathBuf::from("ntk-manifest.yml");

    if !manifest_path.exists() {
        error!("Manifest file not found: {}", manifest_path.display());
        println!("{}", "❌ Manifest file not found: ntk-manifest.yml".red());
        println!("   Run '/new' to create a new manifest");
        return ExitStatus::Error;
    }

    println!("  📄 Found: {}", manifest_path.display());
    println!(
        "\n{}",
        "ℹ️  Validation logic to be fully implemented".yellow()
    );
    println!("   Expected: Parse and validate manifest structure");

    ExitStatus::Success
}

/// Handle /render command - Preview template rendering
async fn handle_render() -> ExitStatus {
    tracing::debug!("Executing render command");

    println!("{}", "🎨 Rendering Preview...".color(PRIMARY_COLOR));

    // TODO: Implement render preview
    // Should:
    // 1. Load manifest
    // 2. Build render tasks
    // 3. Render templates (without writing files)
    // 4. Display preview of generated code

    println!(
        "\n{}",
        "ℹ️  Render preview to be fully implemented".yellow()
    );
    println!("   Expected: Show preview of files that would be generated");

    ExitStatus::Success
}

/// Handle /apply command - Apply manifest to generate code
async fn handle_apply() -> ExitStatus {
    tracing::debug!("Executing apply command");

    println!("{}", "⚡ Applying Manifest...".color(PRIMARY_COLOR));

    let manifest_path = PathBuf::from("ntk-manifest.yml");

    if !manifest_path.exists() {
        error!("Manifest file not found: {}", manifest_path.display());
        println!("{}", "❌ Manifest file not found: ntk-manifest.yml".red());
        println!("   Run '/new' to create a new manifest");
        return ExitStatus::Error;
    }

    println!("  📄 Loading manifest: {}", manifest_path.display());

    // Create executor and config
    let executor = ManifestExecutor::new();
    let config = ExecutionConfig {
        manifest_path: manifest_path.clone(),
        output_root: PathBuf::from("."), // Current directory
        dry_run: false,
    };

    // Execute manifest
    match executor.execute(config).await {
        Ok(summary) => {
            println!("\n{}", "✅ Manifest applied successfully!".green());
            println!("  📊 Files created: {}", summary.created.len());
            println!("  📝 Files updated: {}", summary.updated.len());
            println!("  ⏭️  Files skipped: {}", summary.skipped.len());

            if !summary.created.is_empty() {
                println!("\n  Created files:");
                for file in &summary.created {
                    println!("    • {}", file.display());
                }
            }

            if !summary.updated.is_empty() {
                println!("\n  Updated files:");
                for file in &summary.updated {
                    println!("    • {}", file.display());
                }
            }

            ExitStatus::Success
        }
        Err(e) => {
            error!("Failed to apply manifest: {}", e);
            println!("{}", format!("❌ Failed to apply manifest: {}", e).red());
            ExitStatus::Error
        }
    }
}
