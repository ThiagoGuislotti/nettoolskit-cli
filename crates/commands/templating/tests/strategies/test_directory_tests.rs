//! Test directory handling tests
//!
//! Validates that test/internal directories skip normalization.

use nettoolskit_templating::{
    DotNetStrategy, GoStrategy, JavaStrategy, LanguageStrategy, PythonStrategy,
};

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