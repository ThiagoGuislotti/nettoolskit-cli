//! Language Strategy Tests
//!
//! Tests for language-specific strategy implementations validating file extensions,
//! naming conventions, and language-specific template behaviors.

use nettoolskit_templating::{
    ClojureStrategy, DotNetStrategy, GoStrategy, JavaStrategy, LanguageStrategy, PythonStrategy,
    RustStrategy,
};

// Language Strategy Tests

#[test]
fn test_dotnet_strategy() {
    // Arrange
    let strategy = DotNetStrategy::default();

    // Act
    let normalized = strategy.normalize_path(&["dotnet", "Domain", "Entity.cs"]);
    let already_normalized = strategy.normalize_path(&["dotnet", "src", "Domain", "Entity.cs"]);

    // Assert
    assert_eq!(normalized, Some("dotnet/src/Domain/Entity.cs".to_string()));
    assert_eq!(already_normalized, None);
    assert_eq!(strategy.file_extension(), "cs");
}

#[test]
fn test_java_strategy() {
    // Arrange
    let strategy = JavaStrategy::default();

    // Act
    let normalized = strategy.normalize_path(&["java", "domain", "Entity.java"]);
    let already_normalized =
        strategy.normalize_path(&["java", "src", "main", "java", "domain", "Entity.java"]);

    // Assert
    assert_eq!(
        normalized,
        Some("java/src/main/java/domain/Entity.java".to_string())
    );
    assert_eq!(already_normalized, None);
    assert_eq!(strategy.file_extension(), "java");
}

#[test]
fn test_go_strategy() {
    // Arrange
    let strategy = GoStrategy::default();

    // Act
    let normalized = strategy.normalize_path(&["go", "domain", "entity.go"]);
    let already_normalized = strategy.normalize_path(&["go", "pkg", "domain", "entity.go"]);

    // Assert
    assert_eq!(normalized, Some("go/pkg/domain/entity.go".to_string()));
    assert_eq!(already_normalized, None);
    assert_eq!(strategy.file_extension(), "go");
}

#[test]
fn test_python_strategy() {
    // Arrange
    let strategy = PythonStrategy::default();

    // Act
    let normalized = strategy.normalize_path(&["python", "domain", "entity.py"]);
    let already_normalized = strategy.normalize_path(&["python", "src", "domain", "entity.py"]);

    // Assert
    assert_eq!(normalized, Some("python/src/domain/entity.py".to_string()));
    assert_eq!(already_normalized, None);
    assert_eq!(strategy.file_extension(), "py");
}

#[test]
fn test_rust_strategy() {
    // Arrange
    let strategy = RustStrategy::default();

    // Act
    let normalized = strategy.normalize_path(&["rust", "domain", "entity.rs"]);
    let already_normalized = strategy.normalize_path(&["rust", "src", "domain", "entity.rs"]);

    // Assert
    assert_eq!(normalized, Some("rust/src/domain/entity.rs".to_string()));
    assert_eq!(already_normalized, None);
    assert_eq!(strategy.file_extension(), "rs");
}

#[test]
fn test_clojure_strategy() {
    // Arrange
    let strategy = ClojureStrategy::default();

    // Act
    let normalized = strategy.normalize_path(&["clojure", "domain", "entity.clj"]);
    let already_normalized = strategy.normalize_path(&["clojure", "src", "domain", "entity.clj"]);

    // Assert
    assert_eq!(
        normalized,
        Some("clojure/src/domain/entity.clj".to_string())
    );
    assert_eq!(already_normalized, None);
    assert_eq!(strategy.file_extension(), "clj");
}

#[test]
fn test_dotnet_strategy_language_id() {
    // Arrange
    let strategy = DotNetStrategy::default();

    // Act & Assert
    assert_eq!(strategy.language_id(), "dotnet");
}

#[test]
fn test_java_strategy_language_id() {
    // Arrange
    let strategy = JavaStrategy::default();

    // Act & Assert
    assert_eq!(strategy.language_id(), "java");
}

#[test]
fn test_go_strategy_language_id() {
    // Arrange
    let strategy = GoStrategy::default();

    // Act & Assert
    assert_eq!(strategy.language_id(), "go");
}

#[test]
fn test_python_strategy_language_id() {
    // Arrange
    let strategy = PythonStrategy::default();

    // Act & Assert
    assert_eq!(strategy.language_id(), "python");
}

#[test]
fn test_dotnet_strategy_template_patterns() {
    // Arrange
    let strategy = DotNetStrategy::default();

    // Act
    let patterns = strategy.template_patterns();

    // Assert
    assert_eq!(patterns.len(), 2);
    assert!(patterns.contains(&"*.cs.hbs".to_string()));
    assert!(patterns.contains(&"**/*.cs.hbs".to_string()));
}

#[test]
fn test_java_strategy_template_patterns() {
    // Arrange
    let strategy = JavaStrategy::default();

    // Act
    let patterns = strategy.template_patterns();

    // Assert
    assert_eq!(patterns.len(), 2);
    assert!(patterns.contains(&"*.java.hbs".to_string()));
    assert!(patterns.contains(&"**/*.java.hbs".to_string()));
}

#[test]
fn test_go_strategy_template_patterns() {
    // Arrange
    let strategy = GoStrategy::default();

    // Act
    let patterns = strategy.template_patterns();

    // Assert
    assert_eq!(patterns.len(), 2);
    assert!(patterns.contains(&"*.go.hbs".to_string()));
    assert!(patterns.contains(&"**/*.go.hbs".to_string()));
}

#[test]
fn test_python_strategy_template_patterns() {
    // Arrange
    let strategy = PythonStrategy::default();

    // Act
    let patterns = strategy.template_patterns();

    // Assert
    assert_eq!(patterns.len(), 2);
    assert!(patterns.contains(&"*.py.hbs".to_string()));
    assert!(patterns.contains(&"**/*.py.hbs".to_string()));
}

#[test]
fn test_dotnet_strategy_nested_paths() {
    // Arrange
    let strategy = DotNetStrategy::default();

    // Act
    let normalized =
        strategy.normalize_path(&["dotnet", "Application", "Commands", "CreateOrder.cs"]);

    // Assert
    assert_eq!(
        normalized,
        Some("dotnet/src/Application/Commands/CreateOrder.cs".to_string())
    );
}

#[test]
fn test_java_strategy_nested_paths() {
    // Arrange
    let strategy = JavaStrategy::default();

    // Act
    let normalized = strategy.normalize_path(&["java", "com", "example", "domain", "Order.java"]);

    // Assert
    assert_eq!(
        normalized,
        Some("java/src/main/java/com/example/domain/Order.java".to_string())
    );
}

#[test]
fn test_go_strategy_nested_paths() {
    // Arrange
    let strategy = GoStrategy::default();

    // Act
    let normalized = strategy.normalize_path(&["go", "domain", "aggregates", "order.go"]);

    // Assert
    assert_eq!(
        normalized,
        Some("go/pkg/domain/aggregates/order.go".to_string())
    );
}

#[test]
fn test_python_strategy_nested_paths() {
    // Arrange
    let strategy = PythonStrategy::default();

    // Act
    let normalized = strategy.normalize_path(&["python", "domain", "entities", "order.py"]);

    // Assert
    assert_eq!(
        normalized,
        Some("python/src/domain/entities/order.py".to_string())
    );
}

#[test]
fn test_dotnet_strategy_test_directory_normalization() {
    // Arrange
    let strategy = DotNetStrategy::default();

    // Act
    let normalized = strategy.normalize_path(&["dotnet", "tests", "Domain", "EntityTests.cs"]);

    // Assert
    assert_eq!(normalized, None);
}

#[test]
fn test_java_strategy_test_directory_normalization() {
    // Arrange
    let strategy = JavaStrategy::default();

    // Act
    let normalized = strategy.normalize_path(&["java", "test", "domain", "OrderTest.java"]);

    // Assert
    assert_eq!(normalized, None);
}

#[test]
fn test_go_strategy_internal_directory_normalization() {
    // Arrange
    let strategy = GoStrategy::default();

    // Act
    let normalized = strategy.normalize_path(&["go", "internal", "handler", "order.go"]);

    // Assert
    assert_eq!(normalized, None);
}

#[test]
fn test_python_strategy_test_directory_normalization() {
    // Arrange
    let strategy = PythonStrategy::default();

    // Act
    let normalized = strategy.normalize_path(&["python", "tests", "domain", "test_order.py"]);

    // Assert
    assert_eq!(normalized, None);
}

#[test]
fn test_dotnet_strategy_single_segment_path() {
    // Arrange
    let strategy = DotNetStrategy::default();

    // Act
    let normalized = strategy.normalize_path(&["dotnet"]);

    // Assert
    assert_eq!(normalized, Some("dotnet/src".to_string()));
}

#[test]
fn test_java_strategy_conventions() {
    // Arrange
    let strategy = JavaStrategy::default();

    // Act
    let conventions = strategy.conventions();

    // Assert
    assert_eq!(conventions.source_dirs, vec!["src", "main", "java"]);
    assert_eq!(conventions.test_dirs, vec!["src", "test", "java"]);
    assert!(conventions.skip_normalization.contains(&"src".to_string()));
}

#[test]
fn test_go_strategy_conventions() {
    // Arrange
    let strategy = GoStrategy::default();

    // Act
    let conventions = strategy.conventions();

    // Assert
    assert_eq!(conventions.source_dirs, vec!["pkg"]);
    assert_eq!(conventions.test_dirs, vec!["internal"]);
    assert!(conventions.skip_normalization.contains(&"pkg".to_string()));
    assert!(conventions
        .skip_normalization
        .contains(&"internal".to_string()));
    assert!(conventions.skip_normalization.contains(&"cmd".to_string()));
}

#[test]
fn test_python_strategy_conventions() {
    // Arrange
    let strategy = PythonStrategy::default();

    // Act
    let conventions = strategy.conventions();

    // Assert
    assert_eq!(conventions.source_dirs, vec!["src"]);
    assert_eq!(conventions.test_dirs, vec!["tests"]);
    assert!(conventions.skip_normalization.contains(&"src".to_string()));
    assert!(conventions
        .skip_normalization
        .contains(&"tests".to_string()));
}

#[test]
fn test_dotnet_strategy_conventions() {
    // Arrange
    let strategy = DotNetStrategy::default();

    // Act
    let conventions = strategy.conventions();

    // Assert
    assert_eq!(conventions.source_dirs, vec!["src"]);
    assert_eq!(conventions.test_dirs, vec!["tests"]);
    assert!(conventions.skip_normalization.contains(&"src".to_string()));
    assert!(conventions
        .skip_normalization
        .contains(&"tests".to_string()));
}
