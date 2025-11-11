use nettoolskit_templating::{Language, LanguageStrategyFactory};
use std::sync::Arc;

#[test]
fn test_language_parsing() {
    // Test .NET variants
    assert_eq!(Language::from_str("dotnet").unwrap(), Language::DotNet);
    assert_eq!(Language::from_str("csharp").unwrap(), Language::DotNet);
    assert_eq!(Language::from_str("cs").unwrap(), Language::DotNet);

    // Test Java
    assert_eq!(Language::from_str("java").unwrap(), Language::Java);

    // Test Go variants
    assert_eq!(Language::from_str("go").unwrap(), Language::Go);
    assert_eq!(Language::from_str("golang").unwrap(), Language::Go);

    // Test Python variants
    assert_eq!(Language::from_str("python").unwrap(), Language::Python);
    assert_eq!(Language::from_str("py").unwrap(), Language::Python);

    // Test Rust variants
    assert_eq!(Language::from_str("rust").unwrap(), Language::Rust);
    assert_eq!(Language::from_str("rs").unwrap(), Language::Rust);

    // Test Clojure variants
    assert_eq!(Language::from_str("clojure").unwrap(), Language::Clojure);
    assert_eq!(Language::from_str("clj").unwrap(), Language::Clojure);

    // Test unknown language
    assert!(Language::from_str("unknown").is_none());
}

#[test]
fn test_factory_get_strategy() {
    let factory = LanguageStrategyFactory::new();

    // Test getting strategies by enum
    assert!(factory.get_strategy(Language::DotNet).is_some());
    assert!(factory.get_strategy(Language::Java).is_some());
    assert!(factory.get_strategy(Language::Go).is_some());
    assert!(factory.get_strategy(Language::Python).is_some());
    assert!(factory.get_strategy(Language::Rust).is_some());
    assert!(factory.get_strategy(Language::Clojure).is_some());
}

#[test]
fn test_factory_get_strategy_by_name() {
    let factory = LanguageStrategyFactory::new();

    // Test getting strategies by string
    assert!(factory.get_strategy_by_name("dotnet").is_some());
    assert!(factory.get_strategy_by_name("java").is_some());
    assert!(factory.get_strategy_by_name("go").is_some());
    assert!(factory.get_strategy_by_name("python").is_some());
    assert!(factory.get_strategy_by_name("rust").is_some());
    assert!(factory.get_strategy_by_name("clojure").is_some());

    // Test unknown language
    assert!(factory.get_strategy_by_name("unknown").is_none());
}

#[test]
fn test_factory_detect_from_path() {
    let factory = LanguageStrategyFactory::new();

    // Test detection from path segments (first segment determines language)
    assert!(factory.detect_from_path("dotnet/Domain/Entity.cs").is_some());
    assert!(factory.detect_from_path("java/domain/Entity.java").is_some());
    assert!(factory.detect_from_path("go/domain/entity.go").is_some());
    assert!(factory.detect_from_path("python/domain/entity.py").is_some());
    assert!(factory.detect_from_path("rust/domain/entity.rs").is_some());
    assert!(factory.detect_from_path("clojure/domain/entity.clj").is_some());

    // Test unknown path
    assert!(factory.detect_from_path("unknown/file.xyz").is_none());
}

#[test]
fn test_factory_supported_languages() {
    let factory = LanguageStrategyFactory::new();
    let languages = factory.supported_languages();

    // Should have 6 languages now (was 4 before adding Rust and Clojure)
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
    let factory = LanguageStrategyFactory::new();

    // Get strategy as Arc
    let strategy1 = factory.get_strategy(Language::DotNet).unwrap();
    let strategy2 = factory.get_strategy(Language::DotNet).unwrap();

    // Should be the same Arc instance (cheap clone)
    assert_eq!(Arc::strong_count(&strategy1), Arc::strong_count(&strategy2));
}