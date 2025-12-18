//! Helpers module tests
//!
//! Tests for custom Handlebars helpers.
//! Currently the helpers module is a placeholder, so these tests verify the module exists.

use nettoolskit_templating::TemplateEngine;

#[test]
fn test_helpers_module_exists() {
    // This test verifies that the helpers module can be imported
    // When custom helpers are added, this will be expanded to test them
    // Currently helpers.rs is a placeholder with only a private PLACEHOLDER constant
    let _engine = TemplateEngine::new();
}

#[test]
fn test_placeholder_module_documentation() {
    // Verify that the helpers module is documented and ready for future expansion
    // The module is intentionally minimal as per the original apply.rs design
    // Future phases will add custom helpers like to_kebab, to_snake, to_pascal_case
    let _engine = TemplateEngine::new();
}
