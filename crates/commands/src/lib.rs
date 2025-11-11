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
//! while `executor` handles long-running operations with progress feedback.

use clap::Parser;

mod error;

// Core modules
pub mod processor;
pub mod registry;

// Re-export error types
pub use error::{CommandError, Result};

// Re-export commonly used types from async-utils
pub mod executor;

pub use executor::{
    AsyncCommandExecutor, CommandHandle, CommandProgress, CommandResult, ProgressSender,
};
pub use registry::CommandRegistry;

/// Exit status codes for command execution.
///
/// Represents the outcome of a command execution, convertible to
/// standard exit codes for shell integration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    /// List available templates (placeholder)
    List,

    /// Create a new project from a template (placeholder)
    New,

    /// Check template or manifest validity (placeholder)
    Check,

    /// Render template preview (placeholder)
    Render,

    /// Apply manifest to existing solution (placeholder)
    Apply,
}

impl Commands {
    /// Convert command enum to slash command string for processor
    ///
    /// This encapsulates the mapping between CLI commands and processor commands,
    /// following SRP - CLI doesn't need to know about command routing.
    pub fn as_slash_command(&self) -> &'static str {
        match self {
            Commands::List => "/list",
            Commands::New => "/new",
            Commands::Check => "/check",
            Commands::Render => "/render",
            Commands::Apply => "/apply",
        }
    }

    /// Execute this command using the processor
    ///
    /// This is the main entry point for command execution from CLI.
    /// Delegates to processor::process_command() for actual dispatch.
    pub async fn execute(self) -> ExitStatus {
        processor::process_command(self.as_slash_command()).await
    }
}

// NOTE: Command execution is handled by processor::process_command()
// This avoids duplication - processor.rs is the single source of truth for dispatch
