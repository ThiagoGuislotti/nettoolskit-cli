use nettoolskit_templating::TemplateEngine;
use serde_json::json;

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