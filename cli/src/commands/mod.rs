use clap::Parser;


pub mod apply;
pub mod check;
pub mod list;
pub mod new;
pub mod render;

#[derive(Debug, Parser)]
pub struct GlobalArgs {
    /// Enable verbose logging
    #[clap(short, long)]
    pub verbose: bool,

    /// Configuration file path
    #[clap(short, long)]
    pub config: Option<std::path::PathBuf>,
}

#[derive(Debug, clap::Subcommand)]
pub enum Commands {
    /// List available templates
    List(list::ListArgs),

    /// Create a project from a template
    New(new::NewArgs),

    /// Validate a manifest or template
    Check(check::CheckArgs),

    /// Render a template preview
    Render(render::RenderArgs),

    /// Apply a manifest to an existing solution
    Apply(apply::ApplyArgs),

    /// Generate shell completion scripts
    Completion(CompletionArgs),
}#[derive(Debug, Parser)]
pub struct CompletionArgs {
    /// Shell to generate completions for
    #[clap(value_enum, default_value_t = clap_complete::Shell::Bash)]
    pub shell: clap_complete::Shell,
}