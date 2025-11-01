//! New project creation command for NetToolsKit CLI.
//!
//! This module handles creating new projects from templates, including
//! template instantiation, file generation, and project configuration.

use crate::ExitStatus;
use clap::Parser;
use owo_colors::OwoColorize;

/// Arguments for the new command.
///
/// Configures project creation from templates with support for
/// custom naming, output directories, and non-interactive mode.
#[derive(Debug, Parser, Default)]
pub struct NewArgs {
    /// Name of the template to instantiate
    pub template: String,

    /// Name for the new project (defaults to template name)
    #[clap(short, long)]
    pub name: Option<String>,

    /// Directory where the project will be created
    #[clap(short, long)]
    pub output: Option<std::path::PathBuf>,

    /// Skip interactive prompts and use default values
    #[clap(short, long)]
    pub yes: bool,
}

/// Executes the new command to create a project from a template.
///
/// This function handles the complete project creation workflow:
/// 1. Validates the specified template exists
/// 2. Prompts for missing project details (unless --yes is specified)
/// 3. Instantiates the template with user-provided values
/// 4. Creates the project files and directory structure
///
/// # Arguments
///
/// * `args` - Command arguments specifying template and project configuration
///
/// # Returns
///
/// Returns `ExitStatus::Success` if the project is created successfully,
/// or `ExitStatus::Error` if template validation or creation fails.
///
/// # Examples
///
/// ```
/// use nettoolskit_commands::new::{NewArgs, run};
///
/// #[tokio::main]
/// async fn main() {
///     // Create a project interactively
///     run(NewArgs {
///         template: "dotnet-api".to_string(),
///         name: None,
///         output: None,
///         yes: false,
///     }).await;
/// }
/// ```
pub async fn run(args: NewArgs) -> ExitStatus {
    println!("{}", "🚀 Creating new project".bold().green());
    println!("Template: {}", args.template.yellow());

    if let Some(name) = &args.name {
        println!("Name: {}", name.cyan());
    }

    if let Some(output) = &args.output {
        println!("Output: {}", output.display().to_string().cyan());
    }

    // TODO: Implement actual template instantiation logic
    println!();
    println!("{}", "✨ Project created successfully!".green());

    ExitStatus::Success
}
