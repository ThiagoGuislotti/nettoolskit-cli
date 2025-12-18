//! Tests for the string utils module
//!
//! Validates directory path truncation utilities
//! with comprehensive test coverage

use nettoolskit_core::string_utils::string::{truncate_directory, truncate_directory_with_middle};

// Basic Truncation Tests (truncate_directory)

#[test]
fn test_truncate_directory_no_truncation_needed() {
    // Arrange
    let short_path = "C:\\short\\path";

    // Act
    let result = truncate_directory(short_path, 50);

    // Assert
    assert_eq!(result, short_path);
}

#[test]
fn test_truncate_directory_basic_truncation() {
    // Arrange
    let long_path = "C:\\very\\long\\path\\to\\some\\project";

    // Act
    let result = truncate_directory(long_path, 25);

    // Assert
    assert!(result.len() <= 25);
    assert!(result.contains("..."));
}

#[test]
fn test_truncate_directory_unix_paths() {
    // Arrange
    let unix_path = "/home/user/very/long/path/to/project";

    // Act
    let result = truncate_directory(unix_path, 20);

    // Assert
    assert!(result.len() <= 20);
    assert!(result.contains("..."));
}

// Middle Truncation Tests (truncate_directory_with_middle)

#[test]
fn test_truncate_directory_with_middle_no_truncation() {
    // Arrange
    let short_path = "C:\\short\\path";

    // Act
    let result = truncate_directory_with_middle(short_path, 50);

    // Assert
    assert_eq!(result, short_path);
}

#[test]
fn test_truncate_directory_with_middle_windows_path() {
    // Arrange
    let long_path = "C:\\Users\\username\\Documents\\Projects\\NetToolsKit\\tools\\nettoolskit-cli";

    // Act
    let result = truncate_directory_with_middle(long_path, 50);

    // Assert
    assert!(result.len() <= 50);
    assert!(result.contains("\\...\\"));
    assert!(result.starts_with("C:"));
    assert!(result.ends_with("nettoolskit-cli"));
}

#[test]
fn test_truncate_directory_with_middle_unix_path() {
    // Arrange
    let unix_path = "/home/user/Documents/Projects/NetToolsKit/tools/nettoolskit-cli";

    // Act
    let result = truncate_directory_with_middle(unix_path, 40);

    // Assert
    assert!(result.len() <= 40);
    assert!(result.contains("/.../"));
    assert!(result.starts_with('/'));
    assert!(result.ends_with("nettoolskit-cli"));
}

#[test]
fn test_truncate_directory_with_middle_home_path() {
    // Arrange
    let home_path = "~\\Documents\\Trabalho\\Pessoal\\Desenvolvimento\\Projetos\\NetToolsKit\\tools\\nettoolskit-cli";

    // Act
    let result = truncate_directory_with_middle(home_path, 60);

    // Assert
    assert!(result.len() <= 60);
    assert!(result.contains("\\...\\"));
    assert!(result.starts_with('~'));
    assert!(result.ends_with("nettoolskit-cli"));

    let parts: Vec<&str> = result.split("\\...\\").collect();
    assert_eq!(parts.len(), 2);
    assert!(!parts[0].is_empty());
    assert!(!parts[1].is_empty());
}

// Edge Cases Tests

#[test]
fn test_truncate_directory_with_middle_very_short_limit() {
    // Arrange
    let long_path = "C:\\very\\long\\path\\to\\project";

    // Act
    let result = truncate_directory_with_middle(long_path, 10);

    // Assert
    assert!(result.len() <= 10);
    assert!(!result.is_empty());
}

#[test]
fn test_truncate_directory_with_middle_balanced_split() {
    // Arrange
    let test_path = "~\\Documents\\Trabalho\\Pessoal\\Desenvolvimento\\Projetos\\NetToolsKit\\tools\\nettoolskit-cli";

    // Act
    let result = truncate_directory_with_middle(test_path, 50);

    // Assert
    if result.contains("\\...\\") {
        let parts: Vec<&str> = result.split("\\...\\").collect();
        if parts.len() == 2 {
            assert!(parts[1].len() >= parts[0].len());
        }
    }
}

#[test]
fn test_truncate_directory_with_middle_preserves_separators() {
    // Arrange
    let windows_path = "C:\\Windows\\System32\\drivers\\etc";

    // Act
    let result = truncate_directory_with_middle(windows_path, 25);

    // Assert
    if result.contains("\\...\\") {
        assert!(!result.contains("\\\\...\\\\"));
        assert!(result.contains('\\'));
    }
}

#[test]
fn test_truncate_directory_with_middle_simple_paths() {
    // Arrange
    let simple_path = "C:\\temp";

    // Act
    let result = truncate_directory_with_middle(simple_path, 10);

    // Assert
    if simple_path.len() <= 10 {
        assert_eq!(result, simple_path);
    }
}
