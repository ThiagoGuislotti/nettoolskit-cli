use clap::Parser;
use nettoolskit_cli::ExitStatus;
use owo_colors::OwoColorize;

#[derive(Debug, Parser)]
pub struct NewArgs {
    /// Template name to use
    pub template: String,

    /// Project name
    #[clap(short, long)]
    pub name: Option<String>,

    /// Output directory
    #[clap(short, long)]
    pub output: Option<std::path::PathBuf>,

    /// Skip interactive prompts
    #[clap(short, long)]
    pub yes: bool,
}

pub async fn run(args: NewArgs) -> ExitStatus {
    println!("{}", "🚀 Creating new project".bold().green());
    println!("Template: {}", args.template.yellow());

    if let Some(name) = &args.name {
        println!("Name: {}", name.cyan());
    }

    if let Some(output) = &args.output {
        println!("Output: {}", output.display().to_string().cyan());
    }

    // Placeholder implementation
    println!();
    println!("{}", "✨ Project created successfully!".green());

    ExitStatus::Success
}