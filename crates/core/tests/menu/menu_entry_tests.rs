//! MenuEntry trait tests
//!
//! Tests for basic menu entry functionality.

use nettoolskit_core::MenuEntry;

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

#[test]
fn test_menu_entry_basic() {
    let entry = TestEntry {
        label: "test".to_string(),
        description: "Test entry".to_string(),
    };

    assert_eq!(entry.label(), "test");
    assert_eq!(entry.description(), "Test entry");
}

#[test]
fn test_menu_entry_empty_description() {
    let entry = TestEntry {
        label: "empty".to_string(),
        description: String::new(),
    };

    assert_eq!(entry.label(), "empty");
    assert_eq!(entry.description(), "");
}
