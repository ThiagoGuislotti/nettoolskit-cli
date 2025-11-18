//! UI Integration Tests
//!
//! Integration tests that verify CLI and UI component interactions,
//! including input handling and command palette functionality.
use nettoolskit_ui::CommandPalette;
use nettoolskit_core::MenuEntry;

#[derive(Clone)]
struct TestEntry {
    label: String,
    description: String,
}

impl MenuEntry for TestEntry {
    fn label(&self) -> &str { &self.label }
    fn description(&self) -> &str { &self.description }
}

fn create_test_entries() -> Vec<TestEntry> {
    vec![TestEntry { label: "/test".to_string(), description: "Test".to_string() }]
}

// CommandPalette Creation Tests

#[test]
fn test_command_palette_creation() {
    // Arrange
    // (No setup needed)

    // Act
    let palette = CommandPalette::new(create_test_entries());

    // Assert
    assert!(!palette.is_active(), "Palette should start inactive");
}

// UI Exports Availability Tests

#[test]
fn test_ui_exports_available() {
    use nettoolskit_ui::{
        clear_terminal, CommandPalette, GRAY_COLOR, PRIMARY_COLOR, WHITE_COLOR,
    };
    use nettoolskit_cli::display::print_logo;

    // Arrange
    let expected_primary = (155, 114, 255);

    // Act
    let actual_primary = (PRIMARY_COLOR.0, PRIMARY_COLOR.1, PRIMARY_COLOR.2);
    let palette = CommandPalette::new(create_test_entries());

    // Assert
    assert_eq!(
        actual_primary, expected_primary,
        "PRIMARY_COLOR should match expected RGB"
    );
    assert!(!palette.is_active());
    // Compilation success proves clear_terminal, print_logo, GRAY_COLOR, WHITE_COLOR exist
    let _ = (clear_terminal, print_logo, GRAY_COLOR, WHITE_COLOR);
}

// UI Modules Accessibility Tests

#[test]
fn test_ui_modules_accessible() {
    use nettoolskit_ui::{colors, palette, terminal};
use nettoolskit_core::MenuEntry;

#[derive(Clone)]
struct TestEntry {
    label: String,
    description: String,
}

impl MenuEntry for TestEntry {
    fn label(&self) -> &str { &self.label }
    fn description(&self) -> &str { &self.description }
}

fn create_test_entries() -> Vec<TestEntry> {
    vec![TestEntry { label: "/test".to_string(), description: "Test".to_string() }]
}

    // Arrange
    // (Testing module visibility - no setup needed)

    // Act
    let color_constants = (colors::PRIMARY_COLOR, colors::SECONDARY_COLOR);
    let command_palette = palette::CommandPalette::new(create_test_entries());
    let terminal_layout = terminal::TerminalLayout::initialize::<fn()>(None);

    // Assert
    // Compilation success + successful creation proves modules are accessible
    assert_eq!(color_constants.0 .0, 155); // PRIMARY_COLOR.r
    assert!(!command_palette.is_active());
    let _ = terminal_layout;
}

// Feature-Gated Tests

#[cfg(feature = "modern-tui")]
#[test]
#[ignore = "modern module not yet implemented"]
fn test_modern_module_when_enabled() {
    // TODO: Implement when modern-tui feature is complete
    // use nettoolskit_ui::modern::{App, Tui};
use nettoolskit_core::MenuEntry;

#[derive(Clone)]
struct TestEntry {
    label: String,
    description: String,
}

impl MenuEntry for TestEntry {
    fn label(&self) -> &str { &self.label }
    fn description(&self) -> &str { &self.description }
}

fn create_test_entries() -> Vec<TestEntry> {
    vec![TestEntry { label: "/test".to_string(), description: "Test".to_string() }]
}

    // Arrange
    // (No setup needed)

    // Act
    // let app = App::new();
    // let tui_result = Tui::new();

    // Assert
    // assert!(
    //     tui_result.is_ok(),
    //     "Tui should be creatable when modern-tui feature is enabled"
    // );
    // let _ = app;
}
