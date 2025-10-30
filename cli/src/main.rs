use clap::Parser;
use nettoolskit_cli::interactive_mode;
use nettoolskit_commands::{Commands, GlobalArgs};
use nettoolskit_otel::init_tracing;

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

    // Initialize tracing using our otel module
    if let Err(e) = init_tracing(cli.global.verbose) {
        eprintln!("Warning: Failed to initialize tracing: {}", e);
    }

    // Handle subcommands or launch interactive mode
    let exit_status = match cli.subcommand {
        Some(command) => nettoolskit_commands::execute_command(command, cli.global).await.into(),
        None => interactive_mode().await,
    };

    // Exit with appropriate code
    std::process::exit(exit_status.into());
}