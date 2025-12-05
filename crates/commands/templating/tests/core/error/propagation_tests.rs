use nettoolskit_templating::{TemplateError, TemplateResult};

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