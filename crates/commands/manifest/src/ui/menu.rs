//! Interactive menu for manifest commands
//!
//! This module provides the interactive UI menu for selecting and executing
//! manifest subcommands (check, render, apply).

use crate::models::ManifestAction;
use inquire::Text;
use nettoolskit_core::{path_utils::directory::get_current_directory, ExitStatus};
use nettoolskit_ui::{
    render_enum_menu, render_interactive_menu, render_section_title, Color, EnumMenuConfig,
    FilePicker, MenuConfig,
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
        .with_theme_color(Color::PURPLE);

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
                        show_apply_menu().await;
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

    match prompt_manifest_path(
        "Check",
        "Enter the path to the manifest file",
        "ntk-manifest.yml",
    ) {
        Some(file_path) => {
            println!();
            println!(
                "Validating: {}",
                file_path.display().to_string().color(Color::GREEN)
            );

            match crate::handlers::check::check_file(&file_path, false).await {
                Ok(validation) => {
                    crate::handlers::check::display_validation_result(&file_path, &validation);
                }
                Err(e) => {
                    println!("{} {e}", "❌ Validation failed:".color(Color::RED));
                }
            }
        }
        None => {
            println!("{}", "Validation cancelled".color(Color::YELLOW));
        }
    }

    ExitStatus::Success
}

/// Execute manifest render command (dry-run preview)
async fn execute_render() -> ExitStatus {
    render_section_title("Rendering Preview...", Some("🎨"));

    let path = match prompt_manifest_path(
        "Render",
        "Enter the path to preview rendering",
        "ntk-manifest.yml",
    ) {
        Some(path) => path,
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
pub async fn show_apply_menu() -> ExitStatus {
    render_section_title("Applying Manifest...", Some("⚡"));

    let path = match prompt_manifest_path(
        "Apply",
        "Enter the path to the manifest file",
        "feature.manifest.yaml",
    ) {
        Some(path) => path,
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
        Ok(option) => should_enable_dry_run(option),
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

    let output_root = parse_output_directory_input(output_dir);

    println!();
    println!("{}", "Executing manifest apply...".color(Color::CYAN));

    crate::handlers::apply::execute_apply(path, output_root, dry_run).await
}

fn prompt_manifest_path(
    action_name: &str,
    manual_help: &str,
    placeholder: &str,
) -> Option<PathBuf> {
    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let picker = FilePicker::for_manifests(current_dir)
        .with_title(format!("{action_name} Manifest"))
        .with_subtitle("Select a manifest file or press Esc to type the path manually")
        .with_prompt("Select manifest file:")
        .with_help_message(
            "Type to fuzzy-filter. Use re:<regex> for regex or lit:<text> for literal matching.",
        )
        .with_page_size(10);

    if let Some(path) = picker.show() {
        return Some(path);
    }

    println!();
    println!(
        "{}",
        "No file selected in picker. Type a path manually or press Enter to cancel."
            .color(Color::YELLOW)
    );

    prompt_manifest_path_text(manual_help, placeholder)
}

fn prompt_manifest_path_text(help_message: &str, placeholder: &str) -> Option<PathBuf> {
    match Text::new("Manifest file path:")
        .with_help_message(help_message)
        .with_placeholder(placeholder)
        .prompt()
    {
        Ok(path) => parse_manifest_input(&path),
        Err(_) => None,
    }
}

fn parse_manifest_input(raw: &str) -> Option<PathBuf> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(PathBuf::from(trimmed))
    }
}

fn should_enable_dry_run(selected_option: &str) -> bool {
    selected_option.trim_start().starts_with("Yes")
}

fn parse_output_directory_input(
    output_dir: Result<String, inquire::InquireError>,
) -> Option<PathBuf> {
    match output_dir {
        Ok(dir) => parse_manifest_input(&dir),
        Err(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_manifest_input, parse_output_directory_input, should_enable_dry_run};
    use inquire::InquireError;
    use std::path::PathBuf;

    #[test]
    fn parse_manifest_input_rejects_blank_values() {
        assert_eq!(parse_manifest_input(""), None);
        assert_eq!(parse_manifest_input("   "), None);
    }

    #[test]
    fn parse_manifest_input_trims_and_builds_path() {
        assert_eq!(
            parse_manifest_input("  ./ntk-manifest.yml  "),
            Some(PathBuf::from("./ntk-manifest.yml"))
        );
    }

    #[test]
    fn should_enable_dry_run_matches_yes_option_prefix() {
        assert!(should_enable_dry_run("Yes - Dry-run (preview only)"));
        assert!(should_enable_dry_run("   Yes - anything"));
        assert!(!should_enable_dry_run("No - Apply changes"));
    }

    #[test]
    fn parse_output_directory_input_parses_non_empty_path() {
        assert_eq!(
            parse_output_directory_input(Ok(" ./src ".to_string())),
            Some(PathBuf::from("./src"))
        );
    }

    #[test]
    fn parse_output_directory_input_returns_none_for_blank_or_error() {
        assert_eq!(parse_output_directory_input(Ok("   ".to_string())), None);
        assert_eq!(
            parse_output_directory_input(Err(InquireError::OperationCanceled)),
            None
        );
    }
}
