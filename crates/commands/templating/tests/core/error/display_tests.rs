use nettoolskit_templating::TemplateError;
use std::io;

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