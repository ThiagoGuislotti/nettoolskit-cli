//! Menu UI tests
//!
//! Tests for manifest interactive menu functionality.

use nettoolskit_manifest::models::ManifestAction;
use nettoolskit_core::MenuEntry;

#[test]
fn test_manifest_action_menu_entries() {
    let check = ManifestAction::Check;
    assert_eq!(check.label(), "check");
    assert!(!check.description().is_empty());

    let render = ManifestAction::Render;
    assert_eq!(render.label(), "render");

    let apply = ManifestAction::Apply;
    assert_eq!(apply.label(), "apply");

    let back = ManifestAction::Back;
    assert_eq!(back.label(), "back");
}

#[test]
fn test_manifest_action_all_variants() {
    use nettoolskit_core::MenuProvider;
    let actions = ManifestAction::all_variants();
    assert_eq!(actions.len(), 4);
    assert!(actions.contains(&ManifestAction::Check));
    assert!(actions.contains(&ManifestAction::Render));
    assert!(actions.contains(&ManifestAction::Apply));
    assert!(actions.contains(&ManifestAction::Back));
}
