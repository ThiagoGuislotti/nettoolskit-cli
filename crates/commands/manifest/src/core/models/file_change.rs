//! File change operation

use super::file_change_kind::FileChangeKind;
use std::path::PathBuf;

/// File change operation
#[derive(Debug)]
pub struct FileChange {
    /// Target file path.
    pub path: PathBuf,
    /// File content to write.
    pub content: String,
    /// Whether this is a create or update.
    pub kind: FileChangeKind,
    /// Optional descriptive note.
    pub note: Option<String>,
}
