use clap::Parser;

pub mod apply;
pub mod check;
pub mod list;
pub mod new;
pub mod processor;
pub mod render;

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

#[derive(Debug, Parser)]
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