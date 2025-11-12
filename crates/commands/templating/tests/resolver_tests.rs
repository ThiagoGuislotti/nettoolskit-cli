mod common;

use nettoolskit_templating::TemplateResolver;
use std::{fs, sync::Arc};
use tempfile::TempDir;

// Template Resolution Tests

#[tokio::test]
async fn test_resolve_direct_path() {
    // Arrange
    let temp = common::create_dotnet_test_structure();
    let resolver = TemplateResolver::new(temp.path().join("templates/dotnet"));

    // Act
    let result = resolver.resolve("src/Domain/Entity.cs.hbs").await;

    // Assert
    assert!(result.is_ok());
    assert!(result.unwrap().exists());
}

#[tokio::test]
async fn test_resolve_normalized_dotnet_path() {
    // Arrange
    let temp = common::create_dotnet_test_structure();
    let resolver = TemplateResolver::new(temp.path().join("templates"));

    // Act
    let result = resolver.resolve("dotnet/Domain/Entity.cs.hbs").await;

    // Assert
    // Critical: should find "dotnet/src/Domain/Entity.cs.hbs" via normalization
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_resolve_by_filename_search() {
    // Arrange
    let temp = common::create_dotnet_test_structure();
    let resolver = TemplateResolver::new(temp.path().join("templates/dotnet"));

    // Act
    let result = resolver.resolve("Entity.cs.hbs").await;

    // Assert
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_resolve_not_found() {
    use nettoolskit_templating::TemplateError;

    // Arrange
    let temp = common::create_dotnet_test_structure();
    let resolver = TemplateResolver::new(temp.path().join("templates/dotnet"));

    // Act
    let result = resolver.resolve("NonExistent.hbs").await;

    // Assert
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
    // Arrange
    let temp = TempDir::new().unwrap();
    let templates = temp.path().join("templates");

    let java_path = templates.join("java/src/main/java/domain");
    fs::create_dir_all(&java_path).unwrap();
    fs::write(java_path.join("Entity.java.hbs"), "java content").unwrap();

    let python_path = templates.join("python/src/domain");
    fs::create_dir_all(&python_path).unwrap();
    fs::write(python_path.join("entity.py.hbs"), "python content").unwrap();

    let go_path = templates.join("go/pkg/domain");
    fs::create_dir_all(&go_path).unwrap();
    fs::write(go_path.join("entity.go.hbs"), "go content").unwrap();

    let rust_path = templates.join("rust/src/domain");
    fs::create_dir_all(&rust_path).unwrap();
    fs::write(rust_path.join("entity.rs.hbs"), "rust content").unwrap();

    let clojure_path = templates.join("clojure/src/domain");
    fs::create_dir_all(&clojure_path).unwrap();
    fs::write(clojure_path.join("entity.clj.hbs"), "clojure content").unwrap();

    let resolver = TemplateResolver::new(&templates);

    // Act & Assert - Critical: test all language-specific path normalizations
    let java_result = resolver.resolve("java/domain/Entity.java.hbs").await;
    assert!(java_result.is_ok(), "Java normalization should work");

    let python_result = resolver.resolve("python/domain/entity.py.hbs").await;
    assert!(python_result.is_ok(), "Python normalization should work");

    let go_result = resolver.resolve("go/domain/entity.go.hbs").await;
    assert!(go_result.is_ok(), "Go normalization should work");

    let rust_result = resolver.resolve("rust/domain/entity.rs.hbs").await;
    assert!(rust_result.is_ok(), "Rust normalization should work");

    let clojure_result = resolver.resolve("clojure/domain/entity.clj.hbs").await;
    assert!(clojure_result.is_ok(), "Clojure normalization should work");
}

#[tokio::test]
async fn test_cache_performance() {
    // Arrange
    let temp = common::create_dotnet_test_structure();
    let resolver = TemplateResolver::new(temp.path().join("templates/dotnet"));

    // Act - First call: cache miss
    let start = std::time::Instant::now();
    let result1 = resolver.resolve("src/Domain/Entity.cs.hbs").await;
    let duration1 = start.elapsed();

    // Act - Second call: cache hit
    let start = std::time::Instant::now();
    let result2 = resolver.resolve("src/Domain/Entity.cs.hbs").await;
    let duration2 = start.elapsed();

    // Assert
    assert!(result1.is_ok());
    assert!(result2.is_ok());
    // Critical: Cache hit should be at least 10x faster
    assert!(
        duration2 < duration1 / 10,
        "Cache hit ({:?}) should be much faster than miss ({:?})",
        duration2,
        duration1
    );
    let (cache_size, _) = resolver.cache_stats();
    assert_eq!(cache_size, 1, "Cache should contain 1 entry");
}

#[tokio::test]
async fn test_parallel_resolve() {
    // Arrange
    let temp = common::create_dotnet_test_structure();
    let resolver = Arc::new(TemplateResolver::new(temp.path().join("templates/dotnet")));
    let mut handles = vec![];

    // Act - Spawn concurrent operations
    for _ in 0..10 {
        let resolver_clone = Arc::clone(&resolver);
        let handle =
            tokio::spawn(async move { resolver_clone.resolve("src/Domain/Entity.cs.hbs").await });
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
