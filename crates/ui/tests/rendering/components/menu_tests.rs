use nettoolskit_ui::{MenuConfig, Color};

#[test]
fn test_menu_config_builder() {
    let items = vec!["Option 1", "Option 2", "Option 3"];
    let config = MenuConfig::new("Select an option:", items)
        .with_cursor_color(Color::RED)
        .with_help_message("Use arrow keys")
        .with_page_size(5);

    assert_eq!(config.prompt, "Select an option:");
    assert_eq!(config.items.len(), 3);
    assert_eq!(config.page_size, 5);
    assert_eq!(config.cursor_color, Color::RED);
}
