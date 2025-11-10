//! String utility function tests

use nettoolskit_utils::string::{truncate_directory, truncate_directory_with_middle};

#[test]
fn test_truncate_directory_no_truncation_needed() {
    let short_path = "C:\\short\\path";
    let result = truncate_directory(short_path, 50);
    assert_eq!(result, short_path);
}

#[test]
fn test_truncate_directory_basic_truncation() {
    let long_path = "C:\\very\\long\\path\\to\\some\\project";
    let result = truncate_directory(long_path, 25);
    assert!(result.len() <= 25);
    assert!(result.contains("..."));
}

#[test]
fn test_truncate_directory_unix_paths() {
    let unix_path = "/home/user/very/long/path/to/project";
    let result = truncate_directory(unix_path, 20);
    assert!(result.len() <= 20);
    assert!(result.contains("..."));
}

#[test]
fn test_truncate_directory_with_middle_no_truncation() {
    let short_path = "C:\\short\\path";
    let result = truncate_directory_with_middle(short_path, 50);
    assert_eq!(result, short_path);
}

#[test]
fn test_truncate_directory_with_middle_windows_path() {
    let long_path = "C:\\Users\\username\\Documents\\Projects\\NetToolsKit\\tools\\nettoolskit-cli";
    let result = truncate_directory_with_middle(long_path, 50);

    assert!(result.len() <= 50);
    assert!(result.contains("\\...\\"));
    assert!(result.starts_with("C:"));
    assert!(result.ends_with("nettoolskit-cli"));
}

#[test]
fn test_truncate_directory_with_middle_unix_path() {
    let unix_path = "/home/user/Documents/Projects/NetToolsKit/tools/nettoolskit-cli";
    let result = truncate_directory_with_middle(unix_path, 40);

    println!("Unix path: {}", unix_path);
    println!("Result: {}", result);
    println!("Result length: {}", result.len());

    assert!(result.len() <= 40);
    assert!(result.contains("/.../"));
    assert!(result.starts_with("/"));
    assert!(result.ends_with("nettoolskit-cli"));
}

#[test]
fn test_truncate_directory_with_middle_home_path() {
    let home_path = "~\\Documents\\Trabalho\\Pessoal\\Desenvolvimento\\Projetos\\NetToolsKit\\tools\\nettoolskit-cli";
    let result = truncate_directory_with_middle(home_path, 60);

    assert!(result.len() <= 60);
    assert!(result.contains("\\...\\"));
    assert!(result.starts_with("~"));
    assert!(result.ends_with("nettoolskit-cli"));

    // Should show reasonable amount of both start and end
    let parts: Vec<&str> = result.split("\\...\\").collect();
    assert_eq!(parts.len(), 2);
    assert!(!parts[0].is_empty());
    assert!(!parts[1].is_empty());
}

#[test]
fn test_truncate_directory_with_middle_very_short_limit() {
    let long_path = "C:\\very\\long\\path\\to\\project";
    let result = truncate_directory_with_middle(long_path, 10);

    println!("Long path: {}", long_path);
    println!("Result: {}", result);
    println!("Result length: {}", result.len());

    assert!(result.len() <= 10);
    // For very short limits, it should truncate but might not start with "..."
    assert!(!result.is_empty());
}

#[test]
fn test_truncate_directory_with_middle_balanced_split() {
    let test_path = "~\\Documents\\Trabalho\\Pessoal\\Desenvolvimento\\Projetos\\NetToolsKit\\tools\\nettoolskit-cli";
    let result = truncate_directory_with_middle(test_path, 50);

    // Should have balanced front and back parts
    if result.contains("\\...\\") {
        let parts: Vec<&str> = result.split("\\...\\").collect();
        if parts.len() == 2 {
            // Back part should be longer than front (60% vs 35%)
            assert!(parts[1].len() >= parts[0].len());
        }
    }
}

#[test]
fn test_truncate_directory_with_middle_preserves_separators() {
    let windows_path = "C:\\Windows\\System32\\drivers\\etc";
    let result = truncate_directory_with_middle(windows_path, 25);

    if result.contains("\\...\\") {
        // Should not have double separators
        assert!(!result.contains("\\\\...\\\\"));
        // Should use proper Windows separators
        assert!(result.contains("\\"));
    }
}

#[test]
fn test_truncate_directory_with_middle_simple_paths() {
    let simple_path = "C:\\temp";
    let result = truncate_directory_with_middle(simple_path, 10);

    // Simple paths should not be over-truncated
    if simple_path.len() <= 10 {
        assert_eq!(result, simple_path);
    }
}
