//! Language Strategy Factory Tests
//!
//! Tests for LanguageStrategyFactory validating language detection,
//! strategy creation, and language-specific configuration.

use nettoolskit_templating::{Language, LanguageStrategyFactory};
use std::sync::Arc;

// Language Parsing Tests

#[test]
fn test_language_parsing() {
    use std::str::FromStr;

    // Assert - Test all language variants
    assert_eq!(Language::from_str("dotnet").unwrap(), Language::DotNet);
    assert_eq!(Language::from_str("csharp").unwrap(), Language::DotNet);
    assert_eq!(Language::from_str("cs").unwrap(), Language::DotNet);
    assert_eq!(Language::from_str("java").unwrap(), Language::Java);
    assert_eq!(Language::from_str("go").unwrap(), Language::Go);
    assert_eq!(Language::from_str("golang").unwrap(), Language::Go);
    assert_eq!(Language::from_str("python").unwrap(), Language::Python);
    assert_eq!(Language::from_str("py").unwrap(), Language::Python);
    assert_eq!(Language::from_str("rust").unwrap(), Language::Rust);
    assert_eq!(Language::from_str("rs").unwrap(), Language::Rust);
    assert_eq!(Language::from_str("clojure").unwrap(), Language::Clojure);
    assert_eq!(Language::from_str("clj").unwrap(), Language::Clojure);
    assert!(Language::parse("unknown").is_none());
}

// Factory Strategy Tests

#[test]
fn test_factory_get_strategy() {
    // Arrange
    let factory = LanguageStrategyFactory::new();

    // Assert - All languages should have strategies
    assert!(factory.get_strategy(Language::DotNet).is_some());
    assert!(factory.get_strategy(Language::Java).is_some());
    assert!(factory.get_strategy(Language::Go).is_some());
    assert!(factory.get_strategy(Language::Python).is_some());
    assert!(factory.get_strategy(Language::Rust).is_some());
    assert!(factory.get_strategy(Language::Clojure).is_some());
}

#[test]
fn test_factory_get_strategy_by_name() {
    // Arrange
    let factory = LanguageStrategyFactory::new();

    // Assert - Get strategies by string name
    assert!(factory.get_strategy_by_name("dotnet").is_some());
    assert!(factory.get_strategy_by_name("java").is_some());
    assert!(factory.get_strategy_by_name("go").is_some());
    assert!(factory.get_strategy_by_name("python").is_some());
    assert!(factory.get_strategy_by_name("rust").is_some());
    assert!(factory.get_strategy_by_name("clojure").is_some());
    assert!(factory.get_strategy_by_name("unknown").is_none());
}

#[test]
fn test_factory_detect_from_path() {
    // Arrange
    let factory = LanguageStrategyFactory::new();

    // Assert - Critical: first path segment determines language
    assert!(factory
        .detect_from_path("dotnet/Domain/Entity.cs")
        .is_some());
    assert!(factory
        .detect_from_path("java/domain/Entity.java")
        .is_some());
    assert!(factory.detect_from_path("go/domain/entity.go").is_some());
    assert!(factory
        .detect_from_path("python/domain/entity.py")
        .is_some());
    assert!(factory.detect_from_path("rust/domain/entity.rs").is_some());
    assert!(factory
        .detect_from_path("clojure/domain/entity.clj")
        .is_some());
    assert!(factory.detect_from_path("unknown/file.xyz").is_none());
}

#[test]
fn test_factory_supported_languages() {
    // Arrange
    let factory = LanguageStrategyFactory::new();

    // Act
    let languages = factory.supported_languages();

    // Assert
    assert_eq!(languages.len(), 6);
    assert!(languages.contains(&Language::DotNet));
    assert!(languages.contains(&Language::Java));
    assert!(languages.contains(&Language::Go));
    assert!(languages.contains(&Language::Python));
    assert!(languages.contains(&Language::Rust));
    assert!(languages.contains(&Language::Clojure));
}

#[test]
fn test_factory_arc_clone_performance() {
    // Arrange
    let factory = LanguageStrategyFactory::new();

    // Act
    let strategy1 = factory.get_strategy(Language::DotNet).unwrap();
    let strategy2 = factory.get_strategy(Language::DotNet).unwrap();

    // Assert - Critical: Arc clones should share same instance
    assert_eq!(Arc::strong_count(&strategy1), Arc::strong_count(&strategy2));
}
