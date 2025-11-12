//! Tests for TemplateError types
//!
//! This test file validates all TemplateError variants, including
//! Display formatting, Debug formatting, conversions, and error propagation.

use nettoolskit_templating::{TemplateError, TemplateResult};
use std::io;

// Display Tests

#[test]
fn test_not_found_error_display() {
    // Arrange
    let error = TemplateError::NotFound {
        template: "dotnet/Entity.cs.hbs".to_string(),
    };

    // Act
    let display = format!("{}", error);

    // Assert
    assert_eq!(display, "Template not found: dotnet/Entity.cs.hbs");
}

#[test]
fn test_read_error_display() {
    // Arrange
    let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
    let error = TemplateError::ReadError {
        path: "/templates/file.hbs".to_string(),
        source: io_error,
    };

    // Act
    let display = format!("{}", error);

    // Assert
    assert!(display.contains("Failed to read template /templates/file.hbs"));
    assert!(display.contains("access denied"));
}

#[test]
fn test_registration_error_display() {
    // Arrange
    let error = TemplateError::RegistrationError {
        template: "invalid_template".to_string(),
        message: "syntax error at line 5".to_string(),
    };

    // Act
    let display = format!("{}", error);

    // Assert
    assert_eq!(
        display,
        "Failed to register template invalid_template: syntax error at line 5"
    );
}

#[test]
fn test_render_error_display() {
    // Arrange
    let error = TemplateError::RenderError {
        template: "Entity.cs.hbs".to_string(),
        message: "missing variable 'name'".to_string(),
    };

    // Act
    let display = format!("{}", error);

    // Assert
    assert_eq!(
        display,
        "Failed to render template Entity.cs.hbs: missing variable 'name'"
    );
}

// Debug Tests

#[test]
fn test_not_found_error_debug() {
    // Arrange
    let error = TemplateError::NotFound {
        template: "test.hbs".to_string(),
    };

    // Act
    let debug = format!("{:?}", error);

    // Assert
    assert!(debug.contains("NotFound"));
    assert!(debug.contains("test.hbs"));
}

#[test]
fn test_all_error_variants_have_debug() {
    // Arrange
    let errors = vec![
        TemplateError::NotFound {
            template: "test".to_string(),
        },
        TemplateError::ReadError {
            path: "path".to_string(),
            source: io::Error::new(io::ErrorKind::NotFound, "not found"),
        },
        TemplateError::RegistrationError {
            template: "test".to_string(),
            message: "error".to_string(),
        },
        TemplateError::RenderError {
            template: "test".to_string(),
            message: "error".to_string(),
        },
    ];

    // Act & Assert
    for error in errors {
        let debug = format!("{:?}", error);
        assert!(!debug.is_empty());
        assert!(debug.contains("Error") || debug.contains("NotFound"));
    }
}

// Error Source Tests

#[test]
fn test_read_error_has_source() {
    // Arrange
    let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let error = TemplateError::ReadError {
        path: "/test/file.hbs".to_string(),
        source: io_error,
    };

    // Act & Assert
    use std::error::Error;
    assert!(error.source().is_some());
}

#[test]
fn test_other_errors_no_source() {
    use std::error::Error;

    // Arrange
    let errors = vec![
        TemplateError::NotFound {
            template: "test".to_string(),
        },
        TemplateError::RegistrationError {
            template: "test".to_string(),
            message: "error".to_string(),
        },
        TemplateError::RenderError {
            template: "test".to_string(),
            message: "error".to_string(),
        },
    ];

    // Act & Assert
    for error in errors {
        assert!(error.source().is_none());
    }
}

// Result Type Tests

#[test]
fn test_template_result_ok() {
    // Arrange & Act
    let result: TemplateResult<String> = Ok("success".to_string());

    // Assert
    assert!(result.is_ok());
    assert_eq!(result.as_ref().unwrap(), "success");
}

#[test]
fn test_template_result_err() {
    // Arrange & Act
    let result: TemplateResult<String> = Err(TemplateError::NotFound {
        template: "test.hbs".to_string(),
    });

    // Assert
    assert!(result.is_err());
}

// Error Propagation Tests

#[test]
fn test_error_propagation_with_question_mark() {
    // Arrange
    fn failing_function() -> TemplateResult<()> {
        Err(TemplateError::NotFound {
            template: "missing.hbs".to_string(),
        })
    }

    fn outer_function() -> TemplateResult<String> {
        failing_function()?;
        Ok("never reached".to_string())
    }

    // Act
    let result = outer_function();

    // Assert
    assert!(result.is_err());
    match result.unwrap_err() {
        TemplateError::NotFound { template } => {
            assert_eq!(template, "missing.hbs");
        }
        _ => panic!("Wrong error type"),
    }
}

#[test]
fn test_error_chain_propagation() {
    // Arrange
    fn inner() -> TemplateResult<()> {
        Err(TemplateError::RenderError {
            template: "inner.hbs".to_string(),
            message: "inner error".to_string(),
        })
    }

    fn middle() -> TemplateResult<()> {
        inner()?;
        Ok(())
    }

    fn outer() -> TemplateResult<()> {
        middle()?;
        Ok(())
    }

    // Act
    let result = outer();

    // Assert
    assert!(result.is_err());
}

// Edge Cases Tests

#[test]
fn test_error_with_empty_template_name() {
    // Arrange
    let error = TemplateError::NotFound {
        template: String::new(),
    };

    // Act
    let display = format!("{}", error);

    // Assert
    assert_eq!(display, "Template not found: ");
}

#[test]
fn test_error_with_long_template_path() {
    // Arrange
    let long_path = "a/".repeat(100) + "template.hbs";
    let error = TemplateError::NotFound {
        template: long_path.clone(),
    };

    // Act
    let display = format!("{}", error);

    // Assert
    assert!(display.contains(&long_path));
}

#[test]
fn test_error_with_unicode_template_name() {
    // Arrange
    let error = TemplateError::NotFound {
        template: "模板/实体.hbs".to_string(),
    };

    // Act
    let display = format!("{}", error);

    // Assert
    assert!(display.contains("模板/实体.hbs"));
}

#[test]
fn test_error_with_special_characters() {
    // Arrange
    let error = TemplateError::RegistrationError {
        template: "template-with-dashes_and_underscores.hbs".to_string(),
        message: "Error: 'syntax' issue!".to_string(),
    };

    // Act
    let display = format!("{}", error);

    // Assert
    assert!(display.contains("template-with-dashes_and_underscores.hbs"));
    assert!(display.contains("Error: 'syntax' issue!"));
}

#[test]
fn test_render_error_with_multiline_message() {
    // Arrange
    let error = TemplateError::RenderError {
        template: "test.hbs".to_string(),
        message: "Line 1\nLine 2\nLine 3".to_string(),
    };

    // Act
    let display = format!("{}", error);

    // Assert
    assert!(display.contains("Line 1"));
    assert!(display.contains("Line 2"));
    assert!(display.contains("Line 3"));
}

// Error Matching Tests

#[test]
fn test_match_on_error_variant() {
    // Arrange
    let error = TemplateError::NotFound {
        template: "test.hbs".to_string(),
    };

    // Act
    let message = match error {
        TemplateError::NotFound { template } => format!("Template {} not found", template),
        TemplateError::ReadError { path, .. } => format!("Read error: {}", path),
        TemplateError::RegistrationError { template, .. } => {
            format!("Registration error: {}", template)
        }
        TemplateError::RenderError { template, .. } => format!("Render error: {}", template),
    };

    // Assert
    assert_eq!(message, "Template test.hbs not found");
}

#[test]
fn test_destructuring_read_error() {
    // Arrange
    let io_error = io::Error::new(io::ErrorKind::NotFound, "not found");
    let error = TemplateError::ReadError {
        path: "/path/to/template.hbs".to_string(),
        source: io_error,
    };

    // Act & Assert
    if let TemplateError::ReadError { path, source } = error {
        assert_eq!(path, "/path/to/template.hbs");
        assert_eq!(source.kind(), io::ErrorKind::NotFound);
    } else {
        panic!("Expected ReadError variant");
    }
}

#[test]
fn test_error_can_be_boxed() {
    // Arrange & Act
    let error: Box<dyn std::error::Error> = Box::new(TemplateError::NotFound {
        template: "test.hbs".to_string(),
    });

    // Assert
    let display = format!("{}", error);
    assert!(display.contains("Template not found"));
}
