//! Tests for directory utilities
//!
//! Validates `get_current_directory` function for home path substitution

use nettoolskit_core::path_utils::directory::get_current_directory;

// Basic Functionality Tests

#[test]
fn test_get_current_directory_returns_non_empty() {
    // Act
    let dir = get_current_directory();

    // Assert
    assert!(!dir.is_empty(), "Current directory should not be empty");
}

#[test]
fn test_get_current_directory_contains_tilde_when_in_home() {
    // Arrange
    // This test checks if we're in a subdirectory of home

    // Act
    let dir = get_current_directory();

    // Assert
    // Should either be "~" or start with "~/" or "~\"
    let has_home_substitute = dir == "~" || dir.starts_with("~/") || dir.starts_with("~\\");

    // Note: This might fail if running outside home directory, which is acceptable
    if std::env::var("HOME").is_ok() || std::env::var("USERPROFILE").is_ok() {
        // If we have a home var, we should be able to process it
        assert!(
            has_home_substitute || !dir.contains("home") && !dir.contains("Users"),
            "Directory should use ~ for home or not contain home path: {dir}"
        );
    }
}

#[test]
fn test_get_current_directory_format() {
    // Act
    let dir = get_current_directory();

    // Assert
    // Should not have trailing slash (unless it's just "~")
    if dir != "~" {
        assert!(
            !dir.ends_with('/') && !dir.ends_with('\\'),
            "Directory should not have trailing separator: {dir}"
        );
    }
}

#[test]
fn test_get_current_directory_is_valid_path() {
    // Act
    let dir = get_current_directory();

    // Assert
    // Should be a valid string (no null bytes, valid UTF-8)
    assert!(
        !dir.contains('\0'),
        "Directory should not contain null bytes"
    );
    assert!(
        dir.chars().all(|c| !c.is_control() || c == '\\' || c == '/'),
        "Directory should contain valid path characters"
    );
}

// Edge Cases

#[test]
fn test_get_current_directory_consistency() {
    // Arrange
    // Call multiple times

    // Act
    let dir1 = get_current_directory();
    let dir2 = get_current_directory();

    // Assert
    assert_eq!(
        dir1, dir2,
        "Multiple calls should return the same directory"
    );
}

#[test]
fn test_get_current_directory_tilde_replacement() {
    // Arrange
    let dir = get_current_directory();

    // Act & Assert
    if dir.starts_with('~') {
        // If tilde is used, there should be no absolute home path in the result
        assert!(
            !dir.contains("home/") && !dir.contains("Users\\"),
            "Tilde should replace home path, not be combined with it: {dir}"
        );
    }
}
