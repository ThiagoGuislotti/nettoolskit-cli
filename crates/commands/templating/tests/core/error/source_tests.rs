use nettoolskit_templating::TemplateError;
use std::error::Error;
use std::io;

#[test]
fn test_read_error_has_source() {
    // Arrange
    let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let error = TemplateError::ReadError {
        path: "/test/file.hbs".to_string(),
        source: io_error,
    };

    // Act & Assert
    assert!(error.source().is_some());
}

#[test]
fn test_other_errors_no_source() {
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