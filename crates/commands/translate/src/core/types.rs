//! Core types for template translation

/// Translation request parameters
#[derive(Debug, Clone)]
pub struct TranslateRequest {
    /// Source language identifier (e.g., "csharp", "python")
    pub from: String,
    /// Target language identifier (e.g., "typescript", "rust")
    pub to: String,
    /// Template file path to translate
    pub path: String,
}
