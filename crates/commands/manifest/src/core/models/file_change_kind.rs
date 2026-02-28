//! Kind of file operation

/// Kind of file operation
#[derive(Debug, Clone, Copy)]
pub enum FileChangeKind {
    /// New file creation.
    Create,
    /// Update to an existing file.
    Update,
}
