/// Template rendering errors
use std::fmt;

/// Specialized Result type for template operations
pub type TemplateResult<T> = Result<T, TemplateError>;

/// Errors that can occur during template operations
#[derive(Debug)]
pub enum TemplateError {
    /// Template file not found
    NotFound {
        /// Template path or name
        template: String,
    },

    /// Failed to read template file
    ReadError {
        /// Template path
        path: String,
        /// Underlying I/O error
        source: std::io::Error,
    },

    /// Failed to register template with Handlebars
    RegistrationError {
        /// Template name
        template: String,
        /// Handlebars error message
        message: String,
    },

    /// Failed to render template
    RenderError {
        /// Template name
        template: String,
        /// Handlebars error message
        message: String,
    },
}

impl fmt::Display for TemplateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TemplateError::NotFound { template } => {
                write!(f, "Template not found: {}", template)
            }
            TemplateError::ReadError { path, source } => {
                write!(f, "Failed to read template {}: {}", path, source)
            }
            TemplateError::RegistrationError { template, message } => {
                write!(f, "Failed to register template {}: {}", template, message)
            }
            TemplateError::RenderError { template, message } => {
                write!(f, "Failed to render template {}: {}", template, message)
            }
        }
    }
}

impl std::error::Error for TemplateError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            TemplateError::ReadError { source, .. } => Some(source),
            _ => None,
        }
    }
}
