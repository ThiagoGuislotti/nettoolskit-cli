/// Handler for /manifest apply command
use crate::definitions::ExitStatus;
use nettoolskit_manifest::{ExecutionConfig, ManifestExecutor};
use owo_colors::OwoColorize;
use std::path::PathBuf;

/// Execute manifest application
///
/// # Arguments
/// * `manifest_path` - Path to manifest file
/// * `output_root` - Root directory for generated files (defaults to current dir)
/// * `dry_run` - If true, perform validation without making changes
///
/// # Returns
/// Exit status indicating success or failure
pub async fn execute_apply(
    manifest_path: PathBuf,
    output_root: Option<PathBuf>,
    dry_run: bool,
) -> ExitStatus {
    // Resolve output root
    let output_root = output_root.unwrap_or_else(|| {
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    });

    // Display execution plan
    println!("Manifest: {}", manifest_path.display());
    println!("Output root: {}", output_root.display());
    if dry_run {
        println!("{}", "DRY-RUN mode enabled (no files will be modified)".yellow());
    }
    println!();

    // Create execution config
    let config = ExecutionConfig {
        manifest_path: manifest_path.clone(),
        output_root,
        dry_run,
    };

    // Execute manifest
    println!("{}", "Executing Manifest".cyan().bold());
    println!("{}", "─".repeat(18).cyan());
    println!("⏳ Load → Validate → Guards → Templates → Change Plan → Execute");
    println!();

    let executor = ManifestExecutor::new();
    match executor.execute(config).await {
        Ok(summary) => {
            println!("{}", "✓ Manifest applied successfully".green());
            println!();

            // Display execution summary
            display_summary(&summary);

            ExitStatus::Success
        }
        Err(e) => {
            println!("{}", format!("✗ Manifest execution failed: {}", e).red().bold());
            ExitStatus::Error
        }
    }
}

/// Display execution summary
fn display_summary(summary: &nettoolskit_manifest::ExecutionSummary) {
    println!("{}", "Execution Summary".cyan().bold());
    println!("{}", "─".repeat(17).cyan());

    // Display notes
    if !summary.notes.is_empty() {
        println!("Notes:");
        for note in &summary.notes {
            println!("  • {}", note);
        }
        println!();
    }

    // Display created files
    if !summary.created.is_empty() {
        println!("{}", format!("Files created: {}", summary.created.len()).green());
        for path in &summary.created {
            println!("  + {}", path.display());
        }
        println!();
    }

    // Display updated files
    if !summary.updated.is_empty() {
        println!("{}", format!("Files updated: {}", summary.updated.len()).green());
        for path in &summary.updated {
            println!("  ~ {}", path.display());
        }
        println!();
    }

    // Display skipped files
    if !summary.skipped.is_empty() {
        println!("Files skipped: {}", summary.skipped.len());
        for (path, reason) in &summary.skipped {
            println!("  - {} ({})", path.display(), reason);
        }
        println!();
    }

    // Display statistics
    println!("{}", "Statistics".cyan().bold());
    println!("{}", "─".repeat(10).cyan());
    println!("Total operations: {}",
        summary.created.len() + summary.updated.len());
    println!("Skipped: {}", summary.skipped.len());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_apply_with_missing_manifest() {
        let manifest_path = PathBuf::from("nonexistent.yaml");

        let status = execute_apply(
            manifest_path,
            None,
            false,
        ).await;

        assert_eq!(status, ExitStatus::Error);
    }

    #[tokio::test]
    async fn test_apply_dry_run_creates_config() {
        let manifest_path = PathBuf::from("test.yaml");

        // Just verify the function can be called with dry-run flag
        // Actual behavior is tested in integration tests
        let _status = execute_apply(
            manifest_path,
            None,
            true,
        ).await;
    }
}
