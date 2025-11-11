/// Error types for manifest operations
use std::path::PathBuf;
use thiserror::Error;

/// Result type for manifest operations
pub type ManifestResult<T> = Result<T, ManifestError>;

/// Errors that can occur during manifest operations
#[derive(Debug, Error)]
pub enum ManifestError {
    /// Manifest file not found
    #[error("manifest not found: {path}")]
    ManifestNotFound { path: String },

    /// Failed to read manifest file
    #[error("failed to read manifest from {path}: {source}")]
    ReadError {
        path: String,
        source: std::io::Error,
    },

    /// Failed to parse manifest YAML
    #[error("failed to parse manifest: {0}")]
    ParseError(#[from] serde_yaml::Error),

    /// Manifest validation failed
    #[error("manifest validation failed: {0}")]
    ValidationError(String),

    /// Template not found
    #[error("template not found: {path}")]
    TemplateNotFound { path: String },

    /// Template rendering failed
    #[error("template rendering failed: {0}")]
    RenderError(String),

    /// Template rendering failed with details
    #[error("failed to render template '{template}': {reason}")]
    TemplateRenderError { template: String, reason: String },

    /// File system error
    #[error("file system error: {0}")]
    FsError(#[from] std::io::Error),

    /// Solution root not found
    #[error("solution root not found: {path}")]
    SolutionNotFound { path: PathBuf },

    /// Project not found
    #[error("project not found: {path}")]
    ProjectNotFound { path: PathBuf },

    /// Collision policy violation
    #[error("file collision detected: {path} (policy: {policy})")]
    CollisionDetected { path: PathBuf, policy: String },

    /// Missing required field
    #[error("missing required field: {field}")]
    MissingField { field: String },

    /// Validation error
    #[error("validation error: {0}")]
    Validation(String),

    /// Invalid configuration
    #[error("invalid configuration: {0}")]
    InvalidConfiguration(String),

    /// Generic error
    #[error("{0}")]
    Other(String),
}

impl From<String> for ManifestError {
    fn from(msg: String) -> Self {
        ManifestError::Other(msg)
    }
}

impl From<&str> for ManifestError {
    fn from(msg: &str) -> Self {
        ManifestError::Other(msg.to_string())
    }
}