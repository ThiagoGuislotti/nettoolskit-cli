//! Convention and edge case tests
//!
//! Tests conventions() method and edge cases like single-segment paths.

use nettoolskit_templating::{
    DotNetStrategy, GoStrategy, JavaStrategy, LanguageStrategy, PythonStrategy,
};

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