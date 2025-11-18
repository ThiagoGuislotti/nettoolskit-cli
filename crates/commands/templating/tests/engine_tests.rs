//! Template Engine Tests
//!
//! Tests for TemplateEngine validating template rendering, variable substitution,
//! filter application, and error handling for Handlebars templates.

use nettoolskit_templating::TemplateEngine;
use serde_json::json;

// Basic Rendering Tests

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

// Caching Tests

#[tokio::test]
async fn test_template_caching() {
    // Arrange
    let engine = TemplateEngine::new();
    let data = json!({"name": "Test"});

    // Act - First render: cache miss
    let start = std::time::Instant::now();
    let result1 = engine
        .render_from_string("Hello {{name}}!", &data, "cached_test".to_string())
        .await;
    let duration1 = start.elapsed();

    // Act - Second render: cache hit
    let start = std::time::Instant::now();
    let result2 = engine
        .render_from_string("Hello {{name}}!", &data, "cached_test".to_string())
        .await;
    let duration2 = start.elapsed();

    // Assert
    assert!(result1.is_ok());
    assert!(result2.is_ok());
    // Critical: Cache hit should be at least 2x faster
    assert!(
        duration2 < duration1 / 2,
        "Cache hit ({:?}) should be faster than miss ({:?})",
        duration2,
        duration1
    );
    let (cache_size, _) = engine.cache_stats();
    assert_eq!(cache_size, 1, "Cache should contain 1 template");
}

// TODO Insertion Tests

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

// Error Handling Tests

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

// Edge Cases Tests

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

#[tokio::test]
async fn test_cache_stats_accuracy() {
    let engine = TemplateEngine::new();
    let data = json!({"name": "Test"});

    // Initial state
    let (size, hits) = engine.cache_stats();
    assert_eq!(size, 0);
    assert_eq!(hits, 0);

    // Render first template
    let _ = engine
        .render_from_string("Template 1: {{name}}", &data, "template1".to_string())
        .await;

    let (size, _) = engine.cache_stats();
    assert_eq!(size, 1);

    // Render second template
    let _ = engine
        .render_from_string("Template 2: {{name}}", &data, "template2".to_string())
        .await;

    let (size, _) = engine.cache_stats();
    assert_eq!(size, 2);
}
