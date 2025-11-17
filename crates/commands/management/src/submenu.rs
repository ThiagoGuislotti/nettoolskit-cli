/// Manifest submenu - Interactive menu for manifest commands
use crate::definitions::ExitStatus;
use inquire::ui::{RenderConfig, Color, Styled};
use inquire::{Select, Text};
use nettoolskit_core::string_utils::string::truncate_directory_with_middle;
use nettoolskit_ui::{PRIMARY_COLOR, SECONDARY_COLOR, WHITE_COLOR, GRAY_COLOR};
use owo_colors::OwoColorize;
use std::env;
use std::path::PathBuf;

/// Manifest subcommand options
#[derive(Debug, Clone)]
pub enum ManifestSubcommand {
    List,
    Check,
    Render,
    Apply,
    Back,
}

impl ManifestSubcommand {
    fn get_label(&self) -> &str {
        match self {
            ManifestSubcommand::List => "List",
            ManifestSubcommand::Check => "Check",
            ManifestSubcommand::Render => "Render",
            ManifestSubcommand::Apply => "Apply",
            ManifestSubcommand::Back => "Back to main menu",
        }
    }

    fn get_description(&self) -> &str {
        match self {
            ManifestSubcommand::List => "Discover available manifests",
            ManifestSubcommand::Check => "Validate manifest structure",
            ManifestSubcommand::Render => "Preview generated files",
            ManifestSubcommand::Apply => "Generate/update project files",
            ManifestSubcommand::Back => "",
        }
    }
}

impl std::fmt::Display for ManifestSubcommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = self.get_label();
        let desc = self.get_description();
        if desc.is_empty() {
            write!(f, "{}", label)
        } else {
            // Label will be colored by inquire based on selection
            // Description in gray (using ANSI escape codes directly)
            write!(f, "{} - \x1b[90m{}\x1b[0m", label, desc)
        }
    }
}

/// Display manifest submenu and handle selection
pub async fn show_manifest_menu() -> ExitStatus {
    // Get current directory
    let current_dir = env::current_dir()
        .ok()
        .and_then(|p| p.to_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "unknown".to_string());

    // Configure custom theme matching primary color
    let mut render_config = RenderConfig::default();
    render_config.prompt_prefix = Styled::new("?").with_fg(Color::Rgb { r: PRIMARY_COLOR.0, g: PRIMARY_COLOR.1, b: PRIMARY_COLOR.2 });
    render_config.highlighted_option_prefix = Styled::new("❯").with_fg(Color::Rgb { r: PRIMARY_COLOR.0, g: PRIMARY_COLOR.1, b: PRIMARY_COLOR.2 });
    render_config.selected_option = Some(render_config.selected_option.unwrap_or_default().with_fg(Color::Rgb { r: PRIMARY_COLOR.0, g: PRIMARY_COLOR.1, b: PRIMARY_COLOR.2 }));
    render_config.help_message = render_config.help_message.with_fg(Color::DarkYellow);

    loop {
        // Print command being executed
        println!();
        println!("{}", "> manifest".color(PRIMARY_COLOR).bold());

        println!();

        // Print box header
        let box_width = 89; // Same as main menu
        println!("{}", "╭─────────────────────────────────────────────────────────────────────────────────────────╮".color(PRIMARY_COLOR));
        println!(
            "{}{}{}{}",
            "│".color(PRIMARY_COLOR),
            " >_".color(PRIMARY_COLOR).bold(),
            " Manifest Commands Menu".color(WHITE_COLOR).bold(),
            "                                                              │".color(PRIMARY_COLOR)
        );
        println!("{}", "│    Interactive menu for manifest operations                                            │".color(PRIMARY_COLOR));
        println!("{}", "│                                                                                         │".color(PRIMARY_COLOR));

        // Calculate available width for directory path (same logic as main menu)
        let dir_label = "    directory: ";
        let available_width = box_width - dir_label.len() - 1 - 4 - 4; // -1 for │, -4 for spaces, -4 for safety margin
        let truncated_dir = truncate_directory_with_middle(&current_dir, available_width);

        // Calculate padding for directory line
        let dir_text_length = dir_label.len() + truncated_dir.len();
        let padding_needed = box_width - dir_text_length;
        let padding = " ".repeat(padding_needed.max(4));

        println!(
            "{}{}{}{}",
            "│".color(PRIMARY_COLOR),
            "    directory: ".color(GRAY_COLOR),
            truncated_dir.color(WHITE_COLOR),
            format!("{}│", padding).color(PRIMARY_COLOR)
        );
        println!("{}", "╰─────────────────────────────────────────────────────────────────────────────────────────╯".color(PRIMARY_COLOR));

        println!();
        println!("{}", "    [Use ↑↓ to navigate, Enter to select, /quit to exit]".color(GRAY_COLOR));
        println!();

        let options = vec![
            ManifestSubcommand::List,
            ManifestSubcommand::Check,
            ManifestSubcommand::Render,
            ManifestSubcommand::Apply,
            ManifestSubcommand::Back,
        ];

        let selection = Select::new("Select a manifest command:", options)
            .with_page_size(6)
            .with_render_config(render_config)
            .without_help_message()
            .prompt();

        match selection {
            Ok(ManifestSubcommand::List) => {
                execute_list().await;
            }
            Ok(ManifestSubcommand::Check) => {
                execute_check().await;
            }
            Ok(ManifestSubcommand::Render) => {
                execute_render().await;
            }
            Ok(ManifestSubcommand::Apply) => {
                execute_apply_interactive().await;
            }
            Ok(ManifestSubcommand::Back) => {
                println!("{}", "← Returning to main menu...".yellow());
                return ExitStatus::Success;
            }
            Err(_) => {
                println!("{}", "Menu cancelled".yellow());
                return ExitStatus::Success;
            }
        }
    }
}

/// Execute manifest list command
async fn execute_list() -> ExitStatus {
    println!();
    println!("{}", "📋 Discovering Manifests...".cyan().bold());
    println!("{}", "─".repeat(25).cyan());
    println!();

    // TODO: Implement actual manifest discovery
    println!("{}", "ℹ️  Manifest discovery will list all available manifest files".yellow());
    println!("{}", "   Searching for: manifest.yaml, *.manifest.yaml".dimmed());
    println!();

    ExitStatus::Success
}

/// Execute manifest check command
async fn execute_check() -> ExitStatus {
    println!();
    println!("{}", "✅ Validating Manifest...".cyan().bold());
    println!("{}", "─".repeat(23).cyan());
    println!();

    // Prompt for manifest path
    let manifest_path = Text::new("Manifest file path:")
        .with_help_message("Enter the path to the manifest file")
        .with_placeholder("manifest.yaml")
        .prompt();

    match manifest_path {
        Ok(path) if !path.is_empty() => {
            println!();
            println!("Validating: {}", path.green());
            // TODO: Implement actual validation
            println!("{}", "ℹ️  Manifest validation will check structure and dependencies".yellow());
            println!();
        }
        _ => {
            println!("{}", "Validation cancelled".yellow());
        }
    }

    ExitStatus::Success
}

/// Execute manifest render command
async fn execute_render() -> ExitStatus {
    println!();
    println!("{}", "🎨 Rendering Preview...".cyan().bold());
    println!("{}", "─".repeat(20).cyan());
    println!();

    // TODO: Implement actual rendering
    println!("{}", "ℹ️  Manifest rendering will preview generated files".yellow());
    println!("{}", "   Features: syntax highlighting, variable substitution".dimmed());
    println!();

    ExitStatus::Success
}

/// Execute manifest apply command interactively
async fn execute_apply_interactive() -> ExitStatus {
    println!();
    println!("{}", "⚡ Applying Manifest...".cyan().bold());
    println!("{}", "─".repeat(21).cyan());
    println!();

    // Prompt for manifest path
    let manifest_path = Text::new("Manifest file path:")
        .with_help_message("Enter the path to the manifest file")
        .with_placeholder("feature.manifest.yaml")
        .prompt();

    let path = match manifest_path {
        Ok(p) if !p.is_empty() => PathBuf::from(p),
        _ => {
            println!("{}", "Apply cancelled".yellow());
            return ExitStatus::Success;
        }
    };

    // Prompt for dry-run
    let dry_run_options = vec!["No - Apply changes", "Yes - Dry-run (preview only)"];
    let dry_run_selection = Select::new("Run in dry-run mode?", dry_run_options)
        .with_help_message("Dry-run will preview changes without modifying files")
        .prompt();

    let dry_run = match dry_run_selection {
        Ok(option) => option.starts_with("Yes"),
        Err(_) => {
            println!("{}", "Apply cancelled".yellow());
            return ExitStatus::Success;
        }
    };

    // Prompt for output directory
    let output_dir = Text::new("Output directory (optional):")
        .with_help_message("Leave empty to use current directory")
        .with_placeholder("./src")
        .prompt();

    let output_root = match output_dir {
        Ok(dir) if !dir.is_empty() => Some(PathBuf::from(dir)),
        _ => None,
    };

    println!();
    println!("{}", "Executing manifest apply...".cyan());

    // Execute the apply handler
    crate::handlers::execute_apply(path, output_root, dry_run).await
}
