use nettoolskit_core::ExitStatus;
use nettoolskit_translate::{handle_translate, TranslateRequest};
use std::fs;
use tempfile::TempDir;

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