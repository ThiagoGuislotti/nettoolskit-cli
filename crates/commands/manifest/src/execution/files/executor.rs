/// File operations executor
use crate::core::error::ManifestResult;
use crate::core::models::{ExecutionSummary, FileChange, FileChangeKind};
use std::fs;
use std::path::PathBuf;

/// Execute file change plan
pub fn execute_plan(
    changes: Vec<FileChange>,
    dry_run: bool,
    summary: &mut ExecutionSummary,
) -> ManifestResult<()> {
    for change in changes {
        if dry_run {
            match change.kind {
                FileChangeKind::Create => {
                    summary.notes.push(format!(
                        "would create: {} ({})",
                        change.path.display(),
                        change.note.as_deref().unwrap_or("no note")
                    ));
                }
                FileChangeKind::Update => {
                    summary.notes.push(format!(
                        "would update: {} ({})",
                        change.path.display(),
                        change.note.as_deref().unwrap_or("no note")
                    ));
                }
            }
            continue;
        }

        // Ensure parent directory exists
        if let Some(parent) = change.path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Write file
        fs::write(&change.path, &change.content)?;

        match change.kind {
            FileChangeKind::Create => {
                summary.created.push(change.path.clone());
            }
            FileChangeKind::Update => {
                summary.updated.push(change.path.clone());
            }
        }
    }

    Ok(())
}

/// Ensure directory exists
pub fn ensure_directory(
    path: &PathBuf,
    dry_run: bool,
    summary: &mut ExecutionSummary,
    note: &str,
) -> ManifestResult<()> {
    if path.exists() {
        return Ok(());
    }

    if dry_run {
        summary.notes.push(format!(
            "create directory (dry-run): {} ({})",
            path.display(),
            note
        ));
        return Ok(());
    }

    fs::create_dir_all(path)?;
    summary
        .notes
        .push(format!("Created directory: {} ({})", path.display(), note));
    Ok(())
}
