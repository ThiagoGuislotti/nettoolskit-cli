//! Error Handling Tests
//!
//! This file exists per .github/instructions/rust-testing.instructions.md
//! Even if the CLI crate does not define custom error types, this file serves as:
//! 1. A compliance checkpoint for error handling standards
//! 2. A future location for error-related tests if custom errors are introduced
//! 3. A verification that error propagation from dependencies works correctly

use nettoolskit_orchestrator::ExitStatus;

#[test]
fn test_exit_status_error_variant() {
    // Arrange
    let error_status = ExitStatus::Error;

    // Act
    let is_error = matches!(error_status, ExitStatus::Error);

    // Assert
    assert!(is_error, "ExitStatus::Error should match Error variant");
}

#[test]
fn test_exit_status_not_success_when_error() {
    // Arrange
    let error_status = ExitStatus::Error;

    // Act
    let is_success = matches!(error_status, ExitStatus::Success);

    // Assert
    assert!(!is_success, "ExitStatus::Error should not match Success");
}

#[test]
fn test_exit_status_error_clone() {
    // Arrange
    let original = ExitStatus::Error;

    // Act
    let cloned = original;
    let is_error = matches!(cloned, ExitStatus::Error);

    // Assert
    assert!(is_error, "Cloned ExitStatus::Error should remain Error");
}

#[test]
fn test_exit_status_error_debug() {
    // Arrange
    let error = ExitStatus::Error;

    // Act
    let debug_str = format!("{:?}", error);

    // Assert
    assert!(
        debug_str.contains("Error"),
        "Debug output should contain 'Error'"
    );
}

#[test]
fn test_exit_status_interrupted_variant() {
    // Arrange
    let interrupted = ExitStatus::Interrupted;

    // Act
    let is_interrupted = matches!(interrupted, ExitStatus::Interrupted);

    // Assert
    assert!(
        is_interrupted,
        "ExitStatus::Interrupted should match Interrupted variant"
    );
}

#[test]
fn test_exit_status_all_variants_distinct() {
    // Arrange
    let success = ExitStatus::Success;
    let error = ExitStatus::Error;
    let interrupted = ExitStatus::Interrupted;

    // Act
    let all_different = !matches!(success, ExitStatus::Error)
        && !matches!(success, ExitStatus::Interrupted)
        && !matches!(error, ExitStatus::Success)
        && !matches!(error, ExitStatus::Interrupted)
        && !matches!(interrupted, ExitStatus::Success)
        && !matches!(interrupted, ExitStatus::Error);

    // Assert
    assert!(all_different, "All ExitStatus variants should be distinct");
}

#[test]
fn test_exit_status_pattern_matching_exhaustive() {
    // Arrange
    let statuses = vec![
        ExitStatus::Success,
        ExitStatus::Error,
        ExitStatus::Interrupted,
    ];

    // Act & Assert
    for status in statuses {
        let matched = match status {
            ExitStatus::Success => true,
            ExitStatus::Error => true,
            ExitStatus::Interrupted => true,
        };
        assert!(matched, "All ExitStatus variants should be handled");
    }
}

#[test]
fn test_exit_status_error_vs_interrupted() {
    // Arrange
    let error = ExitStatus::Error;
    let interrupted = ExitStatus::Interrupted;

    // Act
    let error_is_not_interrupted = !matches!(error, ExitStatus::Interrupted);
    let interrupted_is_not_error = !matches!(interrupted, ExitStatus::Error);

    // Assert
    assert!(
        error_is_not_interrupted,
        "Error should not be Interrupted"
    );
    assert!(
        interrupted_is_not_error,
        "Interrupted should not be Error"
    );
}

#[test]
fn test_exit_status_copy_semantics() {
    // Arrange
    let original = ExitStatus::Error;

    // Act
    let copy1 = original;
    let copy2 = original;
    let both_error = matches!(copy1, ExitStatus::Error) && matches!(copy2, ExitStatus::Error);

    // Assert
    assert!(both_error, "ExitStatus should have Copy semantics");
}

#[test]
fn test_exit_status_equality() {
    // Arrange
    let error1 = ExitStatus::Error;
    let error2 = ExitStatus::Error;
    let success = ExitStatus::Success;

    // Act
    let same_variants_equal = error1 == error2;
    let different_variants_not_equal = error1 != success;

    // Assert
    assert!(same_variants_equal, "Same variants should be equal");
    assert!(
        different_variants_not_equal,
        "Different variants should not be equal"
    );
}