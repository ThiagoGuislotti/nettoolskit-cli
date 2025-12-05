///! Summary of manifest execution

use owo_colors::OwoColorize;
use std::path::PathBuf;

/// Summary of manifest execution
#[derive(Default, Debug)]
pub struct ExecutionSummary {
    pub created: Vec<PathBuf>,
    pub updated: Vec<PathBuf>,
    pub skipped: Vec<(PathBuf, String)>,
    pub notes: Vec<String>,
}

impl ExecutionSummary {
    /// Print summary to console
    pub fn print(&self, dry_run: bool) {
        if dry_run {
            println!("{}", "Plan summary (dry-run)".bold().yellow());
        } else {
            println!("{}", "Plan summary".bold().green());
        }

        if !self.created.is_empty() {
            println!("{}", "Created files:".bold());
            for path in &self.created {
                println!("  {}", path.display());
            }
        }

        if !self.updated.is_empty() {
            println!("{}", "Updated files:".bold());
            for path in &self.updated {
                println!("  {}", path.display());
            }
        }

        if !self.skipped.is_empty() {
            println!("{}", "Skipped items:".bold());
            for (path, reason) in &self.skipped {
                println!("  {} ({})", path.display(), reason);
            }
        }

        if !self.notes.is_empty() {
            println!("{}", "Notes:".bold());
            for note in &self.notes {
                println!("  {}", note);
            }
        }

        if self.created.is_empty()
            && self.updated.is_empty()
            && self.skipped.is_empty()
            && self.notes.is_empty()
        {
            println!("{}", "No operations were scheduled.".italic().blue());
        }
    }
}
