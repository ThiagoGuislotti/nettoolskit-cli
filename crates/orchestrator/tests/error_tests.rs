//! Error Handling Tests
//!
//! This file exists per .github/instructions/rust-testing.instructions.md
//! Tests error handling for orchestrator types, particularly ExitStatus.

use nettoolskit_orchestrator::ExitStatus;

// ExitStatus Tests

#[test]
fn test_exit_status_success_variant() {
    // Arrange
    let status = ExitStatus::Success;

    // Act
    let is_success = matches!(status, ExitStatus::Success);

    // Assert
    assert!(is_success, "ExitStatus::Success should match Success variant");
}

#[test]
fn test_exit_status_error_variant() {
    // Arrange
    let status = ExitStatus::Error;

    // Act
    let is_error = matches!(status, ExitStatus::Error);

    // Assert
    assert!(is_error, "ExitStatus::Error should match Error variant");
}

#[test]
fn test_exit_status_interrupted_variant() {
    // Arrange
    let status = ExitStatus::Interrupted;

    // Act
    let is_interrupted = matches!(status, ExitStatus::Interrupted);

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

#[test]
fn test_exit_status_debug() {
    // Arrange
    let status = ExitStatus::Error;

    // Act
    let debug_str = format!("{:?}", status);

    // Assert
    assert!(
        debug_str.contains("Error"),
        "Debug output should contain 'Error'"
    );
}

#[test]
fn test_exit_status_into_i32() {
    // Arrange & Act
    let success_code: i32 = ExitStatus::Success.into();
    let error_code: i32 = ExitStatus::Error.into();
    let interrupted_code: i32 = ExitStatus::Interrupted.into();

    // Assert
    assert_eq!(success_code, 0, "Success should convert to 0");
    assert_eq!(error_code, 1, "Error should convert to 1");
    assert_eq!(interrupted_code, 130, "Interrupted should convert to 130");
}
