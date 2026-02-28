//! Template rendering errors

use thiserror::Error;

/// Specialized Result type for template operations
pub type TemplateResult<T> = Result<T, TemplateError>;

/// Errors that can occur during template operations
#[derive(Debug, Error)]
pub enum TemplateError {
    /// Template file not found
    #[error("Template not found: {template}")]
    NotFound {
        /// Template path or name
        template: String,
    },

    /// Failed to read template file
    #[error("Failed to read template {path}: {source}")]
    ReadError {
        /// Template path
        path: String,
        /// Underlying I/O error
        source: std::io::Error,
    },

    /// Failed to register template with Handlebars
    #[error("Failed to register template {template}: {message}")]
    RegistrationError {
        /// Template name
        template: String,
        /// Handlebars error message
        message: String,
    },

    /// Failed to render template
    #[error("Failed to render template {template}: {message}")]
    RenderError {
        /// Template name
        template: String,
        /// Handlebars error message
        message: String,
    },
}
