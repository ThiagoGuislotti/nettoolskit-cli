use clap::{CommandFactory, Parser};
use clap_complete::generate;
use nettoolskit_cli::{interactive_mode, ExitStatus};

use std::io;

mod commands;
use commands::{Commands, GlobalArgs};

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

#[tokio::main]
async fn main() {
    // Parse command line arguments
    let cli = Cli::parse();

    // Initialize tracing if verbose mode is enabled
    if cli.global.verbose {
        tracing_subscriber::fmt()
            .with_env_filter("nettoolskit=debug")
            .init();
    }

    // Handle subcommands or launch interactive mode
    let exit_status = match cli.subcommand {
        Some(Commands::List(args)) => {
            commands::list::run(args).await
        }
        Some(Commands::New(args)) => {
            commands::new::run(args).await
        }
        Some(Commands::Check(args)) => {
            commands::check::run(args).await
        }
        Some(Commands::Render(args)) => {
            commands::render::run(args).await
        }
        Some(Commands::Apply(args)) => {
            commands::apply::run(args).await
        }
        Some(Commands::Completion(args)) => {
            let mut cmd = Cli::command();
            generate(args.shell, &mut cmd, "ntk", &mut io::stdout());
            ExitStatus::Success
        }
        None => {
            // Launch interactive mode
            interactive_mode().await
        }
    };

    // Exit with appropriate code
    std::process::exit(match exit_status {
        ExitStatus::Success => 0,
        ExitStatus::Error => 1,
        ExitStatus::Interrupted => 130,
    });
}