/// Tests for CommandError enum and error handling
///
/// Validates error type conversions, Display implementations,
/// and error propagation patterns.

use nettoolskit_commands::{CommandError, Result};

#[test]
fn test_error_display_template_not_found() {
    let error = CommandError::TemplateNotFound("test.hbs".to_string());
    assert_eq!(error.to_string(), "template not found: test.hbs");
}

#[test]
fn test_error_display_invalid_command() {
    let error = CommandError::InvalidCommand("/unknown".to_string());
    assert_eq!(error.to_string(), "invalid command: /unknown");
}

#[test]
fn test_error_display_execution_failed() {
    let error = CommandError::ExecutionFailed("timeout".to_string());
    assert_eq!(error.to_string(), "execution failed: timeout");
}

#[test]
fn test_error_display_template_error() {
    let error = CommandError::TemplateError("syntax error".to_string());
    assert_eq!(error.to_string(), "template rendering failed: syntax error");
}

#[test]
fn test_error_from_string() {
    let error: CommandError = "custom error".to_string().into();
    assert_eq!(error.to_string(), "custom error");
}

#[test]
fn test_error_from_str() {
    let error: CommandError = "another error".into();
    assert_eq!(error.to_string(), "another error");
}

#[test]
fn test_error_from_io_error() {
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let error: CommandError = io_error.into();
    assert!(error.to_string().contains("io error"));
}

#[test]
fn test_result_type_alias() {
    fn returns_ok() -> Result<i32> {
        Ok(42)
    }

    fn returns_err() -> Result<i32> {
        Err(CommandError::Other("test error".to_string()))
    }

    assert!(returns_ok().is_ok());
    assert_eq!(returns_ok().unwrap(), 42);
    assert!(returns_err().is_err());
}

#[test]
fn test_error_propagation() {
    fn inner() -> Result<()> {
        Err(CommandError::InvalidCommand("test".to_string()))
    }

    fn outer() -> Result<()> {
        inner()?;
        Ok(())
    }

    let result = outer();
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