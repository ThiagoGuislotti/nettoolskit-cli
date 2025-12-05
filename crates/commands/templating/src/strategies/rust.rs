///! Rust language strategy implementation

use async_trait::async_trait;
use super::language_strategy::{LanguageConventions, LanguageStrategy};

/// Rust language strategy
#[derive(Debug, Clone)]
pub struct RustStrategy {
    conventions: LanguageConventions,
}

impl RustStrategy {
    pub fn new() -> Self {
        Self {
            conventions: LanguageConventions {
                source_dirs: vec!["src".to_string()],
                test_dirs: vec!["tests".to_string()],
                skip_normalization: vec![
                    "src".to_string(),
                    "tests".to_string(),
                    "benches".to_string(),
                    "examples".to_string(),
                ],
            },
        }
    }
}

impl Default for RustStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LanguageStrategy for RustStrategy {
    fn language_id(&self) -> &str {
        "rust"
    }

    fn conventions(&self) -> &LanguageConventions {
        &self.conventions
    }

    fn normalize_path(&self, path_parts: &[&str]) -> Option<String> {
        if self.is_normalized(path_parts) {
            return None;
        }

        // Insert "src/" after "rust/" or "rs/"
        let lang = path_parts[0];
        let mut normalized = vec![lang, "src"];
        normalized.extend_from_slice(&path_parts[1..]);
        Some(normalized.join("/"))
    }

    fn file_extension(&self) -> &str {
        "rs"
    }
}
