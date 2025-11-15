//! Integration tests for nettoolskit-core crate
//! These tests verify that all modules work together correctly

use nettoolskit_core::string_utils::string;

#[test]
fn test_string_module_exists() {
    // Test that the string module is properly exported
    let test_path = "test/path";
    let result = string::truncate_directory(test_path, 100);
    assert_eq!(result, test_path);
}

#[test]
fn test_string_functions_accessibility() {
    // Test that both main functions are accessible
    let path = "C:\\test\\path";

    // Test truncate_directory
    let result1 = string::truncate_directory(path, 50);
    assert!(!result1.is_empty());

    // Test truncate_directory_with_middle
    let result2 = string::truncate_directory_with_middle(path, 50);
    assert!(!result2.is_empty());
}

#[test]
fn test_module_integration() {
    // Test integration between different parts of the utils module
    let long_path = "C:\\Users\\Test\\Documents\\VeryLongProjectName\\SubFolder\\AnotherFolder\\FinalDestination";

    let standard_truncate = string::truncate_directory(long_path, 40);
    let middle_truncate = string::truncate_directory_with_middle(long_path, 40);

    // Both should respect the length limit
    assert!(standard_truncate.len() <= 40);
    assert!(middle_truncate.len() <= 40);

    // Both should handle the same input without errors
    assert!(!standard_truncate.is_empty());
    assert!(!middle_truncate.is_empty());
}

#[test]
fn test_cross_platform_compatibility() {
    // Test Windows paths
    let windows_path = "C:\\Users\\Test\\Documents\\Project";
    let win_result = string::truncate_directory_with_middle(windows_path, 25);
    assert!(win_result.contains("\\") || win_result.len() <= 25);

    // Test Unix paths
    let unix_path = "/home/test/documents/project";
    let unix_result = string::truncate_directory_with_middle(unix_path, 25);
    assert!(unix_result.contains("/") || unix_result.len() <= 25);
}

#[test]
fn test_edge_cases_handling() {
    // Empty string
    let empty = "";
    assert_eq!(string::truncate_directory(empty, 10), empty);
    assert_eq!(string::truncate_directory_with_middle(empty, 10), empty);

    // Very long limit
    let short_path = "C:\\test";
    assert_eq!(string::truncate_directory(short_path, 1000), short_path);
    assert_eq!(
        string::truncate_directory_with_middle(short_path, 1000),
        short_path
    );

    // Single character paths
    let single = "C";
    assert_eq!(string::truncate_directory(single, 5), single);
    assert_eq!(string::truncate_directory_with_middle(single, 5), single);
}
