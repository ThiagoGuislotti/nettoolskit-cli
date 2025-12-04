use nettoolskit_manifest::models::*;
use strum::IntoEnumIterator;

#[test]
fn test_action_name() {
    // Arrange
    let check = ManifestAction::Check;
    let render = ManifestAction::Render;
    let apply = ManifestAction::Apply;

    // Act
    let check_name = check.name();
    let render_name = render.name();
    let apply_name = apply.name();

    // Assert
    assert_eq!(check_name, "check");
    assert_eq!(render_name, "render");
    assert_eq!(apply_name, "apply");
}

#[test]
fn test_action_description() {
    // Arrange
    let actions = ManifestAction::iter().collect::<Vec<_>>();

    // Act & Assert
    for action in actions {
        assert!(!action.description().is_empty());
    }
}

#[test]
fn test_full_command() {
    // Arrange
    let check = ManifestAction::Check;

    // Act
    let full = check.full_command();

    // Assert
    assert_eq!(full, "/manifest check");
}

#[test]
fn test_get_action() {
    // Act
    let check = get_action("check");
    let render = get_action("render");
    let invalid = get_action("invalid");

    // Assert
    assert_eq!(check, Some(ManifestAction::Check));
    assert_eq!(render, Some(ManifestAction::Render));
    assert_eq!(invalid, None);
}

#[test]
fn test_enum_iteration() {
    // Act
    let actions: Vec<ManifestAction> = ManifestAction::iter().collect();

    // Assert
    assert_eq!(actions.len(), 3); // check, render, apply
}

#[test]
fn test_palette_entries() {
    // Act
    let entries = palette_entries();

    // Assert
    assert_eq!(entries.len(), 3);
    assert!(entries.iter().any(|(name, _)| *name == "check"));
    assert!(entries.iter().any(|(name, _)| *name == "render"));
    assert!(entries.iter().any(|(name, _)| *name == "apply"));
}

#[test]
fn test_menu_provider_all_variants() {
    use nettoolskit_core::MenuProvider;

    // Act
    let entries = ManifestAction::all_variants();

    // Assert
    assert_eq!(entries.len(), 3);
    assert!(entries.contains(&ManifestAction::Check));
    assert!(entries.contains(&ManifestAction::Render));
    assert!(entries.contains(&ManifestAction::Apply));
}

#[test]
fn test_menu_provider_menu_items() {
    use nettoolskit_core::MenuProvider;

    // Act
    let items = ManifestAction::menu_items();

    // Assert
    assert_eq!(items.len(), 3);
    assert!(items.iter().any(|item| item.starts_with("check -")));
    assert!(items.iter().any(|item| item.starts_with("render -")));
    assert!(items.iter().any(|item| item.starts_with("apply -")));

    // Verify format "label - description"
    for item in items {
        assert!(item.contains(" - "), "Menu item should be in format 'label - description'");
    }
}
