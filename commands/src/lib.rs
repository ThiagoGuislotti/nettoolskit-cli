//! Command processing and execution for NetToolsKit CLI
//!
//! This crate provides the core command processing logic, including:
//! - Command parsing and validation
//! - Async command execution with progress tracking
//! - Template rendering and application
//! - Exit status handling
//!
//! # Architecture
//!
//! Commands follow a processor pattern where each command type has its own
//! dedicated module. The `processor` module coordinates command execution,
//! while `async_executor` handles long-running operations with progress feedback.

use clap::Parser;

mod error;

pub mod apply;
pub mod async_executor;
pub mod check;
pub mod list;
pub mod new;
pub mod processor;
pub mod processor_async;
pub mod render;

// Re-export error types
pub use error::{CommandError, Result};

// Re-export commonly used types
pub use async_executor::{
    AsyncCommandExecutor, CommandHandle, CommandProgress, CommandResult, ProgressSender,
};

/// Exit status codes for command execution.
///
/// Represents the outcome of a command execution, convertible to
/// standard exit codes for shell integration.
#[derive(Debug, Clone, Copy)]
pub enum ExitStatus {
    /// Command executed successfully (exit code 0)
    Success,
    /// Command failed with an error (exit code 1)
    Error,
    /// Command was interrupted by user (exit code 130 - SIGINT)
    Interrupted,
}

impl From<ExitStatus> for std::process::ExitCode {
    fn from(status: ExitStatus) -> Self {
        match status {
            ExitStatus::Success => std::process::ExitCode::SUCCESS,
            ExitStatus::Error => std::process::ExitCode::FAILURE,
            ExitStatus::Interrupted => std::process::ExitCode::from(130),
        }
    }
}

impl From<ExitStatus> for i32 {
    fn from(status: ExitStatus) -> Self {
        match status {
            ExitStatus::Success => 0,
            ExitStatus::Error => 1,
            ExitStatus::Interrupted => 130,
        }
    }
}

/// Global arguments available across all commands.
///
/// These options can be specified with any command and control
/// cross-cutting concerns like logging and configuration.
#[derive(Debug, Clone, Parser)]
pub struct GlobalArgs {
    /// Set logging level (off, error, warn, info, debug, trace)
    #[clap(long, global = true, default_value = "info")]
    pub log_level: String,

    /// Path to configuration file
    #[clap(long, global = true)]
    pub config: Option<String>,

    /// Enable verbose output
    #[clap(short, long, global = true)]
    pub verbose: bool,
}

/// Available CLI commands.
///
/// Each variant corresponds to a top-level command that can be
/// executed from the CLI. Commands are parsed using clap's derive API.
#[derive(Debug, Parser)]
pub enum Commands {
    /// List available templates
    List(list::ListArgs),

    /// Create a new project from a template
    New(new::NewArgs),

    /// Check template or manifest validity
    Check(check::CheckArgs),

    /// Render template preview
    Render(render::RenderArgs),

    /// Apply manifest to existing solution
    Apply(apply::ApplyArgs),
}

pub async fn execute_command(cmd: Commands, _global_args: GlobalArgs) -> ExitStatus {
    match cmd {
        Commands::List(args) => list::run(args).await,
        Commands::New(args) => new::run(args).await,
        Commands::Check(args) => check::run(args).await,
        Commands::Render(args) => render::run(args).await,
        Commands::Apply(args) => apply::run(args).await,
    }
}

/// Execute multiple commands concurrently (useful for batch operations)
pub async fn execute_commands_concurrent(
    commands: Vec<Commands>,
    global_args: GlobalArgs,
) -> Vec<ExitStatus> {
    use futures::future::join_all;

    let futures: Vec<_> = commands
        .into_iter()
        .map(|cmd| execute_command(cmd, global_args.clone()))
        .collect();

    join_all(futures).await
}

/// Execute a command with timeout and cancellation support
pub async fn execute_command_with_timeout(
    cmd: Commands,
    global_args: GlobalArgs,
    timeout: std::time::Duration,
) -> crate::Result<ExitStatus> {
    match nettoolskit_async_utils::with_timeout(timeout, execute_command(cmd, global_args)).await {
        Ok(status) => Ok(status),
        Err(_timeout_err) => Err("Command timed out".into()),
    }
}

// Slash command definitions for the interactive palette
use strum::IntoEnumIterator;
use strum_macros::{AsRefStr, EnumIter, EnumString, IntoStaticStr};

/// Slash commands for interactive command palette.
///
/// These commands can be invoked by typing a leading slash (/) in the
/// interactive prompt. The enum order determines presentation order in the popup.
///
/// # Note
///
/// Do not alphabetically sort! Enum order is intentional for UX.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, EnumString, EnumIter, AsRefStr, IntoStaticStr,
)]
#[strum(serialize_all = "kebab-case")]
pub enum SlashCommand {
    // DO NOT ALPHA-SORT! Enum order is presentation order in the popup
    List,
    New,
    Check,
    Render,
    Apply,
    Quit,
}

impl SlashCommand {
    /// User-visible description shown in the popup.
    pub fn description(self) -> &'static str {
        match self {
            SlashCommand::List => "List available templates",
            SlashCommand::New => "Create a project from a template",
            SlashCommand::Check => "Validate a manifest or template",
            SlashCommand::Render => "Render a template preview",
            SlashCommand::Apply => "Apply a manifest to an existing solution",
            SlashCommand::Quit => "Exit NetToolsKit CLI",
        }
    }

    /// Command string without the leading '/'.
    pub fn command(self) -> &'static str {
        self.into()
    }

    /// Whether this command can be run while a task is in progress.
    pub fn available_during_task(self) -> bool {
        match self {
            SlashCommand::List
            | SlashCommand::New
            | SlashCommand::Check
            | SlashCommand::Render
            | SlashCommand::Apply => false,
            SlashCommand::Quit => true,
        }
    }
}

/// Return all built-in commands in a Vec paired with their command string.
pub fn built_in_slash_commands() -> Vec<(&'static str, SlashCommand)> {
    SlashCommand::iter().map(|c| (c.command(), c)).collect()
}

// Re-export commands from core to maintain API compatibility
pub use nettoolskit_core::commands::COMMANDS;
