use crate::rendering::common;
use nettoolskit_templating::{BatchRenderer, RenderRequest};
use serde_json::json;
use std::fs;

#[tokio::test]
async fn test_batch_render_empty_requests() {
    // Arrange
    let temp = common::create_batch_test_templates();
    let renderer = BatchRenderer::new(temp.path().join("templates"));
    let requests: Vec<RenderRequest<serde_json::Value>> = vec![];

    // Act
    let result = renderer.render_batch(requests).await.unwrap();

    // Assert
    assert_eq!(result.succeeded, 0);
    assert_eq!(result.failed, 0);
    assert!(result.errors.is_empty());
}

#[tokio::test]
async fn test_batch_render_creates_output_directories() {
    // Arrange
    let temp = common::create_batch_test_templates();
    let output_dir = temp.path().join("output").join("nested").join("deep");
    let renderer = BatchRenderer::new(temp.path().join("templates"));
    let requests = vec![RenderRequest {
        template: "dotnet/Domain/Entity.cs.hbs".to_string(),
        data: json!({"name": "User"}),
        output: output_dir.join("User.cs"),
    }];

    // Act
    let result = renderer.render_batch(requests).await.unwrap();

    // Assert
    assert_eq!(result.succeeded, 1);
    assert!(output_dir.exists());
    assert!(output_dir.join("User.cs").exists());
}

#[tokio::test]
async fn test_batch_render_overwrites_existing_files() {
    // Arrange
    let temp = common::create_batch_test_templates();
    let output_dir = temp.path().join("output");
    fs::create_dir_all(&output_dir).unwrap();
    let output_path = output_dir.join("User.cs");
    fs::write(&output_path, "old content").unwrap();
    let renderer = BatchRenderer::new(temp.path().join("templates"));
    let requests = vec![RenderRequest {
        template: "dotnet/Domain/Entity.cs.hbs".to_string(),
        data: json!({"name": "User"}),
        output: output_path.clone(),
    }];

    // Act
    let result = renderer.render_batch(requests).await.unwrap();

    // Assert
    assert_eq!(result.succeeded, 1);
    let content = fs::read_to_string(&output_path).unwrap();
    assert!(content.contains("public class User"));
    assert!(!content.contains("old content"));
}

#[tokio::test]
async fn test_batch_render_with_unicode_data() {
    // Arrange
    let temp = common::create_batch_test_templates();
    let output_dir = temp.path().join("output");
    let renderer = BatchRenderer::new(temp.path().join("templates"));
    let requests = vec![RenderRequest {
        template: "dotnet/Domain/Entity.cs.hbs".to_string(),
        data: json!({"name": "用户"}),
        output: output_dir.join("User.cs"),
    }];

    // Act
    let result = renderer.render_batch(requests).await.unwrap();

    // Assert
    assert_eq!(result.succeeded, 1);
    let content = fs::read_to_string(output_dir.join("User.cs")).unwrap();
    assert!(content.contains("用户"));
}