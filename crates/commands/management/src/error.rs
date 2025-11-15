use thiserror::Error;

/// Errors that can occur during command processing and execution.
///
/// Following the Codex pattern of using thiserror for library error types,
/// this enum provides type-safe error handling with descriptive messages.
#[derive(Error, Debug)]
pub enum CommandError {
    /// Template file not found
    #[error("template not found: {0}")]
    TemplateNotFound(String),

    /// Invalid command syntax or arguments
    #[error("invalid command: {0}")]
    InvalidCommand(String),

    /// Command execution failed
    #[error("execution failed: {0}")]
    ExecutionFailed(String),

    /// Template rendering error
    #[error("template rendering failed: {0}")]
    TemplateError(String),

    /// File system error
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// Async runtime error
    #[error("runtime error: {0}")]
    Runtime(String),

    /// Generic error for compatibility during migration
    #[error("{0}")]
    Other(String),
}

/// Result type alias using CommandError
pub type Result<T> = std::result::Result<T, CommandError>;

// Conversion from String
impl From<String> for CommandError {
    fn from(msg: String) -> Self {
        CommandError::Other(msg)
    }
}

// Conversion from &str
impl From<&str> for CommandError {
    fn from(msg: &str) -> Self {
        CommandError::Other(msg.to_string())
    }
}
