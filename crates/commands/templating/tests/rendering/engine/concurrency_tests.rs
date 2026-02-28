use nettoolskit_templating::TemplateEngine;
use serde_json::json;
use std::sync::Arc;

/// Stress test: multiple tasks registering and rendering different templates concurrently.
/// Validates that `Arc<RwLock<Handlebars>>` handles concurrent write (registration)
/// and read (rendering) access without data races or panics.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_concurrent_template_registration_and_rendering() {
    // Arrange
    let engine = Arc::new(TemplateEngine::new());
    let task_count = 50;

    // Act — spawn many tasks that each register and render a unique template
    let mut handles = Vec::with_capacity(task_count);
    for i in 0..task_count {
        let engine = Arc::clone(&engine);
        handles.push(tokio::spawn(async move {
            let template = format!("Hello {{{{name}}}} from template {i}!");
            let data = json!({"name": format!("User{i}")});
            let name = format!("concurrent_template_{i}");

            engine
                .render_from_string(&template, &data, name)
                .await
                .unwrap()
        }));
    }

    let results: Vec<String> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r.expect("task panicked"))
        .collect();

    // Assert — each task produced correct output
    for (i, result) in results.iter().enumerate() {
        let expected = format!("Hello User{i} from template {i}!\n");
        assert_eq!(result, &expected, "Mismatch at task {i}");
    }
}

/// Stress test: same template name registered concurrently from multiple tasks.
/// Validates RwLock serializes competing registrations without UB.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_concurrent_same_template_registration() {
    // Arrange
    let engine = Arc::new(TemplateEngine::new());
    let task_count = 20;

    // Act — all tasks try to register and render the same template name
    let mut handles = Vec::with_capacity(task_count);
    for i in 0..task_count {
        let engine = Arc::clone(&engine);
        handles.push(tokio::spawn(async move {
            let data = json!({"value": i});
            engine
                .render_from_string("Result: {{value}}", &data, "shared_template".to_string())
                .await
                .unwrap()
        }));
    }

    let results: Vec<String> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r.expect("task panicked"))
        .collect();

    // Assert — all results match the expected pattern (only first registration wins,
    // but cache check + render should always produce valid output)
    for result in &results {
        assert!(
            result.starts_with("Result: "),
            "Unexpected output: {result}"
        );
        assert!(result.ends_with('\n'), "Missing trailing newline: {result}");
    }
}

/// Validates that cache stats reflect concurrent registrations correctly.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_cache_population_under_concurrency() {
    // Arrange
    let engine = Arc::new(TemplateEngine::new());
    let unique_templates = 30;

    // Act
    let mut handles = Vec::with_capacity(unique_templates);
    for i in 0..unique_templates {
        let engine = Arc::clone(&engine);
        handles.push(tokio::spawn(async move {
            let data = json!({"n": i});
            engine
                .render_from_string(
                    &format!("T{i}: {{{{n}}}}"),
                    &data,
                    format!("cache_test_{i}"),
                )
                .await
                .unwrap()
        }));
    }

    futures::future::join_all(handles).await;

    // Assert — cache should contain all unique templates
    let (len, _capacity) = engine.cache_stats();
    assert_eq!(
        len, unique_templates,
        "Cache should hold all {unique_templates} templates"
    );
}
