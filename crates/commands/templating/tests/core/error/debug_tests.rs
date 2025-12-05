use nettoolskit_templating::TemplateError;
use std::io;

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