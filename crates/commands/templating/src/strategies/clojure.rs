//! Clojure language strategy implementation

use async_trait::async_trait;
use super::language_strategy::{LanguageConventions, LanguageStrategy};

/// Clojure language strategy
#[derive(Debug, Clone)]
pub struct ClojureStrategy {
    conventions: LanguageConventions,
}

impl ClojureStrategy {
    pub fn new() -> Self {
        Self {
            conventions: LanguageConventions {
                source_dirs: vec!["src".to_string()],
                test_dirs: vec!["test".to_string()],
                skip_normalization: vec!["src".to_string(), "test".to_string(), "dev".to_string()],
            },
        }
    }
}

impl Default for ClojureStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LanguageStrategy for ClojureStrategy {
    fn language_id(&self) -> &str {
        "clojure"
    }

    fn conventions(&self) -> &LanguageConventions {
        &self.conventions
    }

    fn normalize_path(&self, path_parts: &[&str]) -> Option<String> {
        if self.is_normalized(path_parts) {
            return None;
        }

        // Insert "src/" after "clojure/" or "clj/"
        let lang = path_parts[0];
        let mut normalized = vec![lang, "src"];
        normalized.extend_from_slice(&path_parts[1..]);
        Some(normalized.join("/"))
    }

    fn file_extension(&self) -> &str {
        "clj"
    }
}
