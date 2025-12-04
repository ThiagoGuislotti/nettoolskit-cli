use nettoolskit_ui::format_menu_item as format_item_helper;

#[test]
fn test_format_menu_item_with_description() {
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
