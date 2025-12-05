use nettoolskit_templating::TemplateEngine;
use serde_json::json;

#[tokio::test]
async fn test_render_with_invalid_syntax() {
    // Arrange
    let engine = TemplateEngine::new();
    let data = json!({"name": "Test"});

    // Act
    let result = engine
        .render_from_string(
            "{{name", // Missing closing braces
            &data,
            "invalid_syntax".to_string(),
        )
        .await;

    // Assert
    assert!(result.is_err());
}

#[tokio::test]
async fn test_render_with_unicode_content() {
    // Arrange
    let engine = TemplateEngine::new();
    let data = json!({"emoji": "🚀", "chinese": "你好"});

    // Act
    let result = engine
        .render_from_string("{{emoji}} {{chinese}}", &data, "unicode".to_string())
        .await;

    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "🚀 你好\n");
}

#[tokio::test]
async fn test_render_with_special_characters() {
    // Arrange
    let engine = TemplateEngine::new();
    let data = json!({"text": "<script>alert('xss')</script>"});

    // Act
    let result = engine
        .render_from_string("{{text}}", &data, "special_chars".to_string())
        .await;

    // Assert
    assert!(result.is_ok());
    let rendered = result.unwrap();
    // Critical: Handlebars escapes HTML by default for security
    assert!(rendered.contains("&lt;script&gt;"));
}

#[tokio::test]
async fn test_render_with_large_template() {
    let engine = TemplateEngine::new();
    let data = json!({"value": "test"});

    // Create a large template (1000 lines)
    let template = (0..1000)
        .map(|i| format!("Line {}: {{{{value}}}}", i))
        .collect::<Vec<_>>()
        .join("\n");

    let result = engine
        .render_from_string(&template, &data, "large".to_string())
        .await;

    assert!(result.is_ok());
    let rendered = result.unwrap();
    assert!(rendered.contains("Line 0: test"));
    assert!(rendered.contains("Line 999: test"));
}