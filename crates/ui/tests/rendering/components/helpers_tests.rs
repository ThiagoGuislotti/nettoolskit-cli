use nettoolskit_ui::format_menu_item as format_item_helper;
use nettoolskit_ui::{render_command, render_menu_instructions, render_section_title};
use nettoolskit_ui::{set_color_override, ColorLevel};

#[test]
fn test_format_menu_item_with_description() {
    set_color_override(Some(ColorLevel::Ansi16));
    let result = format_item_helper("check", Some("Validate manifest"));
    assert!(result.contains("check"));
    assert!(result.contains("Validate manifest"));
    assert!(result.contains("\x1b[90m")); // Gray color code
}

#[test]
fn test_format_menu_item_without_description() {
    let result = format_item_helper("check", None);
    assert_eq!(result, "check");
}

#[test]
fn test_format_menu_item_with_empty_description() {
    let result = format_item_helper("check", Some(""));
    assert_eq!(result, "check");
}

#[test]
fn test_format_menu_item_long_label() {
    let result = format_item_helper("a-very-long-command-name", Some("desc"));
    assert!(result.contains("a-very-long-command-name"));
    assert!(result.contains("desc"));
}

#[test]
fn test_render_command_does_not_panic() {
    render_command("manifest");
}

#[test]
fn test_render_command_empty_string() {
    render_command("");
}

#[test]
fn test_render_menu_instructions_does_not_panic() {
    render_menu_instructions();
}

#[test]
fn test_render_section_title_without_icon() {
    render_section_title("My Section", None);
}

#[test]
fn test_render_section_title_with_icon() {
    render_section_title("Artifacts", Some("📦"));
}

#[test]
fn test_render_section_title_empty_title() {
    render_section_title("", None);
}

#[test]
fn test_render_section_title_with_empty_icon() {
    render_section_title("Section", Some(""));
}
