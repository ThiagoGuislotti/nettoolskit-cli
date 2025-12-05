//! Language strategy trait definition
//!
//! Defines the interface for language-specific template path resolution.

use async_trait::async_trait;

/// Language conventions for template organization
#[derive(Debug, Clone)]
pub struct LanguageConventions {
    /// Source directory segments (e.g., ["src"] for .NET, ["src", "main", "java"] for Java)
    pub source_dirs: Vec<String>,
    /// Test directory segments (e.g., ["tests"] for .NET, ["src", "test", "java"] for Java)
    pub test_dirs: Vec<String>,
    /// Common directory prefixes to skip normalization
    pub skip_normalization: Vec<String>,
}

/// Strategy pattern for language-specific template path resolution
#[async_trait]
pub trait LanguageStrategy: Send + Sync {
    /// Get the language identifier (e.g., "dotnet", "java", "go", "python")
    fn language_id(&self) -> &str;

    /// Get language conventions
    fn conventions(&self) -> &LanguageConventions;

    /// Normalize a template path according to language conventions
    ///
    /// # Arguments
    /// * `path_parts` - Template path split by '/'
    ///
    /// # Returns
    /// Normalized path with conventional directories inserted, or None if already normalized
    fn normalize_path(&self, path_parts: &[&str]) -> Option<String>;

    /// Check if path is already normalized (contains conventional directories)
    fn is_normalized(&self, path_parts: &[&str]) -> bool {
        if path_parts.len() <= 1 {
            return false;
        }

        let second = path_parts[1];
        self.conventions()
            .skip_normalization
            .iter()
            .any(|skip| skip == second)
    }

    /// Get file extension for this language (e.g., "cs", "java", "go", "py")
    fn file_extension(&self) -> &str;

    /// Get common template patterns for this language
    fn template_patterns(&self) -> Vec<String> {
        vec![
            format!("*.{}.hbs", self.file_extension()),
            format!("**/*.{}.hbs", self.file_extension()),
        ]
    }
}
