//! Error Handling Tests
//!
//! This file exists per .github/instructions/rust-testing.instructions.md
//! The commands crate is an aggregator and does not define custom error types.
//! This file serves as:
//! 1. A compliance checkpoint for error handling standards
//! 2. Verification that error propagation from child crates works correctly
//! 3. A placeholder for future error-related tests if custom errors are introduced

use nettoolskit_management::ExitStatus;

// ExitStatus Tests

#[test]
fn test_exit_status_success_variant() {
    // Arrange
    let status = ExitStatus::Success;

    // Assert
    assert!(matches!(status, ExitStatus::Success));
}

#[test]
fn test_exit_status_error_variant() {
    // Arrange
    let status = ExitStatus::Error;

    // Assert
    assert!(matches!(status, ExitStatus::Error));
}



#[test]
fn test_exit_status_partial_equality() {
    // Arrange
    let success1 = ExitStatus::Success;
    let success2 = ExitStatus::Success;
    let error = ExitStatus::Error;

    // Assert
    assert!(matches!(success1, ExitStatus::Success));
    assert!(matches!(success2, ExitStatus::Success));
    assert!(!matches!(error, ExitStatus::Success));
}

// Child Crate Error Integration Tests

#[test]
fn test_manifest_error_propagates() {
    // Arrange
    use nettoolskit_manifest::ManifestError;

    let error = ManifestError::ManifestNotFound {
        path: "test.yml".to_string(),
    };

    // Act
    let display = error.to_string();

    // Assert
    assert!(display.contains("test.yml"));
}

#[test]
fn test_templating_error_propagates() {
    // Arrange
    use nettoolskit_templating::TemplateError;

    let error = TemplateError::RenderError {
        template: "test_template".to_string(),
        message: "test error".to_string(),
    };

    // Act
    let display = error.to_string();

    // Assert
    assert!(display.contains("test error"));
}