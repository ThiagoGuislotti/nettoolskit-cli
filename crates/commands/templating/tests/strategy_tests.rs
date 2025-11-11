use nettoolskit_templating::{
    DotNetStrategy, JavaStrategy, GoStrategy, PythonStrategy,
    RustStrategy, ClojureStrategy, LanguageStrategy,
};

#[test]
fn test_dotnet_strategy() {
    let strategy = DotNetStrategy::default();

    // Test normalization: ["dotnet", "Domain", "Entity.cs"] -> "dotnet/src/Domain/Entity.cs"
    let normalized = strategy.normalize_path(&["dotnet", "Domain", "Entity.cs"]);
    assert_eq!(normalized, Some("dotnet/src/Domain/Entity.cs".to_string()));

    // Test already normalized path (returns None when already normalized)
    let already_normalized = strategy.normalize_path(&["dotnet", "src", "Domain", "Entity.cs"]);
    assert_eq!(already_normalized, None);

    // Test file extension
    assert_eq!(strategy.file_extension(), "cs");
}

#[test]
fn test_java_strategy() {
    let strategy = JavaStrategy::default();

    // Test normalization: ["java", "domain", "Entity.java"] -> "java/src/main/java/domain/Entity.java"
    let normalized = strategy.normalize_path(&["java", "domain", "Entity.java"]);
    assert_eq!(normalized, Some("java/src/main/java/domain/Entity.java".to_string()));

    // Test already normalized path (returns None)
    let already_normalized = strategy.normalize_path(&["java", "src", "main", "java", "domain", "Entity.java"]);
    assert_eq!(already_normalized, None);

    // Test file extension
    assert_eq!(strategy.file_extension(), "java");
}

#[test]
fn test_go_strategy() {
    let strategy = GoStrategy::default();

    // Test normalization: ["go", "domain", "entity.go"] -> "go/pkg/domain/entity.go"
    let normalized = strategy.normalize_path(&["go", "domain", "entity.go"]);
    assert_eq!(normalized, Some("go/pkg/domain/entity.go".to_string()));

    // Test already normalized path (returns None)
    let already_normalized = strategy.normalize_path(&["go", "pkg", "domain", "entity.go"]);
    assert_eq!(already_normalized, None);

    // Test file extension
    assert_eq!(strategy.file_extension(), "go");
}

#[test]
fn test_python_strategy() {
    let strategy = PythonStrategy::default();

    // Test normalization: ["python", "domain", "entity.py"] -> "python/src/domain/entity.py"
    let normalized = strategy.normalize_path(&["python", "domain", "entity.py"]);
    assert_eq!(normalized, Some("python/src/domain/entity.py".to_string()));

    // Test already normalized path (returns None)
    let already_normalized = strategy.normalize_path(&["python", "src", "domain", "entity.py"]);
    assert_eq!(already_normalized, None);

    // Test file extension
    assert_eq!(strategy.file_extension(), "py");
}

#[test]
fn test_rust_strategy() {
    let strategy = RustStrategy::default();

    // Test normalization: ["rust", "domain", "entity.rs"] -> "rust/src/domain/entity.rs"
    let normalized = strategy.normalize_path(&["rust", "domain", "entity.rs"]);
    assert_eq!(normalized, Some("rust/src/domain/entity.rs".to_string()));

    // Test already normalized path (returns None)
    let already_normalized = strategy.normalize_path(&["rust", "src", "domain", "entity.rs"]);
    assert_eq!(already_normalized, None);

    // Test file extension
    assert_eq!(strategy.file_extension(), "rs");
}

#[test]
fn test_clojure_strategy() {
    let strategy = ClojureStrategy::default();

    // Test normalization: ["clojure", "domain", "entity.clj"] -> "clojure/src/domain/entity.clj"
    let normalized = strategy.normalize_path(&["clojure", "domain", "entity.clj"]);
    assert_eq!(normalized, Some("clojure/src/domain/entity.clj".to_string()));

    // Test already normalized path (returns None)
    let already_normalized = strategy.normalize_path(&["clojure", "src", "domain", "entity.clj"]);
    assert_eq!(already_normalized, None);

    // Test file extension
    assert_eq!(strategy.file_extension(), "clj");
}