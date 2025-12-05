//! Language identifier tests
//!
//! Validates language_id() returns correct string identifier.

use nettoolskit_templating::{
    DotNetStrategy, GoStrategy, JavaStrategy, LanguageStrategy, PythonStrategy,
};

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