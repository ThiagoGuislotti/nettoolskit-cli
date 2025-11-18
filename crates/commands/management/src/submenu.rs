/// Manifest submenu - Interactive menu for manifest commands
use crate::definitions::ExitStatus;
use inquire::Text;
use nettoolskit_core::path_utils::directory::get_current_directory;
use nettoolskit_ui::{
    BoxConfig, MenuConfig, render_box, render_interactive_menu,
    PRIMARY_COLOR, WHITE_COLOR, GRAY_COLOR
};
use owo_colors::OwoColorize;
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
            ManifestSubcommand::List =>     "   List",
            ManifestSubcommand::Check =>    "   Check",
            ManifestSubcommand::Render =>   "   Render",
            ManifestSubcommand::Apply =>    "   Apply",
            ManifestSubcommand::Back =>     "   Back",
        }
    }

    fn get_description(&self) -> &str {
        match self {
            ManifestSubcommand::List =>     "Discover available manifests",
            ManifestSubcommand::Check =>    "Validate manifest structure",
            ManifestSubcommand::Render =>   "Preview generated files",
            ManifestSubcommand::Apply =>    "Generate/update project files",
            ManifestSubcommand::Back =>     "To main menu",
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
    let current_dir = get_current_directory();

    loop {
        // Print command being executed
        println!();
        println!("{}", "> manifest".color(PRIMARY_COLOR));
        println!();

        // Render box using component
        let box_config = BoxConfig::new("Manifest Commands Menu")
            .with_title_prefix(">_")
            .with_title_color(WHITE_COLOR)
            .with_subtitle("Interactive menu for manifest operations")
            .add_footer_item("directory", current_dir.clone(), WHITE_COLOR)
            .with_border_color(PRIMARY_COLOR)
            .with_width(89)
            .with_spacing(false);

        render_box(box_config);

        println!();
        println!(
            "{}",
            "   Use ↑↓ to navigate, Enter to select, /quit to exit".color(GRAY_COLOR)
        );
        println!();

        // Render menu using component
        let menu_items = vec![
            ManifestSubcommand::List,
            ManifestSubcommand::Check,
            ManifestSubcommand::Render,
            ManifestSubcommand::Apply,
            ManifestSubcommand::Back,
        ];

        let menu_config = MenuConfig::new("Select a manifest command:", menu_items)
            .with_cursor_color(PRIMARY_COLOR)
            .with_page_size(6);

        let selection = render_interactive_menu(menu_config);

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
    let dry_run_menu = MenuConfig::new("Run in dry-run mode?", dry_run_options)
        .with_cursor_color(PRIMARY_COLOR)
        .with_help_message("Dry-run will preview changes without modifying files");

    let dry_run_selection = render_interactive_menu(dry_run_menu);

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
