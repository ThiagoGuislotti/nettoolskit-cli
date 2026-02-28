use nettoolskit_ui::{Color, EnumMenuConfig, MenuConfig};

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

#[test]
fn test_menu_config_defaults() {
    let config = MenuConfig::new("prompt", vec!["a"]);
    assert_eq!(config.prompt, "prompt");
    assert_eq!(config.cursor_color, Color::PURPLE);
    assert!(config.help_message.is_none());
    assert_eq!(config.page_size, 6);
}

#[test]
fn test_menu_config_page_size_clamp() {
    let config = MenuConfig::new("p", vec!["x"]).with_page_size(0);
    assert_eq!(config.page_size, 1);
}

#[test]
fn test_menu_config_debug_clone() {
    let config = MenuConfig::new("p", vec!["a", "b"]).with_help_message("help");
    let cloned = config.clone();
    assert_eq!(cloned.prompt, "p");
    assert_eq!(cloned.items, vec!["a", "b"]);
    assert_eq!(cloned.help_message.as_deref(), Some("help"));
    let _debug = format!("{:?}", cloned);
}

#[test]
fn test_enum_menu_config_defaults() {
    let config = EnumMenuConfig::new("Title", "Subtitle", "/dir");
    assert_eq!(config.title, "Title");
    assert_eq!(config.subtitle, "Subtitle");
    assert_eq!(config.current_dir, "/dir");
    assert_eq!(config.theme_color, Color::PURPLE);
    assert_eq!(config.width, 89);
    assert!(config.footer_items.is_empty());
}

#[test]
fn test_enum_menu_config_with_theme_color() {
    let config = EnumMenuConfig::new("T", "S", "/d").with_theme_color(Color::RED);
    assert_eq!(config.theme_color, Color::RED);
}

#[test]
fn test_enum_menu_config_with_width() {
    let config = EnumMenuConfig::new("T", "S", "/d").with_width(120);
    assert_eq!(config.width, 120);
}

#[test]
fn test_enum_menu_config_add_footer_items() {
    let config = EnumMenuConfig::new("T", "S", "/d")
        .add_footer_item("version", "1.0", Color::GREEN)
        .add_footer_item("branch", "main", Color::YELLOW);
    assert_eq!(config.footer_items.len(), 2);
    assert_eq!(config.footer_items[0].0, "version");
    assert_eq!(config.footer_items[0].1, "1.0");
    assert_eq!(config.footer_items[1].0, "branch");
}

#[test]
fn test_enum_menu_config_chained_builder() {
    let config = EnumMenuConfig::new("Menu", "Pick one", "/home")
        .with_theme_color(Color::GREEN)
        .with_width(100)
        .add_footer_item("env", "prod", Color::RED);
    assert_eq!(config.title, "Menu");
    assert_eq!(config.theme_color, Color::GREEN);
    assert_eq!(config.width, 100);
    assert_eq!(config.footer_items.len(), 1);
}

#[test]
fn test_enum_menu_config_debug_clone() {
    let config = EnumMenuConfig::new("T", "S", "/d").add_footer_item("k", "v", Color::WHITE);
    let cloned = config.clone();
    assert_eq!(cloned.title, "T");
    assert_eq!(cloned.footer_items.len(), 1);
    let _debug = format!("{:?}", cloned);
}
