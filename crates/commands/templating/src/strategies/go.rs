///! Go language strategy implementation

use async_trait::async_trait;
use super::language_strategy::{LanguageConventions, LanguageStrategy};

/// Go language strategy
#[derive(Debug, Clone)]
pub struct GoStrategy {
    conventions: LanguageConventions,
}

impl GoStrategy {
    pub fn new() -> Self {
        Self {
            conventions: LanguageConventions {
                source_dirs: vec!["pkg".to_string()],
                test_dirs: vec!["internal".to_string()],
                skip_normalization: vec![
                    "pkg".to_string(),
                    "internal".to_string(),
                    "cmd".to_string(),
                ],
            },
        }
    }
}

impl Default for GoStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LanguageStrategy for GoStrategy {
    fn language_id(&self) -> &str {
        "go"
    }

    fn conventions(&self) -> &LanguageConventions {
        &self.conventions
    }

    fn normalize_path(&self, path_parts: &[&str]) -> Option<String> {
        if self.is_normalized(path_parts) {
            return None;
        }

        // Insert "pkg/" after "go/" or "golang/"
        let lang = path_parts[0];
        let mut normalized = vec![lang, "pkg"];
        normalized.extend_from_slice(&path_parts[1..]);
        Some(normalized.join("/"))
    }

    fn file_extension(&self) -> &str {
        "go"
    }
}
