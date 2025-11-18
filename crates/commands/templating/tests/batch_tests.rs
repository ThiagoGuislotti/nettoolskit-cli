//! Batch Renderer Tests
//!
//! Tests for BatchRenderer validating concurrent template rendering,
//! error handling, and batch processing workflows.

mod common;

use nettoolskit_templating::{BatchRenderer, RenderRequest};
use serde_json::json;
use std::fs;

// Batch Rendering Tests

#[tokio::test]
async fn test_batch_render_success() {
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
            template: "dotnet/Domain/Entity.cs.hbs".to_string(),
            data: json!({"name": "Product"}),
            output: output_dir.join("Product.cs"),
        },
    ];

    // Act
    let result = renderer.render_batch(requests).await.unwrap();

    // Assert
    assert_eq!(result.succeeded, 2);
    assert_eq!(result.failed, 0);
    assert!(result.errors.is_empty());
    let user_content = fs::read_to_string(output_dir.join("User.cs")).unwrap();
    assert!(user_content.contains("public class User"));
    let product_content = fs::read_to_string(output_dir.join("Product.cs")).unwrap();
    assert!(product_content.contains("public class Product"));
}

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

// Edge Cases Tests

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
async fn test_batch_render_with_multiple_same_template() {
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
            template: "dotnet/Domain/Entity.cs.hbs".to_string(),
            data: json!({"name": "Product"}),
            output: output_dir.join("Product.cs"),
        },
        RenderRequest {
            template: "dotnet/Domain/Entity.cs.hbs".to_string(),
            data: json!({"name": "Order"}),
            output: output_dir.join("Order.cs"),
        },
    ];

    // Act
    let result = renderer.render_batch(requests).await.unwrap();

    // Assert
    assert_eq!(result.succeeded, 3);
    assert_eq!(result.failed, 0);
    let user_content = fs::read_to_string(output_dir.join("User.cs")).unwrap();
    assert!(user_content.contains("public class User"));
    let product_content = fs::read_to_string(output_dir.join("Product.cs")).unwrap();
    assert!(product_content.contains("public class Product"));
    let order_content = fs::read_to_string(output_dir.join("Order.cs")).unwrap();
    assert!(order_content.contains("public class Order"));
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
