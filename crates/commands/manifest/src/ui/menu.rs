//! Interactive menu for manifest commands
//!
//! This module provides the interactive UI menu for selecting and executing
//! manifest subcommands (check, render, apply).

use crate::core::definitions::ManifestAction;
use inquire::Text;
use nettoolskit_core::{ExitStatus, MenuEntry, path_utils::directory::get_current_directory};
use nettoolskit_ui::{
    render_section_title, MenuConfig, render_interactive_menu, CommandPalette, PRIMARY_COLOR,
};
use owo_colors::OwoColorize;
use std::path::PathBuf;

/// Manifest menu item (action or back)
#[derive(Debug, Clone)]
pub enum ManifestMenuItem {
    Action(ManifestAction),
    Back,
}

impl MenuEntry for ManifestMenuItem {
    fn label(&self) -> &str {
        match self {
            ManifestMenuItem::Action(action) => action.name(),
            ManifestMenuItem::Back => "Back",
        }
    }

    fn description(&self) -> &str {
        match self {
            ManifestMenuItem::Action(action) => action.description(),
            ManifestMenuItem::Back => "To main menu",
        }
    }
}

/// Display manifest menu and handle selection
pub async fn show_menu() -> ExitStatus {
    // Get current directory
    let current_dir = get_current_directory();

    loop {
        // Build menu items
        let menu_items = vec![
            ManifestMenuItem::Action(ManifestAction::Check),
            ManifestMenuItem::Action(ManifestAction::Render),
            ManifestMenuItem::Action(ManifestAction::Apply),
            ManifestMenuItem::Back,
        ];

        // Create and show palette menu
        let palette = CommandPalette::new(menu_items)
            .with_title("Manifest Commands Menu")
            .with_subtitle("Interactive menu for manifest operations")
            .with_directory(current_dir.clone());

        match palette.show() {
            Some(selected_label) => {
                // Match selected label to action
                match selected_label.as_str() {
                    "check" => {
                        execute_check().await;
                    }
                    "render" => {
                        execute_render().await;
                    }
                    "apply" => {
                        execute_apply_interactive().await;
                    }
                    "Back" => {
                        println!("{}", "← Returning to main menu...".yellow());
                        return ExitStatus::Success;
                    }
                    _ => {
                        println!("{}", format!("Unknown command: {}", selected_label).red());
                    }
                }
            }
            None => {
                println!("{}", "Menu cancelled".yellow());
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
        .with_cursor_color(PRIMARY_COLOR)
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
