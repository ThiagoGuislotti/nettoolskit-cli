//! NetToolsKit CLI binary entry point.

use clap::{CommandFactory, Parser};
use clap_complete::Shell;
use nettoolskit_cli::interactive_mode;
use nettoolskit_core::{AppConfig, ColorMode, CommandEntry, UnicodeMode};
use nettoolskit_orchestrator::ExitStatus;
use nettoolskit_otel::init_tracing;
use nettoolskit_ui::{set_color_override, set_unicode_override, ColorLevel};

/// Global arguments available across all commands
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

/// Available CLI commands
#[derive(Debug, Parser)]
pub enum Commands {
    /// Manage and apply manifests (opens interactive submenu)
    Manifest,

    /// Translate templates between programming languages
    Translate {
        /// Source language identifier
        #[clap(long)]
        from: String,

        /// Target language identifier
        #[clap(long)]
        to: String,

        /// Template file path to translate
        path: String,
    },

    /// Generate shell completions for the specified shell
    Completions {
        /// Target shell (bash, zsh, fish, powershell)
        #[clap(value_enum)]
        shell: Shell,
    },
}

impl Commands {
    /// Execute this command
    pub async fn execute(self) -> ExitStatus {
        use nettoolskit_orchestrator::{process_command, MainAction};

        match self {
            Commands::Manifest => process_command(&MainAction::Manifest.slash_static()).await,
            Commands::Translate { from, to, path } => {
                let request = nettoolskit_translate::TranslateRequest { from, to, path };
                nettoolskit_translate::handle_translate(request).await
            }
            Commands::Completions { shell } => {
                clap_complete::generate(shell, &mut Cli::command(), "ntk", &mut std::io::stdout());
                ExitStatus::Success
            }
        }
    }
}

/// NetToolsKit CLI
///
/// A toolkit for .NET development with templates, manifests, and automation tools.
/// If no subcommand is specified, the interactive CLI will be launched.
#[derive(Debug, Parser)]
#[clap(
    author = "NetToolsKit Team",
    version,
    bin_name = "ntk",
    override_usage = "ntk [OPTIONS] [PROMPT]\n       ntk [OPTIONS] <COMMAND> [ARGS]",
    disable_help_subcommand = true
)]
struct Cli {
    #[clap(flatten)]
    pub global: GlobalArgs,

    #[clap(subcommand)]
    pub subcommand: Option<Commands>,
}

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
    // Parse command line arguments
    let cli = Cli::parse();

    // Load configuration (file → env → defaults), then apply CLI overrides
    let config = match &cli.global.config {
        Some(path) => {
            let p = std::path::Path::new(path);
            match AppConfig::load_from(p) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Warning: Failed to load config from {}: {}", path, e);
                    AppConfig::load()
                }
            }
        }
        None => AppConfig::load(),
    };

    let verbose = cli.global.verbose || config.general.verbose;

    // Wire user config into the terminal capabilities system
    match config.display.color {
        ColorMode::Always => set_color_override(Some(ColorLevel::TrueColor)),
        ColorMode::Never => set_color_override(Some(ColorLevel::None)),
        ColorMode::Auto => {} // capabilities auto-detect
    }
    match config.display.unicode {
        UnicodeMode::Always => set_unicode_override(Some(true)),
        UnicodeMode::Never => set_unicode_override(Some(false)),
        UnicodeMode::Auto => {} // capabilities auto-detect
    }

    let run_interactive = cli.subcommand.is_none();

    if !run_interactive {
        if let Err(e) = init_tracing(verbose) {
            eprintln!("Warning: Failed to initialize tracing: {}", e);
        }
    }

    let exit_status: ExitStatus = match cli.subcommand {
        Some(command) => command.execute().await,
        None => interactive_mode(verbose).await,
    };

    // Exit with appropriate code
    std::process::exit(exit_status.into());
}
