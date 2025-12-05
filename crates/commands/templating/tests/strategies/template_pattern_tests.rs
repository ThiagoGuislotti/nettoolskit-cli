//! Template pattern tests
//!
//! Validates template_patterns() glob patterns for each language.

use nettoolskit_templating::{
    DotNetStrategy, GoStrategy, JavaStrategy, LanguageStrategy, PythonStrategy,
};

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