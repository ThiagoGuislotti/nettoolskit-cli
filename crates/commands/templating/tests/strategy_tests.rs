use nettoolskit_templating::{
    DotNetStrategy, JavaStrategy, GoStrategy, PythonStrategy,
    RustStrategy, ClojureStrategy, LanguageStrategy,
};

// Language Strategy Tests

#[test]
fn test_dotnet_strategy() {
    // Arrange
    let strategy = DotNetStrategy::default();

    // Act
    let normalized = strategy.normalize_path(&["dotnet", "Domain", "Entity.cs"]);
    let already_normalized = strategy.normalize_path(&["dotnet", "src", "Domain", "Entity.cs"]);

    // Assert
    assert_eq!(normalized, Some("dotnet/src/Domain/Entity.cs".to_string()));
    assert_eq!(already_normalized, None);
    assert_eq!(strategy.file_extension(), "cs");
}

#[test]
fn test_java_strategy() {
    // Arrange
    let strategy = JavaStrategy::default();

    // Act
    let normalized = strategy.normalize_path(&["java", "domain", "Entity.java"]);
    let already_normalized = strategy.normalize_path(&["java", "src", "main", "java", "domain", "Entity.java"]);

    // Assert
    assert_eq!(normalized, Some("java/src/main/java/domain/Entity.java".to_string()));
    assert_eq!(already_normalized, None);
    assert_eq!(strategy.file_extension(), "java");
}

#[test]
fn test_go_strategy() {
    // Arrange
    let strategy = GoStrategy::default();

    // Act
    let normalized = strategy.normalize_path(&["go", "domain", "entity.go"]);
    let already_normalized = strategy.normalize_path(&["go", "pkg", "domain", "entity.go"]);

    // Assert
    assert_eq!(normalized, Some("go/pkg/domain/entity.go".to_string()));
    assert_eq!(already_normalized, None);
    assert_eq!(strategy.file_extension(), "go");
}

#[test]
fn test_python_strategy() {
    // Arrange
    let strategy = PythonStrategy::default();

    // Act
    let normalized = strategy.normalize_path(&["python", "domain", "entity.py"]);
    let already_normalized = strategy.normalize_path(&["python", "src", "domain", "entity.py"]);

    // Assert
    assert_eq!(normalized, Some("python/src/domain/entity.py".to_string()));
    assert_eq!(already_normalized, None);
    assert_eq!(strategy.file_extension(), "py");
}

#[test]
fn test_rust_strategy() {
    // Arrange
    let strategy = RustStrategy::default();

    // Act
    let normalized = strategy.normalize_path(&["rust", "domain", "entity.rs"]);
    let already_normalized = strategy.normalize_path(&["rust", "src", "domain", "entity.rs"]);

    // Assert
    assert_eq!(normalized, Some("rust/src/domain/entity.rs".to_string()));
    assert_eq!(already_normalized, None);
    assert_eq!(strategy.file_extension(), "rs");
}

#[test]
fn test_clojure_strategy() {
    // Arrange
    let strategy = ClojureStrategy::default();

    // Act
    let normalized = strategy.normalize_path(&["clojure", "domain", "entity.clj"]);
    let already_normalized = strategy.normalize_path(&["clojure", "src", "domain", "entity.clj"]);

    // Assert
    assert_eq!(normalized, Some("clojure/src/domain/entity.clj".to_string()));
    assert_eq!(already_normalized, None);
    assert_eq!(strategy.file_extension(), "clj");
}