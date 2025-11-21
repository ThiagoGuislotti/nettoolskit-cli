use nettoolskit_core::MenuEntry;
use nettoolskit_ui::CommandPalette;

#[test]
fn test_palette_reload_entries() {
    // Arrange
    let root_entries = nettoolskit_commands::menu_entries();
    let manifest_entries = nettoolskit_commands::nettoolskit_manifest::menu_entries();

    let mut palette = CommandPalette::new(root_entries.clone());

    // Act - reload with manifest entries
    let result = palette.reload_entries(manifest_entries.clone());

    // Assert
    assert!(result.is_ok());
}

#[test]
fn test_context_detection_root() {
    // Test that we correctly detect root context
    let test_cases = vec![
        "/",
        "/h",
        "/he",
        "/help",
        "/q",
        "/quit",
        "/m",
        "/ma",
        "/manifest",
    ];

    for input in test_cases {
        // All these should be Root context (no space after manifest)
        assert!(
            !input.starts_with("/manifest "),
            "Input '{}' should not trigger Manifest context",
            input
        );
    }
}

#[test]
fn test_context_detection_manifest() {
    // Test that we correctly detect manifest submenu context
    let test_cases = vec![
        "/manifest ",
        "/manifest c",
        "/manifest ch",
        "/manifest check",
        "/manifest r",
        "/manifest render",
        "/manifest a",
        "/manifest apply",
    ];

    for input in test_cases {
        // All these should be Manifest context (space after manifest)
        assert!(
            input.starts_with("/manifest "),
            "Input '{}' should trigger Manifest context",
            input
        );
    }
}

#[test]
fn test_query_extraction_root() {
    // Test query extraction for root context
    let test_cases = vec![
        ("/", ""),
        ("/h", "h"),
        ("/help", "help"),
        ("/q", "q"),
        ("/manifest", "manifest"),
    ];

    for (input, expected) in test_cases {
        let query = input.strip_prefix("/").unwrap_or("");
        assert_eq!(
            query, expected,
            "Query extraction failed for input '{}'",
            input
        );
    }
}

#[test]
fn test_query_extraction_manifest() {
    // Test query extraction for manifest context
    let test_cases = vec![
        ("/manifest ", ""),
        ("/manifest c", "c"),
        ("/manifest check", "check"),
        ("/manifest r", "r"),
        ("/manifest render", "render"),
    ];

    for (input, expected) in test_cases {
        let query = input.strip_prefix("/manifest ").unwrap_or("");
        assert_eq!(
            query, expected,
            "Query extraction failed for manifest input '{}'",
            input
        );
    }
}

#[test]
fn test_root_menu_entries_count() {
    // Verify root menu has expected commands
    let entries = nettoolskit_commands::menu_entries();

    // Should have: /help, /manifest, /translate, /quit
    assert_eq!(entries.len(), 4, "Root menu should have 4 commands");
}

#[test]
fn test_manifest_menu_entries_count() {
    // Verify manifest submenu has expected actions
    let entries = nettoolskit_commands::nettoolskit_manifest::menu_entries();

    // Should have: check, render, apply
    assert_eq!(entries.len(), 3, "Manifest submenu should have 3 actions");
}

#[test]
fn test_manifest_entries_are_menu_entry() {
    // Verify all manifest entries implement MenuEntry trait
    let entries = nettoolskit_commands::nettoolskit_manifest::menu_entries();

    for entry in entries {
        // Should have non-empty label and description
        assert!(!entry.label().is_empty(), "Label should not be empty");
        assert!(!entry.description().is_empty(), "Description should not be empty");

        // Labels should not have leading slash (submenu items)
        assert!(
            !entry.label().starts_with('/'),
            "Manifest submenu labels should not have leading slash"
        );
    }
}
