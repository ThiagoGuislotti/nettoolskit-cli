use crate::handlers::test_helpers::create_test_template;
use nettoolskit_core::ExitStatus;
use nettoolskit_translate::{handle_translate, TranslateRequest};

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