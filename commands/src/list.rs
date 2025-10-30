//! List command implementation for NetToolsKit CLI.
//!
//! This module handles listing available templates from the registry,
//! with support for filtering by name patterns and technology stacks.

use clap::Parser;
use crate::ExitStatus;
use owo_colors::OwoColorize;

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

    ExitStatus::Success
}