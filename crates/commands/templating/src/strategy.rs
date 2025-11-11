/// Language-specific path resolution strategies
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
                skip_normalization: vec![
                    "src".to_string(),
                    "test".to_string(),
                    "dev".to_string(),
                ],
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


