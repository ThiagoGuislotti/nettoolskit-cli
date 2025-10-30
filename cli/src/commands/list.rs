use clap::Parser;
use nettoolskit_cli::ExitStatus;
use owo_colors::OwoColorize;

#[derive(Debug, Parser)]
pub struct ListArgs {
    /// Filter templates by pattern
    #[clap(short, long)]
    pub filter: Option<String>,

    /// Show only templates for specific technology
    #[clap(short, long)]
    pub tech: Option<String>,
}

pub async fn run(args: ListArgs) -> ExitStatus {
    println!("{}", "📋 Available Templates".bold().blue());
    println!();

    // Placeholder implementation
    let templates = vec![
        ("dotnet-api", "ASP.NET Core Web API template"),
        ("dotnet-webapp", "ASP.NET Core Web Application"),
        ("dotnet-classlib", ".NET Class Library"),
        ("dotnet-console", ".NET Console Application"),
        ("vue-app", "Vue.js Application"),
        ("react-app", "React Application"),
    ];

    for (name, description) in templates {
        if let Some(ref filter) = args.filter {
            if !name.contains(filter) && !description.contains(filter) {
                continue;
            }
        }

        if let Some(ref tech) = args.tech {
            if !name.starts_with(tech) {
                continue;
            }
        }

        println!("  {} - {}", name.yellow().bold(), description);
    }

    ExitStatus::Success
}