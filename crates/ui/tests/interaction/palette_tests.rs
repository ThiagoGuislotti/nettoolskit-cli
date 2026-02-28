//! Palette tests
//!
//! Tests for CommandPalette builder API and state management.

use nettoolskit_core::MenuEntry;
use nettoolskit_ui::CommandPalette;

/// Test entry implementing MenuEntry trait
struct TestEntry {
    label: String,
    description: String,
}

impl TestEntry {
    fn new(label: &str, description: &str) -> Self {
        Self {
            label: label.to_string(),
            description: description.to_string(),
        }
    }
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
fn test_palette_new_with_entries() {
    let entries = vec![
        TestEntry::new("/help", "Show help"),
        TestEntry::new("/manifest", "Manage manifests"),
    ];
    let _palette = CommandPalette::new(entries);
}

#[test]
fn test_palette_new_empty() {
    let entries: Vec<TestEntry> = vec![];
    let _palette = CommandPalette::new(entries);
}

#[test]
fn test_palette_with_title() {
    let entries = vec![TestEntry::new("/help", "Help")];
    let _palette = CommandPalette::new(entries).with_title("NetToolsKit Commands");
}

#[test]
fn test_palette_with_subtitle() {
    let entries = vec![TestEntry::new("/help", "Help")];
    let _palette = CommandPalette::new(entries).with_subtitle("Select a command");
}

#[test]
fn test_palette_with_directory() {
    let entries = vec![TestEntry::new("/help", "Help")];
    let _palette = CommandPalette::new(entries).with_directory("/home/user/project");
}

#[test]
fn test_palette_with_prompt() {
    let entries = vec![TestEntry::new("/help", "Help")];
    let _palette = CommandPalette::new(entries).with_prompt("Choose →");
}

#[test]
fn test_palette_chained_builder() {
    let entries = vec![
        TestEntry::new("/help", "Show help"),
        TestEntry::new("/manifest", "Manage manifests"),
        TestEntry::new("/translate", "Translate files"),
    ];
    let _palette = CommandPalette::new(entries)
        .with_title("Commands")
        .with_subtitle("Available commands")
        .with_directory("/workspace")
        .with_prompt("Select →");
}

#[test]
fn test_palette_reload_entries() {
    let initial = vec![TestEntry::new("/help", "Help")];
    let mut palette = CommandPalette::new(initial);

    let updated = vec![
        TestEntry::new("/manifest", "Manifests"),
        TestEntry::new("/translate", "Translate"),
        TestEntry::new("/quit", "Exit"),
    ];
    palette.reload_entries(updated);
}

#[test]
fn test_palette_reload_with_empty() {
    let initial = vec![TestEntry::new("/help", "Help")];
    let mut palette = CommandPalette::new(initial);

    let empty: Vec<TestEntry> = vec![];
    palette.reload_entries(empty);
}

#[test]
fn test_palette_builder_returns_self() {
    // Verify fluent API works and each builder method returns Self
    let entries = vec![TestEntry::new("/a", "A")];
    let palette = CommandPalette::new(entries)
        .with_title("T")
        .with_subtitle("S")
        .with_directory("D")
        .with_prompt("P");

    // If we get here, the builder chain compiled and ran without panics
    drop(palette);
}
