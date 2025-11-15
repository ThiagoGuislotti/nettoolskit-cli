use clap::Parser;
use nettoolskit_cli::interactive_mode;
use nettoolskit_commands::{nettoolskit_translate, ExitStatus};
use nettoolskit_otel::init_tracing;

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
    /// List available templates
    List,

    /// Create a new project from a template
    New,

    /// Check template or manifest validity
    Check,

    /// Render template preview
    Render,

    /// Apply manifest to existing solution
    Apply,

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
}

impl Commands {
    /// Execute this command
    pub async fn execute(self) -> ExitStatus {
        use nettoolskit_commands::process_command;

        match self {
            Commands::Translate { from, to, path } => {
                let request = nettoolskit_translate::TranslateRequest { from, to, path };
                nettoolskit_translate::handle_translate(request).await
            }
            Commands::List => process_command("/list").await,
            Commands::New => process_command("/new").await,
            Commands::Check => process_command("/check").await,
            Commands::Render => process_command("/render").await,
            Commands::Apply => process_command("/apply").await,
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
    override_usage = "ntk [OPTIONS] [PROMPT]\n       ntk [OPTIONS] <COMMAND> [ARGS]"
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

    let run_interactive = cli.subcommand.is_none();

    if !run_interactive {
        if let Err(e) = init_tracing(cli.global.verbose) {
            eprintln!("Warning: Failed to initialize tracing: {}", e);
        }
    }

    let exit_status: ExitStatus = match cli.subcommand {
        Some(command) => command.execute().await,
        None => interactive_mode(cli.global.verbose).await,
    };

    // Exit with appropriate code
    std::process::exit(exit_status.into());
}
