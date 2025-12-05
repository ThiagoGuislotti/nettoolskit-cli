///! Kind of file operation

/// Kind of file operation
#[derive(Debug, Clone, Copy)]
pub enum FileChangeKind {
    Create,
    Update,
}
