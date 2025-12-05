use crate::rendering::common;
use nettoolskit_templating::{BatchRenderer, RenderRequest};
use serde_json::json;

#[tokio::test]
async fn test_batch_render_with_errors() {
    // Arrange
    let temp = common::create_batch_test_templates();
    let output_dir = temp.path().join("output");
    let renderer = BatchRenderer::new(temp.path().join("templates"));
    let requests = vec![
        RenderRequest {
            template: "dotnet/Domain/Entity.cs.hbs".to_string(),
            data: json!({"name": "User"}),
            output: output_dir.join("User.cs"),
        },
        RenderRequest {
            template: "nonexistent/Template.hbs".to_string(),
            data: json!({"name": "Invalid"}),
            output: output_dir.join("Invalid.cs"),
        },
    ];

    // Act
    let result = renderer.render_batch(requests).await.unwrap();

    // Assert
    assert_eq!(result.succeeded, 1);
    assert_eq!(result.failed, 1);
    assert_eq!(result.errors.len(), 1);
}

#[tokio::test]
async fn test_batch_render_all_fail() {
    // Arrange
    let temp = common::create_batch_test_templates();
    let output_dir = temp.path().join("output");
    let renderer = BatchRenderer::new(temp.path().join("templates"));
    let requests = vec![
        RenderRequest {
            template: "nonexistent1.hbs".to_string(),
            data: json!({}),
            output: output_dir.join("out1.txt"),
        },
        RenderRequest {
            template: "nonexistent2.hbs".to_string(),
            data: json!({}),
            output: output_dir.join("out2.txt"),
        },
    ];

    // Act
    let result = renderer.render_batch(requests).await.unwrap();

    // Assert
    assert_eq!(result.succeeded, 0);
    assert_eq!(result.failed, 2);
    assert_eq!(result.errors.len(), 2);
}