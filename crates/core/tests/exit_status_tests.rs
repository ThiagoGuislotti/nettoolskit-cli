//! Tests for ExitStatus enum
//!
//! Validates exit status conversions and behavior.

use nettoolskit_core::ExitStatus;

#[test]
fn test_exit_status_to_i32() {
    // Arrange
    let success = ExitStatus::Success;
    let error = ExitStatus::Error;
    let interrupted = ExitStatus::Interrupted;

    // Act
    let success_code: i32 = success.into();
    let error_code: i32 = error.into();
    let interrupted_code: i32 = interrupted.into();

    // Assert
    assert_eq!(success_code, 0);
    assert_eq!(error_code, 1);
    assert_eq!(interrupted_code, 130);
}

#[test]
fn test_exit_status_to_exit_code() {
    // Arrange
    let success = ExitStatus::Success;

    // Act
    let exit_code: std::process::ExitCode = success.into();

    // Assert - Can't directly compare ExitCode, but we can create one
    let expected: std::process::ExitCode = std::process::ExitCode::from(0);
    // ExitCode doesn't implement PartialEq, so we just verify it compiles
    let _ = exit_code;
    let _ = expected;
}
