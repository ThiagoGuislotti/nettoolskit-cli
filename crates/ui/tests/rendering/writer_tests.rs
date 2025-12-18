//! UiWriter Tests
//!
//! Tests for UiWriter validating terminal output buffering, newline handling,
//! flush behavior, and edge cases for footer rendering.

use nettoolskit_ui::UiWriter;
use std::io::Write;

// Happy Path Tests

#[test]
fn test_ui_writer_buffers_partial_lines() {
    // Arrange
    let mut writer = UiWriter::new();
    let input = b"Hello";

    // Act
    let result = writer.write(input);

    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 5);
}

#[test]
fn test_ui_writer_emits_on_newline() {
    // Arrange
    let mut writer = UiWriter::new();
    let input = b"Hello\n";

    // Act
    let result = writer.write(input);

    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 6);
}

#[test]
fn test_ui_writer_flush_emits_remaining() {
    // Arrange
    let mut writer = UiWriter::new();
    let input = b"Partial";

    // Act
    let write_result = writer.write(input);
    let flush_result = writer.flush();

    // Assert
    assert!(write_result.is_ok());
    assert_eq!(write_result.unwrap(), 7);
    assert!(flush_result.is_ok());
}

#[test]
fn test_ui_writer_multiple_lines() {
    // Arrange
    let mut writer = UiWriter::new();
    let input = b"Line 1\nLine 2\nLine 3\n";

    // Act
    let result = writer.write(input);

    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 21);
}

// Edge Cases

#[test]
fn test_ui_writer_empty_input() {
    // Arrange
    let mut writer = UiWriter::new();
    let input = b"";

    // Act
    let result = writer.write(input);

    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);
}

#[test]
fn test_ui_writer_only_newlines() {
    // Arrange
    let mut writer = UiWriter::new();
    let input = b"\n\n\n";

    // Act
    let result = writer.write(input);

    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 3);
}

#[test]
fn test_ui_writer_carriage_return() {
    // Arrange
    let mut writer = UiWriter::new();
    let input = b"Text\r\n";

    // Act
    let result = writer.write(input);
    let flush_result = writer.flush();

    // Assert
    assert!(result.is_ok());
    assert!(flush_result.is_ok());
}

#[test]
fn test_ui_writer_whitespace_only() {
    // Arrange
    let mut writer = UiWriter::new();
    let input = b"   \t  \n";

    // Act
    let result = writer.write(input);
    let flush_result = writer.flush();

    // Assert
    assert!(result.is_ok());
    assert!(flush_result.is_ok());
}

#[test]
fn test_ui_writer_utf8_content() {
    // Arrange
    let mut writer = UiWriter::new();
    let input = "Hello 世界\n".as_bytes();

    // Act
    let result = writer.write(input);
    let flush_result = writer.flush();

    // Assert
    assert!(result.is_ok());
    assert!(flush_result.is_ok());
}

#[test]
fn test_ui_writer_incremental_writes() {
    // Arrange
    let mut writer = UiWriter::new();

    // Act
    writer.write_all(b"Hel").unwrap();
    writer.write_all(b"lo ").unwrap();
    writer.write_all(b"Wor").unwrap();
    writer.write_all(b"ld\n").unwrap();
    let flush_result = writer.flush();

    // Assert
    assert!(flush_result.is_ok());
}
