use crate::rendering::common;
use nettoolskit_templating::{BatchRenderer, RenderRequest};
use serde_json::json;
use std::fs;

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