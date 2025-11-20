//! Error types for translation operations

use thiserror::Error;

/// Translation errors
#[derive(Debug, Error)]
pub enum TranslateError {
    /// Unknown source language
    #[error("Unknown source language: {0}")]
    UnknownSourceLanguage(String),

    /// Unknown target language
    #[error("Unknown target language: {0}")]
    UnknownTargetLanguage(String),

    /// Template file not found
    #[error("Template file not found: {0}")]
    TemplateNotFound(String),

    /// Translation not supported
    #[error("Translation from {from} to {to} is not supported")]
    UnsupportedTranslation { from: String, to: String },
}
