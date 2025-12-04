//! Interactive menu for manifest commands
//!
//! This module provides the interactive UI menu for selecting and executing
//! manifest subcommands (check, render, apply).

use crate::models::ManifestAction;
use inquire::Text;
use nettoolskit_core::{ExitStatus, path_utils::directory::get_current_directory};
use nettoolskit_ui::{
    render_section_title, EnumMenuConfig, render_enum_menu, MenuConfig, render_interactive_menu, Color,
};
use owo_colors::OwoColorize;
use std::path::PathBuf;

/// Display manifest menu and handle selection
pub async fn show_menu() -> ExitStatus {
    // Get current directory
    let current_dir = get_current_directory();

    loop {
        // Use the generic enum menu renderer
        let menu_config = EnumMenuConfig::new(
            "Manifest Commands Menu",
            "Interactive menu for manifest operations",
            current_dir.clone(),
        )
        .with_theme_color(Color::PURPLE)
        .with_width(89);

        match render_enum_menu::<ManifestAction>(menu_config) {
            Ok(action) => {
                // Execute the selected action
                match action {
                    ManifestAction::Check => {
                        execute_check().await;
                    }
                    ManifestAction::Render => {
                        execute_render().await;
                    }
                    ManifestAction::Apply => {
                        execute_apply_interactive().await;
                    }
                }
            }
            Err(_) => {
                println!("{}", "← Returning to main menu...".yellow());
                return ExitStatus::Success;
            }
        }
    }
}

/// Execute manifest check command
async fn execute_check() -> ExitStatus {
    render_section_title("Validating Manifest...", Some("✅"));

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
    render_section_title("Rendering Preview...", Some("🎨"));

    // TODO: Implement actual rendering
    println!("{}", "ℹ️  Manifest rendering will preview generated files".yellow());
    println!("{}", "   Features: syntax highlighting, variable substitution".dimmed());
    println!();

    ExitStatus::Success
}

/// Execute manifest apply command interactively
async fn execute_apply_interactive() -> ExitStatus {
    render_section_title("Applying Manifest...", Some("⚡"));

    // Prompt for manifest path
    let manifest_path = Text::new("Manifest file path:")
        .with_help_message("Enter the path to the manifest file")
        .with_placeholder("feature.manifest.yaml")
        .prompt();

    let _path = match manifest_path {
        Ok(p) if !p.is_empty() => PathBuf::from(p),
        _ => {
            println!("{}", "Apply cancelled".yellow());
            return ExitStatus::Success;
        }
    };

    // Prompt for dry-run
    let dry_run_options = vec!["No - Apply changes", "Yes - Dry-run (preview only)"];
    let dry_run_menu = MenuConfig::new("Run in dry-run mode?", dry_run_options)
        .with_cursor_color(Color::PURPLE)
        .with_help_message("Dry-run will preview changes without modifying files");

    let dry_run_selection = render_interactive_menu(dry_run_menu);

    let _dry_run = match dry_run_selection {
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

    let _output_root = match output_dir {
        Ok(dir) if !dir.is_empty() => Some(PathBuf::from(dir)),
        _ => None,
    };

    println!();
    println!("{}", "Executing manifest apply...".cyan());

    // TODO: Call the actual apply handler when available
    // For now, just return success
    println!("{}", "ℹ️  Apply handler integration pending".yellow());
    ExitStatus::Success
}
