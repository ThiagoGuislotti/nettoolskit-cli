use nettoolskit_ui::rendering::components::helpers::format_menu_item;

#[test]
fn test_format_menu_item_with_description() {
    let result = format_menu_item("/ help", Some("Display help"));
    assert!(result.contains("/ help"));
    assert!(result.contains("Display help"));
}

#[test]
fn test_format_menu_item_long_label() {
    let result = format_menu_item("/ verylongcommand", Some("Description"));
    assert!(result.contains("/ verylongcommand"));
    assert!(result.contains("Description"));
}

#[test]
fn test_format_menu_item_no_description() {
    let result = format_menu_item("/ test", None);
    assert!(result.contains("/ test"));
    assert!(!result.contains(" - "));
}

#[test]
fn test_all_items_have_description_separator() {
    let items = vec![
        format_menu_item("/ help", Some("Help text")),
        format_menu_item("/ manifest", Some("Manifest text")),
        format_menu_item("/ translate", Some("Translate text")),
    ];

    // All items with descriptions should have a dash separator
    for item in items {
        assert!(item.contains(" - "));
    }
}
