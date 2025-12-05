use nettoolskit_templating::TemplateError;
use std::io;

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