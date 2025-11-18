//! Manifest Error Tests
//!
//! Tests for ManifestError enum validating Display implementation,
//! error conversions, and error propagation patterns.

use nettoolskit_manifest::{ManifestError, ManifestResult};
use std::path::PathBuf;

// Error Display Tests

#[test]
fn test_manifest_not_found_error() {
    // Act
    let error = ManifestError::ManifestNotFound {
        path: "missing.yml".to_string(),
    };

    // Assert
    assert!(error.to_string().contains("manifest not found"));
    assert!(error.to_string().contains("missing.yml"));
}

#[test]
fn test_read_error_display() {
    // Arrange
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");

    // Act
    let error = ManifestError::ReadError {
        path: "test.yml".to_string(),
        source: io_error,
    };
    let msg = error.to_string();

    // Assert
    assert!(msg.contains("failed to read manifest"));
    assert!(msg.contains("test.yml"));
}

#[test]
fn test_validation_error_display() {
    // Act
    let error = ManifestError::ValidationError("missing required field".to_string());

    // Assert
    assert!(error.to_string().contains("manifest validation failed"));
    assert!(error.to_string().contains("missing required field"));
}

#[test]
fn test_template_not_found_error() {
    // Act
    let error = ManifestError::TemplateNotFound {
        path: "templates/missing.hbs".to_string(),
    };

    // Assert
    assert!(error.to_string().contains("template not found"));
    assert!(error.to_string().contains("templates/missing.hbs"));
}

#[test]
fn test_render_error_display() {
    // Act
    let error = ManifestError::RenderError("invalid syntax".to_string());

    // Assert
    assert!(error.to_string().contains("template rendering failed"));
    assert!(error.to_string().contains("invalid syntax"));
}

#[test]
fn test_template_render_error_with_details() {
    // Act
    let error = ManifestError::TemplateRenderError {
        template: "entity.hbs".to_string(),
        reason: "missing variable 'name'".to_string(),
    };
    let msg = error.to_string();

    // Assert
    assert!(msg.contains("failed to render template"));
    assert!(msg.contains("entity.hbs"));
    assert!(msg.contains("missing variable"));
}

#[test]
fn test_solution_not_found_error() {
    // Act
    let error = ManifestError::SolutionNotFound {
        path: PathBuf::from("/path/to/solution"),
    };

    // Assert
    assert!(error.to_string().contains("solution root not found"));
    assert!(error.to_string().contains("/path/to/solution"));
}

#[test]
fn test_project_not_found_error() {
    // Act
    let error = ManifestError::ProjectNotFound {
        path: PathBuf::from("src/MyProject"),
    };

    // Assert
    assert!(error.to_string().contains("project not found"));
    assert!(error.to_string().contains("MyProject"));
}

#[test]
fn test_collision_detected_error() {
    // Act
    let error = ManifestError::CollisionDetected {
        path: PathBuf::from("src/Entity.cs"),
        policy: "fail".to_string(),
    };
    let msg = error.to_string();

    // Assert
    assert!(msg.contains("file collision detected"));
    assert!(msg.contains("Entity.cs"));
    assert!(msg.contains("policy: fail"));
}

#[test]
fn test_missing_field_error() {
    // Act
    let error = ManifestError::MissingField {
        field: "namespaceRoot".to_string(),
    };

    // Assert
    assert!(error.to_string().contains("missing required field"));
    assert!(error.to_string().contains("namespaceRoot"));
}

#[test]
fn test_invalid_configuration_error() {
    // Act
    let error = ManifestError::InvalidConfiguration("dry_run requires manifest_path".to_string());

    // Assert
    assert!(error.to_string().contains("invalid configuration"));
    assert!(error.to_string().contains("dry_run"));
}

// Error Conversion Tests

#[test]
fn test_error_from_string() {
    // Act
    let error: ManifestError = "custom error message".to_string().into();

    // Assert
    assert!(matches!(error, ManifestError::Other(_)));
    assert!(error.to_string().contains("custom error message"));
}

#[test]
fn test_error_from_str() {
    // Act
    let error: ManifestError = "another error".into();

    // Assert
    assert!(matches!(error, ManifestError::Other(_)));
    assert_eq!(error.to_string(), "another error");
}

#[test]
fn test_error_from_io_error() {
    // Arrange
    let io_error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");

    // Act
    let error: ManifestError = io_error.into();

    // Assert
    assert!(matches!(error, ManifestError::FsError(_)));
}

// Result Type Tests

#[test]
fn test_result_type_alias() {
    // Arrange
    fn test_function() -> ManifestResult<String> {
        Ok("success".to_string())
    }

    // Act
    let result = test_function();

    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "success");
}

// Error Propagation Tests

#[test]
fn test_error_propagation_with_question_mark() {
    // Arrange
    fn inner() -> ManifestResult<()> {
        Err(ManifestError::ValidationError("test".to_string()))
    }
    fn outer() -> ManifestResult<String> {
        inner()?;
        Ok("never reached".to_string())
    }

    // Act
    let result = outer();

    // Assert
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("validation failed"));
}

#[test]
fn test_error_debug_format() {
    // Arrange
    let error = ManifestError::Other("debug test".to_string());

    // Act
    let debug_str = format!("{:?}", error);

    // Assert
    assert!(debug_str.contains("Other"));
    assert!(debug_str.contains("debug test"));
}
