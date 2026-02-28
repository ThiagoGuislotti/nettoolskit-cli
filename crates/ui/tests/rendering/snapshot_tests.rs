//! Snapshot tests for UI rendering output
//!
//! Uses `insta` to capture deterministic output from formatting and rendering
//! functions. Run `cargo insta review` after test failures to review/accept
//! new or changed snapshots.

use nettoolskit_ui::core::formatting::format_menu_item as format_aligned;
use nettoolskit_ui::format_menu_item;
use nettoolskit_ui::get_prompt_string;
use nettoolskit_ui::get_prompt_symbol;
use nettoolskit_ui::{set_color_override, set_unicode_override, ColorLevel};

/// Force deterministic capability state so snapshot output
/// is consistent regardless of the test-runner environment.
fn force_caps() {
    set_color_override(Some(ColorLevel::Ansi16));
    set_unicode_override(Some(true));
}

// ─── format_menu_item (core::formatting — aligned variant) ───────────────

#[test]
fn snapshot_aligned_menu_item_standard() {
    force_caps();
    let result = format_aligned("/ help", "Display help information", 20);
    insta::assert_snapshot!(result);
}

#[test]
fn snapshot_aligned_menu_item_short_label() {
    force_caps();
    let result = format_aligned("/ q", "Quit", 20);
    insta::assert_snapshot!(result);
}

#[test]
fn snapshot_aligned_menu_item_long_label() {
    force_caps();
    let result = format_aligned("/ manifest apply --force", "Force apply manifest", 20);
    insta::assert_snapshot!(result);
}

#[test]
fn snapshot_aligned_menu_item_empty_description() {
    force_caps();
    let result = format_aligned("/ help", "", 20);
    insta::assert_snapshot!(result);
}

#[test]
fn snapshot_aligned_menu_item_narrow_column() {
    force_caps();
    let result = format_aligned("/ help", "Display help", 10);
    insta::assert_snapshot!(result);
}

// ─── format_menu_item (rendering::components::helpers variant) ───────────

#[test]
fn snapshot_menu_item_with_description() {
    force_caps();
    let result = format_menu_item("check", Some("Validate manifest file"));
    insta::assert_snapshot!(result);
}

#[test]
fn snapshot_menu_item_without_description() {
    force_caps();
    let result = format_menu_item("check", None);
    insta::assert_snapshot!(result);
}

#[test]
fn snapshot_menu_item_empty_description() {
    force_caps();
    let result = format_menu_item("check", Some(""));
    insta::assert_snapshot!(result);
}

// ─── prompt ──────────────────────────────────────────────────────────────

#[test]
fn snapshot_prompt_string() {
    force_caps();
    let result = get_prompt_string();
    insta::assert_snapshot!(result);
}

#[test]
fn snapshot_prompt_symbol_raw() {
    force_caps();
    let result = get_prompt_symbol();
    insta::assert_snapshot!(result);
}
