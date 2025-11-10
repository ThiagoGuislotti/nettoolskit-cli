//! List command implementation for NetToolsKit CLI.
//!
//! This module handles listing available templates from the registry,
//! with support for filtering by name patterns and technology stacks.

use crate::ExitStatus;
use clap::Parser;
use owo_colors::OwoColorize;
use tracing::info;

/// Arguments for the list command.
///
/// Provides options to filter and search through available templates
/// based on name patterns or technology categories.
#[derive(Debug, Parser, Default)]
pub struct ListArgs {
    /// Filter templates by name or description pattern
    #[clap(short, long)]
    pub filter: Option<String>,

    /// Show only templates for a specific technology stack
    #[clap(short, long)]
    pub tech: Option<String>,
}

/// Executes the list command to display available templates.
///
/// This function retrieves and displays all available templates from the registry,
/// applying any filters specified by the user. Templates are displayed with their
/// names and descriptions in a user-friendly format.
///
/// # Arguments
///
/// * `args` - Command arguments containing optional filters
///
/// # Returns
///
/// Returns `ExitStatus::Success` if templates are listed successfully.
///
/// # Examples
///
/// ```
/// use nettoolskit_commands::list::{ListArgs, run};
///
/// #[tokio::main]
/// async fn main() {
///     // List all templates
///     run(ListArgs { filter: None, tech: None }).await;
///
///     // List only .NET templates
///     run(ListArgs { filter: None, tech: Some("dotnet".to_string()) }).await;
/// }
/// ```
pub async fn run(args: ListArgs) -> ExitStatus {
    info!(
        filter = ?args.filter,
        tech = ?args.tech,
        "Executing list command with filters"
    );

    println!("{}", "📋 Available Templates".bold().blue());
    println!();

    // TODO: Replace with actual template registry integration
    let templates = vec![
        ("dotnet-api", "ASP.NET Core Web API template"),
        ("dotnet-webapp", "ASP.NET Core Web Application"),
        ("dotnet-classlib", ".NET Class Library"),
        ("dotnet-console", ".NET Console Application"),
        ("vue-app", "Vue.js Application"),
        ("react-app", "React Application"),
    ];

    for (name, description) in templates {
        if let Some(ref filter) = args.filter {
            if !name.contains(filter) && !description.contains(filter) {
                continue;
            }
        }

        if let Some(ref tech) = args.tech {
            if !name.starts_with(tech) {
                continue;
            }
        }

        println!("  {} - {}", name.yellow().bold(), description);
    }

    info!("List command completed successfully");
    ExitStatus::Success
}

/// Async version of list command with progress reporting
///
/// This function demonstrates async execution with progress updates
/// for listing templates, particularly useful when fetching from
/// remote registries or scanning large template collections.
///
/// # Arguments
///
/// * `args` - Command arguments
/// * `progress` - Channel for sending progress updates
///
/// # Returns
///
/// Returns `CommandResult` which is `Result<String, Box<dyn Error>>`
pub async fn run_async(
    args: ListArgs,
    progress: tokio::sync::mpsc::UnboundedSender<crate::async_executor::CommandProgress>,
) -> crate::Result<String> {
    use crate::async_executor::CommandProgress;
    use tokio::time::{sleep, Duration};

    info!(
        filter = ?args.filter,
        tech = ?args.tech,
        "Executing async list command with filters"
    );

    // Send initial progress
    progress
        .send(CommandProgress::message("🔍 Scanning for templates..."))
        .ok();
    sleep(Duration::from_millis(100)).await;

    // TODO: Replace with actual template registry integration
    let templates = vec![
        ("dotnet-api", "ASP.NET Core Web API template"),
        ("dotnet-webapp", "ASP.NET Core Web Application"),
        ("dotnet-classlib", ".NET Class Library"),
        ("dotnet-console", ".NET Console Application"),
        ("vue-app", "Vue.js Application"),
        ("react-app", "React Application"),
    ];

    progress
        .send(CommandProgress::percent("📦 Loading templates...", 30))
        .ok();
    sleep(Duration::from_millis(100)).await;

    let mut result = String::from("📋 Available Templates\n\n");
    let mut count = 0;
    let total = templates.len();

    for (idx, (name, description)) in templates.iter().enumerate() {
        // Apply filters
        if let Some(ref filter) = args.filter {
            if !name.contains(filter.as_str()) && !description.contains(filter.as_str()) {
                continue;
            }
        }

        if let Some(ref tech) = args.tech {
            if !name.starts_with(tech.as_str()) {
                continue;
            }
        }

        // Add to result
        result.push_str(&format!("  {} - {}\n", name, description));
        count += 1;

        // Update progress
        progress
            .send(CommandProgress::steps(
                format!("Processing templates... {}/{}", idx + 1, total),
                idx + 1,
                total,
            ))
            .ok();

        sleep(Duration::from_millis(50)).await;
    }

    // Final progress
    progress
        .send(CommandProgress::percent(
            format!("✅ Found {} templates", count),
            100,
        ))
        .ok();

    info!(count = count, "Async list command completed successfully");
    Ok(result)
}
