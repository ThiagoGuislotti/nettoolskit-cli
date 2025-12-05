use nettoolskit_templating::TemplateEngine;
use serde_json::json;

#[tokio::test]
async fn test_render_simple_template() {
    // Arrange
    let engine = TemplateEngine::new();
    let data = json!({"name": "World"});

    // Act
    let result = engine
        .render_from_string("Hello {{name}}!", &data, "test".to_string())
        .await;

    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Hello World!\n");
}

#[tokio::test]
async fn test_render_ensures_trailing_newline() {
    // Arrange
    let engine = TemplateEngine::new();
    let data = json!({"value": "test"});

    // Act
    let result = engine
        .render_from_string("{{value}}", &data, "test".to_string())
        .await;

    // Assert
    assert!(result.is_ok());
    assert!(result.unwrap().ends_with('\n'));
}

#[tokio::test]
async fn test_render_with_empty_template() {
    // Arrange
    let engine = TemplateEngine::new();
    let data = json!({"name": "Test"});

    // Act
    let result = engine
        .render_from_string("", &data, "empty".to_string())
        .await;

    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "\n");
}