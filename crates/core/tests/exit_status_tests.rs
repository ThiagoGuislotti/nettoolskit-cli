//! Tests for `ExitStatus` enum
//!
//! Validates exit status conversions, display formatting, and behavior.

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

#[test]
fn test_exit_status_error_to_exit_code() {
    let exit_code: std::process::ExitCode = ExitStatus::Error.into();
    let _ = exit_code;
}

#[test]
fn test_exit_status_interrupted_to_exit_code() {
    let exit_code: std::process::ExitCode = ExitStatus::Interrupted.into();
    let _ = exit_code;
}

// Display Tests

#[test]
fn test_exit_status_display_success() {
    assert_eq!(format!("{}", ExitStatus::Success), "success");
}

#[test]
fn test_exit_status_display_error() {
    assert_eq!(format!("{}", ExitStatus::Error), "error");
}

#[test]
fn test_exit_status_display_interrupted() {
    assert_eq!(format!("{}", ExitStatus::Interrupted), "interrupted");
}

// Debug, Clone, Copy, PartialEq, Eq Tests

#[test]
fn test_exit_status_debug() {
    let debug = format!("{:?}", ExitStatus::Success);
    assert!(debug.contains("Success"));
}

#[test]
fn test_exit_status_clone_copy() {
    let status = ExitStatus::Error;
    let cloned = status;
    let copied = status; // Copy
    assert_eq!(status, cloned);
    assert_eq!(status, copied);
}

#[test]
fn test_exit_status_equality() {
    assert_eq!(ExitStatus::Success, ExitStatus::Success);
    assert_ne!(ExitStatus::Success, ExitStatus::Error);
    assert_ne!(ExitStatus::Error, ExitStatus::Interrupted);
}
