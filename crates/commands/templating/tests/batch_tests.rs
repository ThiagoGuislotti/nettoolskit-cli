mod common;

use nettoolskit_templating::{BatchRenderer, RenderRequest};
use serde_json::json;
use std::fs;

#[tokio::test]
async fn test_batch_render_success() {
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

    let result = renderer.render_batch(requests).await.unwrap();

    assert_eq!(result.succeeded, 2);
    assert_eq!(result.failed, 0);
    assert!(result.errors.is_empty());

    // Verify output files
    let user_content = fs::read_to_string(output_dir.join("User.cs")).unwrap();
    assert!(user_content.contains("public class User"));

    let product_content = fs::read_to_string(output_dir.join("Product.cs")).unwrap();
    assert!(product_content.contains("public class Product"));
}

#[tokio::test]
async fn test_batch_render_with_errors() {
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

    let result = renderer.render_batch(requests).await.unwrap();

    assert_eq!(result.succeeded, 1);
    assert_eq!(result.failed, 1);
    assert_eq!(result.errors.len(), 1);
}

#[tokio::test]
async fn test_batch_render_parallelism() {
    let temp = common::create_batch_test_templates();
    let output_dir = temp.path().join("output");

    let renderer = BatchRenderer::new(temp.path().join("templates"))
        .with_max_concurrency(4);

    // Create 20 requests
    let requests: Vec<_> = (0..20)
        .map(|i| RenderRequest {
            template: "dotnet/Domain/Entity.cs.hbs".to_string(),
            data: json!({"name": format!("Entity{}", i)}),
            output: output_dir.join(format!("Entity{}.cs", i)),
        })
        .collect();

    let result = renderer.render_batch(requests).await.unwrap();

    assert_eq!(result.succeeded, 20);
    assert_eq!(result.failed, 0);

    // Verify all files created
    for i in 0..20 {
        let content = fs::read_to_string(output_dir.join(format!("Entity{}.cs", i))).unwrap();
        assert!(content.contains(&format!("public class Entity{}", i)));
    }
}