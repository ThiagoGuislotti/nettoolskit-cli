//! Error handling tests for nettoolskit-core
//!
//! This crate defines only a Result<T> type alias and does not
//! introduce custom error types. All error handling is delegated
//! to the underlying `std::io::Error` or other standard library errors.
//!
//! As per .github/instructions/rust-testing.instructions.md:
//! "Every crate MUST have `tests/error_tests.rs`"
//!
//! This file exists to document that nettoolskit-core follows the
//! error handling pattern of using standard library errors and does
//! not require additional error-specific tests.

use nettoolskit_core::Result;

#[test]
fn test_result_alias_exists() {
    // Arrange
    // (no setup needed)

    // Act
    let result: Result<i32> = Ok(42);

    // Assert
    assert!(matches!(result, Ok(42)));
}

#[test]
fn test_result_alias_with_error() {
    // Arrange
    let error_msg = "test error";

    // Act
    let result: Result<i32> = Err(anyhow::anyhow!(error_msg));

    // Assert
    let Err(error) = result else {
        panic!("Expected Err result");
    };

    assert_eq!(error.to_string(), error_msg);
}
