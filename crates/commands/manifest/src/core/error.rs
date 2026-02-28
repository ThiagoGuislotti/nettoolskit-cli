//! Error types for manifest operations
use std::path::PathBuf;
use thiserror::Error;

/// Result type for manifest operations
pub type ManifestResult<T> = Result<T, ManifestError>;

/// Errors that can occur during manifest operations
#[derive(Debug, Error)]
pub enum ManifestError {
    /// Manifest file not found
    #[error("manifest not found: {path}")]
    ManifestNotFound {
        /// Path to the missing manifest file.
        path: String,
    },

    /// Failed to read manifest file
    #[error("failed to read manifest from {path}: {source}")]
    ReadError {
        /// Path to the manifest file that could not be read.
        path: String,
        /// Underlying I/O error.
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
    TemplateNotFound {
        /// Path to the missing template.
        path: String,
    },

    /// Template rendering failed
    #[error("template rendering failed: {0}")]
    RenderError(String),

    /// Template rendering failed with details
    #[error("failed to render template '{template}': {reason}")]
    TemplateRenderError {
        /// Name of the template that failed to render.
        template: String,
        /// Reason for the rendering failure.
        reason: String,
    },

    /// File system error
    #[error("file system error: {0}")]
    FsError(#[from] std::io::Error),

    /// Solution root not found
    #[error("solution root not found: {path}")]
    SolutionNotFound {
        /// Path to the expected solution root.
        path: PathBuf,
    },

    /// Project not found
    #[error("project not found: {path}")]
    ProjectNotFound {
        /// Path to the missing project.
        path: PathBuf,
    },

    /// Collision policy violation
    #[error("file collision detected: {path} (policy: {policy})")]
    CollisionDetected {
        /// Path of the file that collided.
        path: PathBuf,
        /// Collision policy that was violated.
        policy: String,
    },

    /// Missing required field
    #[error("missing required field: {field}")]
    MissingField {
        /// Name of the missing field.
        field: String,
    },

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
