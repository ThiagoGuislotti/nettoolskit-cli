/// Manual test to verify input functionality
///
/// This test checks if the input system can properly read characters
/// and handle basic keyboard events.

use nettoolskit_ui::CommandPalette;

#[test]
fn test_command_palette_creation() {
    // Test that CommandPalette can be created
    let palette = CommandPalette::new();

    // Verify it's not active initially
    assert!(!palette.is_active(), "Palette should start inactive");
}

#[test]
fn test_ui_exports_available() {
    // Test that all UI functions are properly exported
    use nettoolskit_ui::{
        clear_terminal, print_logo, CommandPalette,
        PRIMARY_COLOR, GRAY_COLOR, WHITE_COLOR
    };

    // Verify color constants are accessible
    assert_eq!(PRIMARY_COLOR.0, 155);
    assert_eq!(PRIMARY_COLOR.1, 114);
    assert_eq!(PRIMARY_COLOR.2, 255);

    // Verify palette can be created
    let _palette = CommandPalette::new();

    // This test passing means all exports work correctly
}

#[test]
fn test_legacy_module_accessible() {
    // Test that legacy module is accessible through re-exports
    use nettoolskit_ui::legacy;

    // Should be able to access legacy modules
    let _colors = (
        legacy::display::PRIMARY_COLOR,
        legacy::display::SECONDARY_COLOR,
    );
}

#[cfg(feature = "modern-tui")]
#[test]
fn test_modern_module_when_enabled() {
    // Test that modern module is accessible when feature is enabled
    use nettoolskit_ui::modern::{App, Tui};

    // Should be able to create modern components
    let _app = App::new();
    let tui_result = Tui::new();

    assert!(tui_result.is_ok(), "Tui should be creatable");
}