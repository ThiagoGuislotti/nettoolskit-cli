///! Python language strategy implementation

use async_trait::async_trait;
use super::language_strategy::{LanguageConventions, LanguageStrategy};

/// Python language strategy
#[derive(Debug, Clone)]
pub struct PythonStrategy {
    conventions: LanguageConventions,
}

impl PythonStrategy {
    pub fn new() -> Self {
        Self {
            conventions: LanguageConventions {
                source_dirs: vec!["src".to_string()],
                test_dirs: vec!["tests".to_string()],
                skip_normalization: vec![
                    "src".to_string(),
                    "tests".to_string(),
                    "test".to_string(),
                ],
            },
        }
    }
}

impl Default for PythonStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LanguageStrategy for PythonStrategy {
    fn language_id(&self) -> &str {
        "python"
    }

    fn conventions(&self) -> &LanguageConventions {
        &self.conventions
    }

    fn normalize_path(&self, path_parts: &[&str]) -> Option<String> {
        if self.is_normalized(path_parts) {
            return None;
        }

        // Insert "src/" after "python/" or "py/"
        let lang = path_parts[0];
        let mut normalized = vec![lang, "src"];
        normalized.extend_from_slice(&path_parts[1..]);
        Some(normalized.join("/"))
    }

    fn file_extension(&self) -> &str {
        "py"
    }
}
