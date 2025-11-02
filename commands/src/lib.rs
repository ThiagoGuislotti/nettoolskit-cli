use clap::Parser;

pub mod apply;
pub mod async_executor;
pub mod check;
pub mod list;
pub mod new;
pub mod processor;
pub mod processor_async;
pub mod render;

// Re-export commonly used types
pub use async_executor::{
    AsyncCommandExecutor, CommandHandle, CommandProgress, CommandResult, ProgressSender,
};

// Error type for the commands crate
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Copy)]
pub enum ExitStatus {
    Success,
    Error,
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

/// Commands that can be invoked by starting a message with a leading slash.
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
