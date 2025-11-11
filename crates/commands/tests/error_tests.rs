/// Tests for CommandError enum and error handling
///
/// Validates error type conversions, Display implementations,
/// and error propagation patterns.

use nettoolskit_commands::{CommandError, Result};

// Error Display Tests

#[test]
fn test_error_display_template_not_found() {
    // Arrange
    let error = CommandError::TemplateNotFound("test.hbs".to_string());

    // Act
    let display = error.to_string();

    // Assert
    assert_eq!(display, "template not found: test.hbs");
}

#[test]
fn test_error_display_invalid_command() {
    // Arrange
    let error = CommandError::InvalidCommand("/unknown".to_string());

    // Act
    let display = error.to_string();

    // Assert
    assert_eq!(display, "invalid command: /unknown");
}

#[test]
fn test_error_display_execution_failed() {
    // Arrange
    let error = CommandError::ExecutionFailed("timeout".to_string());

    // Act
    let display = error.to_string();

    // Assert
    assert_eq!(display, "execution failed: timeout");
}

#[test]
fn test_error_display_template_error() {
    // Arrange
    let error = CommandError::TemplateError("syntax error".to_string());

    // Act
    let display = error.to_string();

    // Assert
    assert_eq!(display, "template rendering failed: syntax error");
}

// Error Conversion Tests

#[test]
fn test_error_from_string() {
    // Act
    let error: CommandError = "custom error".to_string().into();

    // Assert
    assert_eq!(error.to_string(), "custom error");
}

#[test]
fn test_error_from_str() {
    // Act
    let error: CommandError = "another error".into();

    // Assert
    assert_eq!(error.to_string(), "another error");
}

#[test]
fn test_error_from_io_error() {
    // Arrange
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");

    // Act
    let error: CommandError = io_error.into();

    // Assert
    assert!(error.to_string().contains("io error"));
}

// Result Type Tests

#[test]
fn test_result_type_alias() {
    // Arrange
    fn returns_ok() -> Result<i32> {
        Ok(42)
    }

    fn returns_err() -> Result<i32> {
        Err(CommandError::Other("test error".to_string()))
    }

    // Assert
    assert!(returns_ok().is_ok());
    assert_eq!(returns_ok().unwrap(), 42);
    assert!(returns_err().is_err());
}

// Error Propagation Tests

#[test]
fn test_error_propagation() {
    // Arrange
    fn inner() -> Result<()> {
        Err(CommandError::InvalidCommand("test".to_string()))
    }

    fn outer() -> Result<()> {
        inner()?;
        Ok(())
    }

    // Act
    let result = outer();

    // Assert
    assert!(result.is_err());
    match result {
        Err(CommandError::InvalidCommand(msg)) => assert_eq!(msg, "test"),
        _ => panic!("Expected InvalidCommand error"),
    }
}

#[test]
fn test_error_debug_format() {
    let error = CommandError::ExecutionFailed("debug test".to_string());
    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains("ExecutionFailed"));
    assert!(debug_str.contains("debug test"));
}