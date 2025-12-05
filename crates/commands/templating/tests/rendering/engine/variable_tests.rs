use nettoolskit_templating::TemplateEngine;
use serde_json::json;

#[tokio::test]
async fn test_render_with_missing_variable() {
    // Arrange
    let engine = TemplateEngine::new();
    let data = json!({"name": "Test"});

    // Act
    let result = engine
        .render_from_string("Hello {{missing_var}}!", &data, "missing_var".to_string())
        .await;

    // Assert
    // Critical: Handlebars renders empty string for missing variables
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Hello !\n");
}

#[tokio::test]
async fn test_render_with_nested_data() {
    // Arrange
    let engine = TemplateEngine::new();
    let data = json!({
        "user": {
            "name": "John",
            "address": {
                "city": "NYC"
            }
        }
    });

    // Act
    let result = engine
        .render_from_string(
            "{{user.name}} lives in {{user.address.city}}",
            &data,
            "nested".to_string(),
        )
        .await;

    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "John lives in NYC\n");
}

#[tokio::test]
async fn test_render_with_array_iteration() {
    // Arrange
    let engine = TemplateEngine::new();
    let data = json!({
        "items": ["apple", "banana", "cherry"]
    });

    // Act
    let result = engine
        .render_from_string(
            "{{#each items}}{{this}} {{/each}}",
            &data,
            "array".to_string(),
        )
        .await;

    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "apple banana cherry \n");
}

#[tokio::test]
async fn test_render_with_conditionals() {
    // Arrange
    let engine = TemplateEngine::new();
    let data = json!({"show": true, "hide": false});

    // Act
    let result = engine
        .render_from_string(
            "{{#if show}}visible{{/if}}{{#if hide}}hidden{{/if}}",
            &data,
            "conditional".to_string(),
        )
        .await;

    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "visible\n");
}