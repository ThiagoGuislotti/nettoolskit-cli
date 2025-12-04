use nettoolskit_ui::{BoxConfig, Color};

#[test]
fn test_box_config_builder() {
    let config = BoxConfig::new("Test Title")
        .with_title_color(Color::WHITE)
        .with_subtitle("Test subtitle")
        .with_border_color(Color::PURPLE)
        .with_width(89);

    assert_eq!(config.title, "Test Title");
    assert_eq!(config.subtitle, Some("Test subtitle".to_string()));
    assert_eq!(config.width, 89);
}

#[test]
fn test_box_config_default() {
    let config = BoxConfig::default();
    assert_eq!(config.width, 89);
    assert!(config.add_spacing);
}
