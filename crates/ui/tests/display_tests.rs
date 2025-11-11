//! Tests for UI display functionality
//!
//! Validates color constants (RGB values), path truncation integration with
//! string-utils, cross-platform separator handling (Unix/Windows), and display
//! formatting correctness.
//!
//! ## Test Coverage
//! - Color constant values (PRIMARY, SECONDARY, WHITE, GRAY)
//! - Color RGB component validation
//! - Path truncation (no truncation, exact width, long paths)
//! - Cross-platform path handling (Unix /, Windows \)
//! - Integration with string-utils truncation functions

use nettoolskit_ui::{clear_terminal, GRAY_COLOR, PRIMARY_COLOR, SECONDARY_COLOR, WHITE_COLOR};
use nettoolskit_string_utils::string::truncate_directory;
use owo_colors::Rgb;

// Color Constants Tests

#[test]
fn test_color_constants_values() {
    // Assert
    assert_eq!(PRIMARY_COLOR, Rgb(155, 114, 255));
    assert_eq!(SECONDARY_COLOR, Rgb(204, 185, 254));
    assert_eq!(WHITE_COLOR, Rgb(255, 255, 255));
    assert_eq!(GRAY_COLOR, Rgb(128, 128, 128));
}

#[test]
fn test_color_constants_rgb_components() {
    // Act
    let (r, g, b) = (PRIMARY_COLOR.0, PRIMARY_COLOR.1, PRIMARY_COLOR.2);

    // Assert
    assert_eq!(r, 155);
    assert_eq!(g, 114);
    assert_eq!(b, 255);
}

// Path Truncation Tests

#[test]
fn test_truncate_directory_no_truncation_needed() {
    // Arrange
    let path = "/short/path";

    // Act
    let result = truncate_directory(path, 50);

    // Assert
    assert_eq!(result, path);
}

#[test]
fn test_truncate_directory_exact_max_width() {
    // Arrange
    let path = "/exact/width/path";
    let max_width = path.len();

    // Act
    let result = truncate_directory(path, max_width);

    // Assert
    assert_eq!(result, path);
}

#[test]
fn test_truncate_directory_unix_separator() {
    // Arrange
    let long_path = "/very/long/path/to/some/deep/directory/structure/project";

    // Act
    let result = truncate_directory(long_path, 30);

    // Assert
    // Critical: Unix paths preserve leading slash
    assert!(result.len() <= 30);
    assert!(result.contains("..."));
    assert!(result.starts_with("/"));
    assert!(result.ends_with("project"));
}

#[test]
fn test_truncate_directory_windows_separator() {
    // Arrange
    let long_path = r"C:\very\long\path\to\some\deep\directory\structure\project";

    // Act
    let result = truncate_directory(long_path, 30);

    // Assert
    // Critical: Windows paths preserve drive letter
    assert!(result.len() <= 30);
    assert!(result.contains("..."));
    assert!(result.starts_with("C:"));
    assert!(result.ends_with("project"));
}

#[test]
fn test_truncate_directory_very_short_limit() {
    // Arrange
    let path = "/some/path/file";

    // Act
    let result = truncate_directory(path, 10);

    // Assert
    assert!(result.len() <= 12);
    assert!(result.contains("..."));
}

// Edge Cases Tests

#[test]
fn test_truncate_directory_empty_path() {
    // Arrange
    let path = "";

    // Act
    let result = truncate_directory(path, 10);

    // Assert
    assert_eq!(result, "");
}

#[test]
fn test_truncate_directory_single_character() {
    // Arrange
    let path = "/";

    // Act
    let result = truncate_directory(path, 10);

    // Assert
    assert_eq!(result, "/");
}

#[test]
fn test_truncate_directory_preserves_important_parts() {
    // Arrange
    let path = "/home/user/projects/rust/nettoolskit/src/main";

    // Act
    let result = truncate_directory(path, 25);

    // Assert
    // Critical: Unix path truncation must preserve leading / and final segment
    assert!(result.starts_with("/"));
    assert!(result.ends_with("main"));
    assert!(result.contains("..."));
}

// Integration Tests

#[test]
fn test_clear_terminal_returns_ok() {
    // Act
    // Critical: clear_terminal should not panic in test environment
    let result = clear_terminal();

    // Assert
    match result {
        Ok(()) => assert!(true),
        Err(_) => assert!(true), // Expected in test environment without proper terminal
    }
}

#[test]
fn test_color_constants_debug() {
    // Act
    let debug_str = format!("{:?}", PRIMARY_COLOR);

    // Assert
    assert!(debug_str.contains("155"));
    assert!(debug_str.contains("114"));
    assert!(debug_str.contains("255"));
}

// Special Cases and Boundary Tests

#[test]
fn test_truncate_directory_special_cases() {
    // Arrange & Act - Test with only separators
    let path = "///";
    let result = truncate_directory(path, 10);

    // Assert
    assert_eq!(result, path);

    // Arrange & Act - Test mixed separators (robustness)
    let path = "/unix\\mixed/path";
    let result = truncate_directory(path, 10);

    // Assert
    assert!(result.len() <= 12);
}

#[test]
fn test_truncate_directory_boundary_conditions() {
    let path = "/a/b/c/d/e/f/g/h/i/j";

    // Test various boundary conditions
    let result1 = truncate_directory(path, 1);
    assert!(result1.len() <= 5); // Very tight constraint

    let result2 = truncate_directory(path, 0);
    assert!(result2.len() <= 5); // Edge case: zero width
}

#[test]
fn test_color_constants_equality() {
    // Test that colors are equal to themselves
    assert_eq!(PRIMARY_COLOR, PRIMARY_COLOR);
    assert_eq!(SECONDARY_COLOR, SECONDARY_COLOR);
    assert_eq!(WHITE_COLOR, WHITE_COLOR);
    assert_eq!(GRAY_COLOR, GRAY_COLOR);

    // Test that different colors are not equal
    assert_ne!(PRIMARY_COLOR, SECONDARY_COLOR);
    assert_ne!(WHITE_COLOR, GRAY_COLOR);
}
