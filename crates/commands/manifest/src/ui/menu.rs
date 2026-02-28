//! Interactive menu for manifest commands
//!
//! This module provides the interactive UI menu for selecting and executing
//! manifest subcommands (check, render, apply).

use crate::models::ManifestAction;
use inquire::Text;
use nettoolskit_core::{path_utils::directory::get_current_directory, ExitStatus};
use nettoolskit_ui::{
    render_enum_menu, render_interactive_menu, render_section_title, Color, EnumMenuConfig,
    MenuConfig,
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
                    ManifestAction::Back => {
                        return ExitStatus::Success;
                    }
                }
            }
            Err(_) => {
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
            let file_path = PathBuf::from(&path);
            println!();
            println!("Validating: {}", path.color(Color::GREEN));

            match crate::handlers::check::check_file(&file_path, false).await {
                Ok(validation) => {
                    crate::handlers::check::display_validation_result(&file_path, &validation);
                }
                Err(e) => {
                    println!("{} {e}", "❌ Validation failed:".color(Color::RED));
                }
            }
        }
        _ => {
            println!("{}", "Validation cancelled".color(Color::YELLOW));
        }
    }

    ExitStatus::Success
}

/// Execute manifest render command (dry-run preview)
async fn execute_render() -> ExitStatus {
    render_section_title("Rendering Preview...", Some("🎨"));

    let manifest_path = Text::new("Manifest file path:")
        .with_help_message("Enter the path to preview rendering")
        .with_placeholder("manifest.yaml")
        .prompt();

    let path = match manifest_path {
        Ok(p) if !p.is_empty() => PathBuf::from(p),
        _ => {
            println!("{}", "Render preview cancelled".color(Color::YELLOW));
            return ExitStatus::Success;
        }
    };

    let output_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    println!();
    println!("{}", "Rendering preview (dry-run)...".color(Color::CYAN));
    println!("{}", "No files will be modified.".dimmed());
    println!();

    let config = crate::ExecutionConfig {
        manifest_path: path,
        output_root,
        dry_run: true,
    };

    let executor = crate::ManifestExecutor::new();
    match executor.execute(config).await {
        Ok(summary) => {
            println!("{}", "✓ Render preview completed".color(Color::GREEN));
            println!();

            if !summary.created.is_empty() {
                println!(
                    "{}",
                    format!("Files to create: {}", summary.created.len()).color(Color::GREEN)
                );
                for p in &summary.created {
                    println!("  + {}", p.display());
                }
                println!();
            }

            if !summary.updated.is_empty() {
                println!(
                    "{}",
                    format!("Files to update: {}", summary.updated.len()).color(Color::GREEN)
                );
                for p in &summary.updated {
                    println!("  ~ {}", p.display());
                }
                println!();
            }

            if !summary.skipped.is_empty() {
                println!("Files to skip: {}", summary.skipped.len());
                for (p, reason) in &summary.skipped {
                    println!("  - {} ({})", p.display(), reason);
                }
                println!();
            }

            if !summary.notes.is_empty() {
                println!("{}", "Notes:".color(Color::CYAN));
                for note in &summary.notes {
                    println!("  • {note}");
                }
                println!();
            }

            println!(
                "Total artifacts: {}",
                summary.created.len() + summary.updated.len()
            );
        }
        Err(e) => {
            println!("{} {e}", "❌ Render preview failed:".color(Color::RED));
        }
    }

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

    let path = match manifest_path {
        Ok(p) if !p.is_empty() => PathBuf::from(p),
        _ => {
            println!("{}", "Apply cancelled".color(Color::YELLOW));
            return ExitStatus::Success;
        }
    };

    // Prompt for dry-run
    let dry_run_options = vec!["No - Apply changes", "Yes - Dry-run (preview only)"];
    let dry_run_menu = MenuConfig::new("Run in dry-run mode?", dry_run_options)
        .with_cursor_color(Color::PURPLE)
        .with_help_message("Dry-run will preview changes without modifying files");

    let dry_run_selection = render_interactive_menu(dry_run_menu);

    let dry_run = match dry_run_selection {
        Ok(option) => option.starts_with("Yes"),
        Err(_) => {
            println!("{}", "Apply cancelled".color(Color::YELLOW));
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
    println!("{}", "Executing manifest apply...".color(Color::CYAN));

    crate::handlers::apply::execute_apply(path, output_root, dry_run).await
}
