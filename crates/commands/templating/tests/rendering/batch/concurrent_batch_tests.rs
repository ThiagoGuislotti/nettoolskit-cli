use crate::rendering::common;
use nettoolskit_templating::{BatchRenderer, RenderRequest};
use serde_json::json;
use std::fs;

#[tokio::test]
async fn test_batch_render_parallelism() {
    // Arrange
    let temp = common::create_batch_test_templates();
    let output_dir = temp.path().join("output");
    let renderer = BatchRenderer::new(temp.path().join("templates")).with_max_concurrency(4);
    let requests: Vec<_> = (0..20)
        .map(|i| RenderRequest {
            template: "dotnet/Domain/Entity.cs.hbs".to_string(),
            data: json!({"name": format!("Entity{}", i)}),
            output: output_dir.join(format!("Entity{}.cs", i)),
        })
        .collect();

    // Act
    let result = renderer.render_batch(requests).await.unwrap();

    // Assert - Critical: All 20 requests should succeed with max_concurrency=4
    assert_eq!(result.succeeded, 20);
    assert_eq!(result.failed, 0);
    for i in 0..20 {
        let content = fs::read_to_string(output_dir.join(format!("Entity{}.cs", i))).unwrap();
        assert!(content.contains(&format!("public class Entity{}", i)));
    }
}

#[tokio::test]
async fn test_batch_render_concurrent_same_template() {
    // Arrange
    let temp = common::create_batch_test_templates();
    let output_dir = temp.path().join("output");
    let renderer = BatchRenderer::new(temp.path().join("templates")).with_max_concurrency(8);
    let requests: Vec<_> = (0..50)
        .map(|i| RenderRequest {
            template: "dotnet/Domain/Entity.cs.hbs".to_string(),
            data: json!({"name": format!("Entity{}", i)}),
            output: output_dir.join(format!("Entity{}.cs", i)),
        })
        .collect();

    // Act
    let start = std::time::Instant::now();
    let result = renderer.render_batch(requests).await.unwrap();
    let duration = start.elapsed();

    // Assert - Critical: 50 requests with max_concurrency=8 should complete in <5s
    assert_eq!(result.succeeded, 50);
    assert_eq!(result.failed, 0);
    assert!(
        duration < std::time::Duration::from_secs(5),
        "Batch render took too long: {:?}",
        duration
    );
}