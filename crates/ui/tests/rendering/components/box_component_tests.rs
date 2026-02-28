use nettoolskit_ui::{render_box, BoxConfig, Color};

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

#[test]
fn test_box_config_with_title_prefix() {
    let config = BoxConfig::new("Title").with_title_prefix(">>>");

    assert_eq!(config.title_prefix, Some(">>>".to_string()));
    assert_eq!(config.title, "Title");
}

#[test]
fn test_box_config_add_footer_item() {
    let config = BoxConfig::new("Title")
        .add_footer_item("version", "1.0.0", Color::GREEN)
        .add_footer_item("directory", "/home/user", Color::WHITE);

    assert_eq!(config.footer_items.len(), 2);
    assert_eq!(config.footer_items[0].0, "version");
    assert_eq!(config.footer_items[0].1, "1.0.0");
    assert_eq!(config.footer_items[1].0, "directory");
}

#[test]
fn test_box_config_with_spacing_false() {
    let config = BoxConfig::new("Title").with_spacing(false);

    assert!(!config.add_spacing);
}

#[test]
fn test_box_config_width_clamp() {
    let config = BoxConfig::new("Title").with_width(5);

    assert_eq!(config.width, 10);
}

#[test]
fn test_box_config_width_exact_minimum() {
    let config = BoxConfig::new("Title").with_width(10);

    assert_eq!(config.width, 10);
}

#[test]
fn test_box_config_debug() {
    let config = BoxConfig::new("Debug Test");
    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("Debug Test"));
}

#[test]
fn test_box_config_clone() {
    let config = BoxConfig::new("Clone Test")
        .with_subtitle("sub")
        .add_footer_item("key", "value", Color::WHITE);

    let cloned = config.clone();
    assert_eq!(cloned.title, "Clone Test");
    assert_eq!(cloned.subtitle, Some("sub".to_string()));
    assert_eq!(cloned.footer_items.len(), 1);
}

#[test]
fn test_box_config_chained_builder() {
    let config = BoxConfig::new("Full")
        .with_title_color(Color::CYAN)
        .with_subtitle("sub")
        .with_title_prefix(">>>")
        .with_border_color(Color::GREEN)
        .with_width(60)
        .with_spacing(false)
        .add_footer_item("key", "val", Color::RED);

    assert_eq!(config.title, "Full");
    assert_eq!(config.width, 60);
    assert!(!config.add_spacing);
    assert_eq!(config.footer_items.len(), 1);
}

#[test]
fn test_render_box_basic() {
    let config = BoxConfig::new("Render Test").with_spacing(false);

    render_box(config);
}

#[test]
fn test_render_box_with_subtitle() {
    let config = BoxConfig::new("Title")
        .with_subtitle("A subtitle line")
        .with_spacing(false);

    render_box(config);
}

#[test]
fn test_render_box_with_prefix() {
    let config = BoxConfig::new("Title")
        .with_title_prefix("🔧")
        .with_spacing(false);

    render_box(config);
}

#[test]
fn test_render_box_with_footer_items() {
    let config = BoxConfig::new("Title")
        .add_footer_item("version", "1.0.0", Color::GREEN)
        .add_footer_item("directory", "/home/user/projects/myproject", Color::WHITE)
        .with_spacing(false);

    render_box(config);
}

#[test]
fn test_render_box_with_directory_truncation() {
    let long_dir = "/home/user/very/deeply/nested/directory/structure/that/is/quite/long/indeed";
    let config = BoxConfig::new("Title")
        .add_footer_item("directory", long_dir, Color::WHITE)
        .with_width(50)
        .with_spacing(false);

    render_box(config);
}

#[test]
fn test_render_box_with_long_non_directory_value() {
    let long_value = "a".repeat(200);
    let config = BoxConfig::new("Title")
        .add_footer_item("data", &long_value, Color::WHITE)
        .with_width(50)
        .with_spacing(false);

    render_box(config);
}

#[test]
fn test_render_box_all_options() {
    let config = BoxConfig::new("Full Box")
        .with_title_color(Color::CYAN)
        .with_subtitle("Subtitle text")
        .with_title_prefix(">>>")
        .with_border_color(Color::PURPLE)
        .with_width(80)
        .with_spacing(true)
        .add_footer_item("version", "2.0.0", Color::GREEN)
        .add_footer_item("directory", "/home/user", Color::WHITE);

    render_box(config);
}

#[test]
fn test_render_box_narrow_width() {
    let config = BoxConfig::new("X").with_width(10).with_spacing(false);

    render_box(config);
}

#[test]
fn test_render_box_no_spacing() {
    let config = BoxConfig::new("No Spacing").with_spacing(false);

    render_box(config);
}

#[test]
fn test_box_config_default_fields() {
    let config = BoxConfig::default();
    assert!(config.title.is_empty());
    assert!(config.subtitle.is_none());
    assert!(config.title_prefix.is_none());
    assert!(config.footer_items.is_empty());
    assert_eq!(config.width, 89);
    assert!(config.add_spacing);
}
