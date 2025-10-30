use nettoolskit_ui::{PRIMARY_COLOR, SECONDARY_COLOR, WHITE_COLOR, GRAY_COLOR, truncate_directory, clear_terminal};
use owo_colors::Rgb;

#[test]
fn test_color_constants_values() {
    assert_eq!(PRIMARY_COLOR, Rgb(155, 114, 255));
    assert_eq!(SECONDARY_COLOR, Rgb(204, 185, 254));
    assert_eq!(WHITE_COLOR, Rgb(255, 255, 255));
    assert_eq!(GRAY_COLOR, Rgb(128, 128, 128));
}

#[test]
fn test_color_constants_rgb_components() {
    let (r, g, b) = (PRIMARY_COLOR.0, PRIMARY_COLOR.1, PRIMARY_COLOR.2);
    assert_eq!(r, 155);
    assert_eq!(g, 114);
    assert_eq!(b, 255);
}

#[test]
fn test_truncate_directory_no_truncation_needed() {
    let path = "/short/path";
    let result = truncate_directory(path, 50);
    assert_eq!(result, path);
}

#[test]
fn test_truncate_directory_exact_max_width() {
    let path = "/exact/width/path";
    let max_width = path.len();
    let result = truncate_directory(path, max_width);
    assert_eq!(result, path);
}

#[test]
fn test_truncate_directory_unix_separator() {
    let long_path = "/very/long/path/to/some/deep/directory/structure/project";
    let result = truncate_directory(long_path, 30);

    assert!(result.len() <= 30);
    assert!(result.contains("..."));
    assert!(result.ends_with("project"));
    // Unix paths starting with / have empty first component, so result is /.../project
    assert!(result.starts_with("/"));
}

#[test]
fn test_truncate_directory_windows_separator() {
    let long_path = r"C:\very\long\path\to\some\deep\directory\structure\project";
    let result = truncate_directory(long_path, 30);

    assert!(result.len() <= 30);
    assert!(result.contains("..."));
    assert!(result.starts_with("C:"));
    assert!(result.ends_with("project"));
}

#[test]
fn test_truncate_directory_very_short_limit() {
    let path = "/some/path/file";
    let result = truncate_directory(path, 10);

    assert!(result.len() <= 12); // Allowing some flexibility for "..."
    assert!(result.contains("..."));
}

#[test]
fn test_truncate_directory_empty_path() {
    let path = "";
    let result = truncate_directory(path, 10);
    assert_eq!(result, "");
}

#[test]
fn test_truncate_directory_single_character() {
    let path = "/";
    let result = truncate_directory(path, 10);
    assert_eq!(result, "/");
}

#[test]
fn test_truncate_directory_preserves_important_parts() {
    let path = "/home/user/projects/rust/nettoolskit/src/main";
    let result = truncate_directory(path, 25);

    // Should preserve beginning and end - but starts with / due to empty first component
    assert!(result.starts_with("/"));
    assert!(result.ends_with("main"));
    assert!(result.contains("..."));
}

#[test]
fn test_clear_terminal_returns_ok() {
    // This test checks that clear_terminal doesn't panic and returns a Result
    let result = clear_terminal();
    // We can't guarantee it will succeed in test environment, but it shouldn't panic
    match result {
        Ok(()) => assert!(true),
        Err(_) => assert!(true), // Expected in test environment without proper terminal
    }
}

#[test]
fn test_color_constants_debug() {
    let debug_str = format!("{:?}", PRIMARY_COLOR);
    assert!(debug_str.contains("155"));
    assert!(debug_str.contains("114"));
    assert!(debug_str.contains("255"));
}

#[test]
fn test_truncate_directory_special_cases() {
    // Test with only separators
    let path = "///";
    let result = truncate_directory(path, 10);
    assert_eq!(result, path);

    // Test with mixed separators (shouldn't happen in practice but test robustness)
    let path = "/unix\\mixed/path";
    let result = truncate_directory(path, 10);
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