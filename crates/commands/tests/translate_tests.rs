//! Unit tests for translate command
//!
//! Tests follow the AAA (Arrange, Act, Assert) pattern per project standards.

use nettoolskit_commands::translate::{handle_translate, TranslateRequest};
use nettoolskit_commands::ExitStatus;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper to create a temporary test directory with a sample template
fn create_test_template() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let template_path = temp_dir.path().join("test-template.cs.hbs");

    fs::write(
        &template_path,
        "namespace {{namespace}} {\n    public class {{className}} {}\n}",
    )
    .expect("Failed to write template");

    (temp_dir, template_path)
}

#[tokio::test]
async fn test_translate_valid_languages() {
    // Arrange
    let (_temp_dir, template_path) = create_test_template();
    let request = TranslateRequest {
        from: "csharp".to_string(),
        to: "java".to_string(),
        path: template_path.to_string_lossy().to_string(),
    };

    // Act
    let result = handle_translate(request).await;

    // Assert
    assert_eq!(result, ExitStatus::Success);
}

#[tokio::test]
async fn test_translate_invalid_source_language() {
    // Arrange
    let (_temp_dir, template_path) = create_test_template();
    let request = TranslateRequest {
        from: "invalid_lang".to_string(),
        to: "java".to_string(),
        path: template_path.to_string_lossy().to_string(),
    };

    // Act
    let result = handle_translate(request).await;

    // Assert
    assert_eq!(result, ExitStatus::Error);
}

#[tokio::test]
async fn test_translate_invalid_target_language() {
    // Arrange
    let (_temp_dir, template_path) = create_test_template();
    let request = TranslateRequest {
        from: "dotnet".to_string(),
        to: "invalid_lang".to_string(),
        path: template_path.to_string_lossy().to_string(),
    };

    // Act
    let result = handle_translate(request).await;

    // Assert
    assert_eq!(result, ExitStatus::Error);
}

#[tokio::test]
async fn test_translate_same_source_and_target() {
    // Arrange
    let (_temp_dir, template_path) = create_test_template();
    let request = TranslateRequest {
        from: "csharp".to_string(),
        to: "csharp".to_string(),
        path: template_path.to_string_lossy().to_string(),
    };

    // Act
    let result = handle_translate(request).await;

    // Assert
    assert_eq!(result, ExitStatus::Error);
}

#[tokio::test]
async fn test_translate_missing_template_file() {
    // Arrange
    let request = TranslateRequest {
        from: "csharp".to_string(),
        to: "java".to_string(),
        path: "/nonexistent/path/template.hbs".to_string(),
    };

    // Act
    let result = handle_translate(request).await;

    // Assert
    assert_eq!(result, ExitStatus::Error);
}

#[tokio::test]
async fn test_translate_language_aliases() {
    // Arrange - Test C# alias
    let (_temp_dir, template_path) = create_test_template();
    let request = TranslateRequest {
        from: "c#".to_string(),
        to: "python".to_string(),
        path: template_path.to_string_lossy().to_string(),
    };

    // Act
    let result = handle_translate(request).await;

    // Assert
    assert_eq!(result, ExitStatus::Success);
}

#[tokio::test]
async fn test_translate_dotnet_to_java() {
    // Arrange
    let (_temp_dir, template_path) = create_test_template();
    let request = TranslateRequest {
        from: "dotnet".to_string(),
        to: "java".to_string(),
        path: template_path.to_string_lossy().to_string(),
    };

    // Act
    let result = handle_translate(request).await;

    // Assert
    assert_eq!(result, ExitStatus::Success);
}

#[tokio::test]
async fn test_translate_python_to_rust() {
    // Arrange
    let (_temp_dir, template_path) = create_test_template();
    let request = TranslateRequest {
        from: "python".to_string(),
        to: "rust".to_string(),
        path: template_path.to_string_lossy().to_string(),
    };

    // Act
    let result = handle_translate(request).await;

    // Assert
    assert_eq!(result, ExitStatus::Success);
}

#[tokio::test]
async fn test_translate_go_to_clojure() {
    // Arrange
    let (_temp_dir, template_path) = create_test_template();
    let request = TranslateRequest {
        from: "go".to_string(),
        to: "clojure".to_string(),
        path: template_path.to_string_lossy().to_string(),
    };

    // Act
    let result = handle_translate(request).await;

    // Assert
    assert_eq!(result, ExitStatus::Success);
}

#[tokio::test]
async fn test_translate_rust_to_golang() {
    // Arrange - Test golang alias
    let (_temp_dir, template_path) = create_test_template();
    let request = TranslateRequest {
        from: "rust".to_string(),
        to: "golang".to_string(),
        path: template_path.to_string_lossy().to_string(),
    };

    // Act
    let result = handle_translate(request).await;

    // Assert
    assert_eq!(result, ExitStatus::Success);
}

#[tokio::test]
async fn test_translate_to_dotnet_creates_output_file() {
    // Arrange
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let template_path = temp_dir.path().join("Entity.cs.hbs");

    fs::write(
        &template_path,
        "namespace {{namespace}} {\n    public class {{class_name}} {}\n}",
    )
    .expect("Failed to write template");

    let request = TranslateRequest {
        from: "java".to_string(),
        to: "dotnet".to_string(),
        path: template_path.to_string_lossy().to_string(),
    };

    // Act
    let result = handle_translate(request).await;

    // Assert
    assert_eq!(result, ExitStatus::Success);

    // Verify output file was created
    let output_path = temp_dir.path().join("Entity.cs");
    assert!(output_path.exists(), "Output file should be created");

    // Verify content was converted
    let content = fs::read_to_string(&output_path).expect("Failed to read output");
    assert!(
        content.contains("{{Namespace}}"),
        "Should convert to PascalCase"
    );
    assert!(
        content.contains("{{ClassName}}"),
        "Should convert class_name to ClassName"
    );
}

#[tokio::test]
async fn test_translate_to_dotnet_adds_xml_docs() {
    // Arrange
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let template_path = temp_dir.path().join("Service.cs.hbs");

    fs::write(&template_path, "public class {{service_name}} { }")
        .expect("Failed to write template");

    let request = TranslateRequest {
        from: "python".to_string(),
        to: "csharp".to_string(),
        path: template_path.to_string_lossy().to_string(),
    };

    // Act
    let result = handle_translate(request).await;

    // Assert
    assert_eq!(result, ExitStatus::Success);

    // Verify XML docs were added
    let output_path = temp_dir.path().join("Service.cs");
    let content = fs::read_to_string(&output_path).expect("Failed to read output");
    assert!(
        content.contains("/// <summary>"),
        "Should add XML documentation"
    );
}

#[tokio::test]
async fn test_translate_placeholder_extraction() {
    // Arrange
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let template_path = temp_dir.path().join("Complex.cs.hbs");

    fs::write(
        &template_path,
        "{{namespace}}.{{class_name}} : {{interface_name}} { {{property_name}} }",
    )
    .expect("Failed to write template");

    let request = TranslateRequest {
        from: "java".to_string(),
        to: "dotnet".to_string(),
        path: template_path.to_string_lossy().to_string(),
    };

    // Act
    let result = handle_translate(request).await;

    // Assert
    assert_eq!(result, ExitStatus::Success);

    // Verify all placeholders were detected and converted
    let output_path = temp_dir.path().join("Complex.cs");
    let content = fs::read_to_string(&output_path).expect("Failed to read output");
    assert!(content.contains("{{Namespace}}"));
    assert!(content.contains("{{ClassName}}"));
    assert!(content.contains("{{InterfaceName}}"));
    assert!(content.contains("{{PropertyName}}"));
}
