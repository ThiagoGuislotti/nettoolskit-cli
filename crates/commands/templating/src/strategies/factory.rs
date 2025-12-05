/// Factory pattern for language strategy instantiation
use super::clojure::ClojureStrategy;
use super::dotnet::DotNetStrategy;
use super::go::GoStrategy;
use super::java::JavaStrategy;
use super::language_strategy::LanguageStrategy;
use super::python::PythonStrategy;
use super::rust::RustStrategy;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

/// Supported languages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    DotNet,
    Java,
    Go,
    Python,
    Rust,
    Clojure,
}

impl FromStr for Language {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "dotnet" | "csharp" | "c#" | "cs" => Ok(Language::DotNet),
            "java" => Ok(Language::Java),
            "go" | "golang" => Ok(Language::Go),
            "python" | "py" => Ok(Language::Python),
            "rust" | "rs" => Ok(Language::Rust),
            "clojure" | "clj" => Ok(Language::Clojure),
            _ => Err(format!("Unknown language: {}", s)),
        }
    }
}

impl Language {
    /// Parse language from string identifier (convenience wrapper)
    pub fn parse(s: &str) -> Option<Self> {
        s.parse().ok()
    }

    /// Get canonical identifier
    pub fn as_str(&self) -> &str {
        match self {
            Language::DotNet => "dotnet",
            Language::Java => "java",
            Language::Go => "go",
            Language::Python => "python",
            Language::Rust => "rust",
            Language::Clojure => "clojure",
        }
    }
}

/// Factory for creating language-specific strategies
///
/// Implements singleton pattern with lazy initialization and thread-safe caching
pub struct LanguageStrategyFactory {
    strategies: HashMap<Language, Arc<dyn LanguageStrategy>>,
}

impl LanguageStrategyFactory {
    /// Create a new factory with all strategies registered
    pub fn new() -> Self {
        let mut strategies: HashMap<Language, Arc<dyn LanguageStrategy>> = HashMap::new();

        strategies.insert(Language::DotNet, Arc::new(DotNetStrategy::new()));
        strategies.insert(Language::Java, Arc::new(JavaStrategy::new()));
        strategies.insert(Language::Go, Arc::new(GoStrategy::new()));
        strategies.insert(Language::Python, Arc::new(PythonStrategy::new()));
        strategies.insert(Language::Rust, Arc::new(RustStrategy::new()));
        strategies.insert(Language::Clojure, Arc::new(ClojureStrategy::new()));

        Self { strategies }
    }

    /// Get strategy for a specific language
    ///
    /// # Performance
    /// - O(1) lookup using HashMap
    /// - Arc clone is cheap (just pointer + atomic increment)
    /// - Thread-safe via Arc (can be shared across threads)
    pub fn get_strategy(&self, language: Language) -> Option<Arc<dyn LanguageStrategy>> {
        self.strategies.get(&language).cloned()
    }

    /// Get strategy by string identifier
    pub fn get_strategy_by_name(&self, name: &str) -> Option<Arc<dyn LanguageStrategy>> {
        Language::parse(name).and_then(|lang| self.get_strategy(lang))
    }

    /// Try to detect language from path prefix
    ///
    /// # Example
    /// ```
    /// use nettoolskit_templating::LanguageStrategyFactory;
    ///
    /// let factory = LanguageStrategyFactory::new();
    /// let strategy = factory.detect_from_path("dotnet/Domain/Entity.cs.hbs");
    /// assert!(strategy.is_some());
    /// ```
    pub fn detect_from_path(&self, path: &str) -> Option<Arc<dyn LanguageStrategy>> {
        let first_segment = path.split('/').next()?;
        self.get_strategy_by_name(first_segment)
    }

    /// Get all registered languages
    pub fn supported_languages(&self) -> Vec<Language> {
        self.strategies.keys().copied().collect()
    }

    /// Check if a language is supported
    pub fn is_supported(&self, language: Language) -> bool {
        self.strategies.contains_key(&language)
    }
}

impl Default for LanguageStrategyFactory {
    fn default() -> Self {
        Self::new()
    }
}
