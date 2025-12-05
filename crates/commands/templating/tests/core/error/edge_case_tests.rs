use nettoolskit_templating::TemplateError;

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