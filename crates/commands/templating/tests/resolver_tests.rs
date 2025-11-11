mod common;

use nettoolskit_templating::TemplateResolver;
use std::{fs, sync::Arc};
use tempfile::TempDir;

#[tokio::test]
async fn test_resolve_direct_path() {
    let temp = common::create_dotnet_test_structure();
    let resolver = TemplateResolver::new(temp.path().join("templates/dotnet"));

    let result = resolver.resolve("src/Domain/Entity.cs.hbs").await;
    assert!(result.is_ok());
    assert!(result.unwrap().exists());
}

#[tokio::test]
async fn test_resolve_normalized_dotnet_path() {
    let temp = common::create_dotnet_test_structure();
    let resolver = TemplateResolver::new(temp.path().join("templates"));

    // Should find "dotnet/src/Domain/Entity.cs.hbs" when given "dotnet/Domain/Entity.cs.hbs"
    let result = resolver.resolve("dotnet/Domain/Entity.cs.hbs").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_resolve_by_filename_search() {
    let temp = common::create_dotnet_test_structure();
    let resolver = TemplateResolver::new(temp.path().join("templates/dotnet"));

    // Should find file even with just filename
    let result = resolver.resolve("Entity.cs.hbs").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_resolve_not_found() {
    use nettoolskit_templating::TemplateError;

    let temp = common::create_dotnet_test_structure();
    let resolver = TemplateResolver::new(temp.path().join("templates/dotnet"));

    let result = resolver.resolve("NonExistent.hbs").await;
    assert!(result.is_err());

    match result {
        Err(TemplateError::NotFound { template }) => {
            assert_eq!(template, "NonExistent.hbs");
        }
        _ => panic!("Expected NotFound error"),
    }
}

#[tokio::test]
async fn test_multi_language_normalization() {
    let temp = TempDir::new().unwrap();
    let templates = temp.path().join("templates");

    // Create Java structure
    let java_path = templates.join("java/src/main/java/domain");
    fs::create_dir_all(&java_path).unwrap();
    fs::write(java_path.join("Entity.java.hbs"), "java content").unwrap();

    // Create Python structure
    let python_path = templates.join("python/src/domain");
    fs::create_dir_all(&python_path).unwrap();
    fs::write(python_path.join("entity.py.hbs"), "python content").unwrap();

    // Create Go structure
    let go_path = templates.join("go/pkg/domain");
    fs::create_dir_all(&go_path).unwrap();
    fs::write(go_path.join("entity.go.hbs"), "go content").unwrap();

    // Create Rust structure
    let rust_path = templates.join("rust/src/domain");
    fs::create_dir_all(&rust_path).unwrap();
    fs::write(rust_path.join("entity.rs.hbs"), "rust content").unwrap();

    // Create Clojure structure
    let clojure_path = templates.join("clojure/src/domain");
    fs::create_dir_all(&clojure_path).unwrap();
    fs::write(clojure_path.join("entity.clj.hbs"), "clojure content").unwrap();

    let resolver = TemplateResolver::new(&templates);

    // Java: should normalize "java/domain/..." to "java/src/main/java/domain/..."
    let java_result = resolver.resolve("java/domain/Entity.java.hbs").await;
    assert!(java_result.is_ok(), "Java normalization should work");

    // Python: should normalize "python/domain/..." to "python/src/domain/..."
    let python_result = resolver.resolve("python/domain/entity.py.hbs").await;
    assert!(python_result.is_ok(), "Python normalization should work");

    // Go: should normalize "go/domain/..." to "go/pkg/domain/..."
    let go_result = resolver.resolve("go/domain/entity.go.hbs").await;
    assert!(go_result.is_ok(), "Go normalization should work");

    // Rust: should normalize "rust/domain/..." to "rust/src/domain/..."
    let rust_result = resolver.resolve("rust/domain/entity.rs.hbs").await;
    assert!(rust_result.is_ok(), "Rust normalization should work");

    // Clojure: should normalize "clojure/domain/..." to "clojure/src/domain/..."
    let clojure_result = resolver.resolve("clojure/domain/entity.clj.hbs").await;
    assert!(clojure_result.is_ok(), "Clojure normalization should work");
}

#[tokio::test]
async fn test_cache_performance() {
    let temp = common::create_dotnet_test_structure();
    let resolver = TemplateResolver::new(temp.path().join("templates/dotnet"));

    // First call: cache miss
    let start = std::time::Instant::now();
    let result1 = resolver.resolve("src/Domain/Entity.cs.hbs").await;
    let duration1 = start.elapsed();
    assert!(result1.is_ok());

    // Second call: cache hit (should be much faster)
    let start = std::time::Instant::now();
    let result2 = resolver.resolve("src/Domain/Entity.cs.hbs").await;
    let duration2 = start.elapsed();
    assert!(result2.is_ok());

    // Cache hit should be at least 10x faster (typically 100-1000x)
    assert!(duration2 < duration1 / 10,
        "Cache hit ({:?}) should be much faster than miss ({:?})",
        duration2, duration1);

    // Verify cache stats
    let (cache_size, _) = resolver.cache_stats();
    assert_eq!(cache_size, 1, "Cache should contain 1 entry");
}

#[tokio::test]
async fn test_parallel_resolve() {
    let temp = common::create_dotnet_test_structure();
    let resolver = Arc::new(TemplateResolver::new(temp.path().join("templates/dotnet")));

    // Spawn multiple concurrent resolve operations
    let mut handles = vec![];

    for _ in 0..10 {
        let resolver_clone = Arc::clone(&resolver);
        let handle = tokio::spawn(async move {
            resolver_clone.resolve("src/Domain/Entity.cs.hbs").await
        });
        handles.push(handle);
    }

    // Wait for all to complete
    let results: Vec<_> = futures::future::join_all(handles).await;

    // All should succeed
    for result in results {
        assert!(result.is_ok());
        assert!(result.unwrap().is_ok());
    }

    // Cache should still have only 1 entry (same path resolved 10 times)
    let (cache_size, _) = resolver.cache_stats();
    assert_eq!(cache_size, 1);
}