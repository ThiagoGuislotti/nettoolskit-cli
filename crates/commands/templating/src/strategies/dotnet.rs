//! .NET language strategy implementation

use async_trait::async_trait;
use super::language_strategy::{LanguageConventions, LanguageStrategy};

/// .NET language strategy
#[derive(Debug, Clone)]
pub struct DotNetStrategy {
    conventions: LanguageConventions,
}

impl DotNetStrategy {
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

impl Default for DotNetStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LanguageStrategy for DotNetStrategy {
    fn language_id(&self) -> &str {
        "dotnet"
    }

    fn conventions(&self) -> &LanguageConventions {
        &self.conventions
    }

    fn normalize_path(&self, path_parts: &[&str]) -> Option<String> {
        if self.is_normalized(path_parts) {
            return None;
        }

        // Insert "src/" after "dotnet/"
        let mut normalized = vec!["dotnet", "src"];
        normalized.extend_from_slice(&path_parts[1..]);
        Some(normalized.join("/"))
    }

    fn file_extension(&self) -> &str {
        "cs"
    }
}
