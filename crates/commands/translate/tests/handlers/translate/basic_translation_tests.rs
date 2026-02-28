use crate::handlers::test_helpers::create_test_template;
use nettoolskit_core::ExitStatus;
use nettoolskit_translate::{handle_translate, TranslateRequest};

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

    // Assert — Java translation is now implemented
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

    // Assert — Java translation is implemented
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

    // Assert — Rust translation is implemented
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

    // Assert — Clojure translation is NOT implemented (returns Error)
    assert_eq!(result, ExitStatus::Error);
}

#[tokio::test]
async fn test_translate_rust_to_golang() {
    // Arrange — Test golang alias
    let (_temp_dir, template_path) = create_test_template();
    let request = TranslateRequest {
        from: "rust".to_string(),
        to: "golang".to_string(),
        path: template_path.to_string_lossy().to_string(),
    };

    // Act
    let result = handle_translate(request).await;

    // Assert — Go translation is implemented
    assert_eq!(result, ExitStatus::Success);
}

#[tokio::test]
async fn test_translate_language_aliases() {
    // Arrange — Test C# alias to python
    let (_temp_dir, template_path) = create_test_template();
    let request = TranslateRequest {
        from: "c#".to_string(),
        to: "python".to_string(),
        path: template_path.to_string_lossy().to_string(),
    };

    // Act
    let result = handle_translate(request).await;

    // Assert — Python translation is implemented
    assert_eq!(result, ExitStatus::Success);
}

#[tokio::test]
async fn test_translate_to_typescript() {
    // Arrange
    let (_temp_dir, template_path) = create_test_template();
    let request = TranslateRequest {
        from: "dotnet".to_string(),
        to: "typescript".to_string(),
        path: template_path.to_string_lossy().to_string(),
    };

    // Act
    let result = handle_translate(request).await;

    // Assert — TypeScript translation is NOT implemented (returns Error)
    assert_eq!(result, ExitStatus::Error);
}

#[tokio::test]
async fn test_translate_ts_alias() {
    // Arrange — Test ts alias
    let (_temp_dir, template_path) = create_test_template();
    let request = TranslateRequest {
        from: "dotnet".to_string(),
        to: "ts".to_string(),
        path: template_path.to_string_lossy().to_string(),
    };

    // Act
    let result = handle_translate(request).await;

    // Assert — TypeScript translation is NOT implemented (returns Error)
    assert_eq!(result, ExitStatus::Error);
}
