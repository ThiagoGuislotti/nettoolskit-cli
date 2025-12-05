//! Nested path normalization tests
//!
//! Tests path normalization with deeply nested directory structures.

use nettoolskit_templating::{
    DotNetStrategy, GoStrategy, JavaStrategy, LanguageStrategy, PythonStrategy,
};

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