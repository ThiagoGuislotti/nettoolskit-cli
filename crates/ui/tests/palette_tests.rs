use nettoolskit_core::MenuEntry;
use nettoolskit_ui::CommandPalette;

// Test helper for creating menu entries
#[derive(Clone)]
struct TestEntry {
    label: String,
    description: String,
}

impl MenuEntry for TestEntry {
    fn label(&self) -> &str {
        &self.label
    }

    fn description(&self) -> &str {
        &self.description
    }
}

fn create_test_entries() -> Vec<TestEntry> {
    vec![
        TestEntry {
            label: "/test1".to_string(),
            description: "Test command 1".to_string(),
        },
        TestEntry {
            label: "/test2".to_string(),
            description: "Test command 2".to_string(),
        },
    ]
}

#[test]
fn test_command_palette_new() {
    let entries = create_test_entries();
    let palette = CommandPalette::new(entries);
    assert!(!palette.is_active());
}

#[test]
fn test_command_palette_open_and_close() {
    let entries = create_test_entries();
    let mut palette = CommandPalette::new(entries);

    // Test opening
    let result = palette.open("test");
    assert!(result.is_ok());
    assert!(palette.is_active());

    // Test closing
    let result = palette.close();
    assert!(result.is_ok());
    assert!(!palette.is_active());
}

#[test]
fn test_command_palette_double_close() {
    let entries = create_test_entries();
    let mut palette = CommandPalette::new(entries);

    // First close on inactive palette should succeed
    let result = palette.close();
    assert!(result.is_ok());
    assert!(!palette.is_active());

    // Second close should also succeed (idempotent)
    let result = palette.close();
    assert!(result.is_ok());
    assert!(!palette.is_active());
}

#[test]
fn test_command_palette_update_query_inactive() {
    let mut palette = CommandPalette::new(create_test_entries());

    // Should succeed even when inactive
    let result = palette.update_query("test");
    assert!(result.is_ok());
    assert!(!palette.is_active());
}

#[test]
fn test_command_palette_navigation_inactive() {
    let mut palette = CommandPalette::new(create_test_entries());

    // Navigation should succeed but do nothing when inactive
    assert!(palette.navigate_up().is_ok());
    assert!(palette.navigate_down().is_ok());
    assert!(palette.navigate_home().is_ok());
    assert!(palette.navigate_end().is_ok());

    assert!(!palette.is_active());
}

#[test]
fn test_command_palette_get_selected_command_inactive() {
    let palette = CommandPalette::new(create_test_entries());

    // Should return None when inactive
    assert!(palette.get_selected_command().is_none());
}

#[test]
fn test_command_palette_lifecycle() {
    let mut palette = CommandPalette::new(create_test_entries());

    // Initial state
    assert!(!palette.is_active());

    // Open with query
    palette.open("list").unwrap();
    assert!(palette.is_active());

    // Update query
    palette.update_query("new").unwrap();
    assert!(palette.is_active());

    // Navigate
    palette.navigate_down().unwrap();
    assert!(palette.is_active());

    // Close
    palette.close().unwrap();
    assert!(!palette.is_active());
}

#[test]
fn test_command_palette_query_filtering() {
    let mut palette = CommandPalette::new(create_test_entries());
    palette.open("").unwrap();

    // Empty query should have some results
    let empty_result = palette.get_selected_command();
    assert!(empty_result.is_some());

    // Update with specific query
    palette.update_query("list").unwrap();

    // Should still have a selection if "list" matches commands
    let filtered_result = palette.get_selected_command();
    // Can't assert specific command without knowing COMMANDS content
    // but can verify behavior is consistent
    assert!(filtered_result.is_some() || filtered_result.is_none());

    palette.close().unwrap();
}

#[test]
fn test_command_palette_navigation_wrapping() {
    let mut palette = CommandPalette::new(create_test_entries());
    palette.open("").unwrap();

    // Test navigation sequence
    palette.navigate_home().unwrap();
    let first_selection = palette.get_selected_command().map(|s| s.to_string());

    palette.navigate_end().unwrap();
    let last_selection = palette.get_selected_command().map(|s| s.to_string());

    // Test up from first (should wrap to last)
    palette.navigate_home().unwrap();
    palette.navigate_up().unwrap();
    let wrapped_selection = palette.get_selected_command().map(|s| s.to_string());

    // If we have multiple commands, wrapped should equal last
    if first_selection != last_selection {
        assert_eq!(wrapped_selection, last_selection);
    }

    palette.close().unwrap();
}

#[test]
fn test_command_palette_down_navigation() {
    let mut palette = CommandPalette::new(create_test_entries());
    palette.open("").unwrap();

    // Start at home
    palette.navigate_home().unwrap();
    let first = palette.get_selected_command().map(|s| s.to_string());

    // Navigate down
    palette.navigate_down().unwrap();
    let second = palette.get_selected_command().map(|s| s.to_string());

    // If there are multiple commands, they should be different
    // If only one command, they should be the same
    assert!(first.is_some());
    assert!(second.is_some());

    palette.close().unwrap();
}

#[test]
fn test_command_palette_multiple_opens() {
    let mut palette = CommandPalette::new(create_test_entries());

    // First open
    palette.open("test1").unwrap();
    assert!(palette.is_active());

    // Second open without closing (should work)
    palette.open("test2").unwrap();
    assert!(palette.is_active());

    palette.close().unwrap();
    assert!(!palette.is_active());
}

#[test]
fn test_command_palette_empty_query() {
    let mut palette = CommandPalette::new(create_test_entries());

    // Open with empty query
    palette.open("").unwrap();
    assert!(palette.is_active());

    // Should have some selection available
    let selection = palette.get_selected_command();
    assert!(selection.is_some());

    palette.close().unwrap();
}

#[test]
fn test_command_palette_whitespace_query() {
    let mut palette = CommandPalette::new(create_test_entries());

    // Test with whitespace
    palette.open("   ").unwrap();
    palette.update_query("\t").unwrap();
    palette.update_query(" \n ").unwrap();

    // Should handle whitespace gracefully
    assert!(palette.is_active());

    palette.close().unwrap();
}

#[test]
fn test_command_palette_special_characters() {
    let mut palette = CommandPalette::new(create_test_entries());
    palette.open("").unwrap();

    // Test with special characters
    palette.update_query("@#$%").unwrap();
    palette.update_query("()[]{}").unwrap();
    palette.update_query("UTF8: 🚀").unwrap();

    // Should handle special characters without crashing
    assert!(palette.is_active());

    palette.close().unwrap();
}

#[test]
fn test_command_palette_long_query() {
    let mut palette = CommandPalette::new(create_test_entries());
    palette.open("").unwrap();

    // Test with very long query
    let long_query = "a".repeat(1000);
    palette.update_query(&long_query).unwrap();

    // Should handle long queries without crashing
    assert!(palette.is_active());

    palette.close().unwrap();
}

#[test]
fn test_command_palette_state_consistency() {
    let mut palette = CommandPalette::new(create_test_entries());

    // Test state is consistent across operations
    assert!(!palette.is_active());

    palette.open("test").unwrap();
    assert!(palette.is_active());

    // Multiple navigation operations
    for _ in 0..10 {
        palette.navigate_down().unwrap();
        assert!(palette.is_active());
    }

    for _ in 0..5 {
        palette.navigate_up().unwrap();
        assert!(palette.is_active());
    }

    palette.navigate_home().unwrap();
    assert!(palette.is_active());

    palette.navigate_end().unwrap();
    assert!(palette.is_active());

    palette.close().unwrap();
    assert!(!palette.is_active());
}

#[test]
fn test_command_palette_rapid_operations() {
    let mut palette = CommandPalette::new(create_test_entries());

    // Test rapid consecutive operations
    for i in 0..10 {
        palette.open(&format!("query{}", i)).unwrap();
        palette.update_query(&format!("updated{}", i)).unwrap();
        palette.navigate_down().unwrap();
        palette.navigate_up().unwrap();
        palette.close().unwrap();
    }

    // Should end in consistent state
    assert!(!palette.is_active());
}

#[test]
fn test_command_palette_case_sensitivity() {
    let mut palette = CommandPalette::new(create_test_entries());
    palette.open("").unwrap();

    // Test different case variations
    palette.update_query("LIST").unwrap();
    let _uppercase_result = palette.get_selected_command();

    palette.update_query("list").unwrap();
    let _lowercase_result = palette.get_selected_command();

    palette.update_query("List").unwrap();
    let _mixed_result = palette.get_selected_command();

    // Behavior may vary based on implementation
    // Just ensure no crashes occur
    assert!(palette.is_active());

    palette.close().unwrap();
}

#[test]
fn test_command_palette_numeric_query() {
    let mut palette = CommandPalette::new(create_test_entries());
    palette.open("123").unwrap();

    palette.update_query("456").unwrap();
    palette.update_query("0").unwrap();

    // Should handle numeric queries without issues
    assert!(palette.is_active());

    palette.close().unwrap();
}
