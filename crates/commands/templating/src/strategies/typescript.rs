//! TypeScript language strategy implementation

use async_trait::async_trait;

use super::language_strategy::{LanguageConventions, LanguageStrategy};

/// TypeScript language strategy
#[derive(Debug, Clone)]
pub struct TypeScriptStrategy {
    conventions: LanguageConventions,
}

impl TypeScriptStrategy {
    /// Create a new TypeScript strategy with default conventions.
    pub fn new() -> Self {
        Self {
            conventions: LanguageConventions {
                source_dirs: vec!["src".to_string()],
                test_dirs: vec!["tests".to_string()],
                skip_normalization: vec![
                    "src".to_string(),
                    "test".to_string(),
                    "tests".to_string(),
                    "lib".to_string(),
                ],
            },
        }
    }
}

impl Default for TypeScriptStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LanguageStrategy for TypeScriptStrategy {
    fn language_id(&self) -> &str {
        "typescript"
    }

    fn conventions(&self) -> &LanguageConventions {
        &self.conventions
    }

    fn normalize_path(&self, path_parts: &[&str]) -> Option<String> {
        if self.is_normalized(path_parts) {
            return None;
        }

        // Insert "src/" after "typescript/"
        let mut normalized = vec!["typescript", "src"];
        normalized.extend_from_slice(&path_parts[1..]);
        Some(normalized.join("/"))
    }

    fn file_extension(&self) -> &str {
        "ts"
    }
}
