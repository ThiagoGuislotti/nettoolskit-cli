use nettoolskit_templating::TemplateEngine;
use serde_json::json;

#[tokio::test]
async fn test_render_with_todo_insertion() {
    // Arrange
    let engine = TemplateEngine::new().with_todo_insertion(true);
    let data = json!({"name": "Test"});

    // Act
    let result = engine
        .render_from_string("Content: {{name}}", &data, "test".to_string())
        .await;

    // Assert
    assert!(result.is_ok());
    let rendered = result.unwrap();
    assert!(rendered.contains("TODO: Review generated content"));
}

#[tokio::test]
async fn test_render_skips_todo_when_already_present() {
    // Arrange
    let engine = TemplateEngine::new().with_todo_insertion(true);
    let data = json!({"name": "Test"});

    // Act
    let result = engine
        .render_from_string(
            "// TODO: existing\nContent: {{name}}",
            &data,
            "test".to_string(),
        )
        .await;

    // Assert
    assert!(result.is_ok());
    let rendered = result.unwrap();
    assert_eq!(rendered.matches("TODO").count(), 1);
}