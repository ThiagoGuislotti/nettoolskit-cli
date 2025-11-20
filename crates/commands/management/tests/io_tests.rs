//! I/O Module Tests
//!
//! Tests for I/O utilities including output formatting and exit status.

use nettoolskit_management::io::{ExitStatus, OutputFormatter, TerminalOutputFormatter};

// ExitStatus Tests

#[test]
fn test_exit_status_success() {
    let status = ExitStatus::Success;
    assert_eq!(status, ExitStatus::Success);
}

#[test]
fn test_exit_status_error() {
    let status = ExitStatus::Error;
    assert_eq!(status, ExitStatus::Error);
}

#[test]
fn test_exit_status_debug() {
    let success = ExitStatus::Success;
    let error = ExitStatus::Error;
    assert_eq!(format!("{:?}", success), "Success");
    assert_eq!(format!("{:?}", error), "Error");
}

#[test]
fn test_exit_status_clone() {
    let status = ExitStatus::Success;
    let cloned = status.clone();
    assert_eq!(status, cloned);
}

#[test]
fn test_exit_status_copy() {
    let status = ExitStatus::Success;
    let copied = status;
    assert_eq!(status, copied);
}

// TerminalOutputFormatter Tests

#[test]
fn test_terminal_output_formatter_new() {
    let formatter = TerminalOutputFormatter::new();
    // Should create successfully
    drop(formatter);
}

#[test]
fn test_terminal_output_formatter_default() {
    let formatter = TerminalOutputFormatter::default();
    // Should create successfully via default
    drop(formatter);
}

#[test]
fn test_terminal_output_formatter_info() {
    let mut formatter = TerminalOutputFormatter::new();
    // Should not panic
    formatter.info("Test info message");
}

#[test]
fn test_terminal_output_formatter_success() {
    let mut formatter = TerminalOutputFormatter::new();
    // Should not panic
    formatter.success("Test success message");
}

#[test]
fn test_terminal_output_formatter_warning() {
    let mut formatter = TerminalOutputFormatter::new();
    // Should not panic
    formatter.warning("Test warning message");
}

#[test]
fn test_terminal_output_formatter_error() {
    let mut formatter = TerminalOutputFormatter::new();
    // Should not panic
    formatter.error("Test error message");
}

#[test]
fn test_terminal_output_formatter_section() {
    let mut formatter = TerminalOutputFormatter::new();
    // Should not panic
    formatter.section("Test Section");
}

#[test]
fn test_terminal_output_formatter_blank_line() {
    let mut formatter = TerminalOutputFormatter::new();
    // Should not panic
    formatter.blank_line();
}

#[test]
fn test_terminal_output_formatter_multiple_calls() {
    let mut formatter = TerminalOutputFormatter::new();
    // Should handle multiple sequential calls
    formatter.section("Section 1");
    formatter.info("Info 1");
    formatter.success("Success 1");
    formatter.blank_line();
    formatter.warning("Warning 1");
    formatter.error("Error 1");
}

#[test]
fn test_output_formatter_trait_implementation() {
    let mut formatter: Box<dyn OutputFormatter> = Box::new(TerminalOutputFormatter::new());
    // Should work through trait object
    formatter.info("Trait test");
    formatter.success("Trait success");
    formatter.warning("Trait warning");
    formatter.error("Trait error");
    formatter.section("Trait section");
    formatter.blank_line();
}
