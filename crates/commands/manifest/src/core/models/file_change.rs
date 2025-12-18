//! File change operation

use std::path::PathBuf;
use super::file_change_kind::FileChangeKind;

/// File change operation
#[derive(Debug)]
pub struct FileChange {
    pub path: PathBuf,
    pub content: String,
    pub kind: FileChangeKind,
    pub note: Option<String>,
}
