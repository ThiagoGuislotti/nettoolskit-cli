//! Helpers module tests
//!
//! Tests for custom Handlebars case-conversion helpers.
//! Category: Unit

use nettoolskit_templating::TemplateEngine;
use serde_json::json;

// ── to_kebab_case ──────────────────────────────────────────────────────────

#[tokio::test]
async fn test_to_kebab_case_from_pascal() {
    let engine = TemplateEngine::new();
    let data = json!({"name": "MyClassName"});
    let result = engine
        .render_from_string("{{to_kebab_case name}}", &data, "kebab_pascal".into())
        .await
        .unwrap();
    assert_eq!(result.trim(), "my-class-name");
}

#[tokio::test]
async fn test_to_kebab_case_from_snake() {
    let engine = TemplateEngine::new();
    let data = json!({"name": "my_class_name"});
    let result = engine
        .render_from_string("{{to_kebab_case name}}", &data, "kebab_snake".into())
        .await
        .unwrap();
    assert_eq!(result.trim(), "my-class-name");
}

#[tokio::test]
async fn test_to_kebab_case_single_word() {
    let engine = TemplateEngine::new();
    let data = json!({"name": "Order"});
    let result = engine
        .render_from_string("{{to_kebab_case name}}", &data, "kebab_single".into())
        .await
        .unwrap();
    assert_eq!(result.trim(), "order");
}

// ── to_snake_case ──────────────────────────────────────────────────────────

#[tokio::test]
async fn test_to_snake_case_from_pascal() {
    let engine = TemplateEngine::new();
    let data = json!({"name": "MyClassName"});
    let result = engine
        .render_from_string("{{to_snake_case name}}", &data, "snake_pascal".into())
        .await
        .unwrap();
    assert_eq!(result.trim(), "my_class_name");
}

#[tokio::test]
async fn test_to_snake_case_from_kebab() {
    let engine = TemplateEngine::new();
    let data = json!({"name": "my-class-name"});
    let result = engine
        .render_from_string("{{to_snake_case name}}", &data, "snake_kebab".into())
        .await
        .unwrap();
    assert_eq!(result.trim(), "my_class_name");
}

// ── to_pascal_case ─────────────────────────────────────────────────────────

#[tokio::test]
async fn test_to_pascal_case_from_snake() {
    let engine = TemplateEngine::new();
    let data = json!({"name": "my_class_name"});
    let result = engine
        .render_from_string("{{to_pascal_case name}}", &data, "pascal_snake".into())
        .await
        .unwrap();
    assert_eq!(result.trim(), "MyClassName");
}

#[tokio::test]
async fn test_to_pascal_case_from_kebab() {
    let engine = TemplateEngine::new();
    let data = json!({"name": "my-class-name"});
    let result = engine
        .render_from_string("{{to_pascal_case name}}", &data, "pascal_kebab".into())
        .await
        .unwrap();
    assert_eq!(result.trim(), "MyClassName");
}

#[tokio::test]
async fn test_to_pascal_case_already_pascal() {
    let engine = TemplateEngine::new();
    let data = json!({"name": "MyClassName"});
    let result = engine
        .render_from_string("{{to_pascal_case name}}", &data, "pascal_noop".into())
        .await
        .unwrap();
    assert_eq!(result.trim(), "MyClassName");
}

// ── to_camel_case ──────────────────────────────────────────────────────────

#[tokio::test]
async fn test_to_camel_case_from_pascal() {
    let engine = TemplateEngine::new();
    let data = json!({"name": "MyClassName"});
    let result = engine
        .render_from_string("{{to_camel_case name}}", &data, "camel_pascal".into())
        .await
        .unwrap();
    assert_eq!(result.trim(), "myClassName");
}

#[tokio::test]
async fn test_to_camel_case_from_snake() {
    let engine = TemplateEngine::new();
    let data = json!({"name": "my_class_name"});
    let result = engine
        .render_from_string("{{to_camel_case name}}", &data, "camel_snake".into())
        .await
        .unwrap();
    assert_eq!(result.trim(), "myClassName");
}

#[tokio::test]
async fn test_to_camel_case_single_word() {
    let engine = TemplateEngine::new();
    let data = json!({"name": "Order"});
    let result = engine
        .render_from_string("{{to_camel_case name}}", &data, "camel_single".into())
        .await
        .unwrap();
    assert_eq!(result.trim(), "order");
}

// ── Combined usage in template ─────────────────────────────────────────────

#[tokio::test]
async fn test_helpers_combined_in_template() {
    let engine = TemplateEngine::new();
    let data = json!({"entity": "OrderItem"});
    let template = r"file: {{to_kebab_case entity}}.cs
class: {{to_pascal_case entity}}
field: {{to_camel_case entity}}
table: {{to_snake_case entity}}";
    let result = engine
        .render_from_string(template, &data, "combined".into())
        .await
        .unwrap();
    let lines: Vec<&str> = result.trim().lines().collect();
    assert_eq!(lines[0], "file: order-item.cs");
    assert_eq!(lines[1], "class: OrderItem");
    assert_eq!(lines[2], "field: orderItem");
    assert_eq!(lines[3], "table: order_item");
}
