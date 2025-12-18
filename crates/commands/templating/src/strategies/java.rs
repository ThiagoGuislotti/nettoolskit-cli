//! Java language strategy implementation

use async_trait::async_trait;
use super::language_strategy::{LanguageConventions, LanguageStrategy};

/// Java language strategy
#[derive(Debug, Clone)]
pub struct JavaStrategy {
    conventions: LanguageConventions,
}

impl JavaStrategy {
    pub fn new() -> Self {
        Self {
            conventions: LanguageConventions {
                source_dirs: vec!["src".to_string(), "main".to_string(), "java".to_string()],
                test_dirs: vec!["src".to_string(), "test".to_string(), "java".to_string()],
                skip_normalization: vec![
                    "src".to_string(),
                    "test".to_string(),
                    "tests".to_string(),
                ],
            },
        }
    }
}

impl Default for JavaStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LanguageStrategy for JavaStrategy {
    fn language_id(&self) -> &str {
        "java"
    }

    fn conventions(&self) -> &LanguageConventions {
        &self.conventions
    }

    fn normalize_path(&self, path_parts: &[&str]) -> Option<String> {
        if self.is_normalized(path_parts) {
            return None;
        }

        // Insert "src/main/java/" after "java/"
        let mut normalized = vec!["java", "src", "main", "java"];
        normalized.extend_from_slice(&path_parts[1..]);
        Some(normalized.join("/"))
    }

    fn file_extension(&self) -> &str {
        "java"
    }
}
