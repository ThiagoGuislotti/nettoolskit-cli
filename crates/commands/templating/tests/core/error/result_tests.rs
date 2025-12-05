use nettoolskit_templating::{TemplateError, TemplateResult};

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