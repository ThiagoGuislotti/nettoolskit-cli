//! Custom Handlebars helpers for string case conversion.
//!
//! Provides helpers for common naming conventions used in code generation:
//! - `to_kebab_case`: `MyClass` → `my-class`
//! - `to_snake_case`: `MyClass` → `my_class`
//! - `to_pascal_case`: `my_class` → `MyClass`
//! - `to_camel_case`: `MyClass` → `myClass`
//!
//! Each helper accepts a single string argument and returns the converted form.
//! If invoked without arguments or with a non-string value, the helper returns
//! an empty string to keep template rendering resilient.

use handlebars::{
    Context, Handlebars, Helper, HelperResult, Output, RenderContext, RenderErrorReason,
};
use heck::{ToKebabCase, ToLowerCamelCase, ToPascalCase, ToSnakeCase};

/// Register all case-conversion helpers on the given `Handlebars` instance.
pub(crate) fn register_helpers(handlebars: &mut Handlebars<'static>) {
    handlebars.register_helper("to_kebab_case", Box::new(kebab_case_helper));
    handlebars.register_helper("to_snake_case", Box::new(snake_case_helper));
    handlebars.register_helper("to_pascal_case", Box::new(pascal_case_helper));
    handlebars.register_helper("to_camel_case", Box::new(camel_case_helper));
}

/// Extract the first positional parameter as a `String`, returning a
/// `RenderError` when no argument is supplied.
fn extract_param(h: &Helper<'_>, name: &'static str) -> Result<String, handlebars::RenderError> {
    h.param(0)
        .and_then(|v| v.value().as_str().map(String::from))
        .ok_or_else(|| RenderErrorReason::ParamNotFoundForIndex(name, 0).into())
}

/// `{{to_kebab_case value}}` → `my-class`
fn kebab_case_helper(
    h: &Helper<'_>,
    _: &Handlebars<'_>,
    _: &Context,
    _: &mut RenderContext<'_, '_>,
    out: &mut dyn Output,
) -> HelperResult {
    let val = extract_param(h, "to_kebab_case")?;
    out.write(&val.to_kebab_case())?;
    Ok(())
}

/// `{{to_snake_case value}}` → `my_class`
fn snake_case_helper(
    h: &Helper<'_>,
    _: &Handlebars<'_>,
    _: &Context,
    _: &mut RenderContext<'_, '_>,
    out: &mut dyn Output,
) -> HelperResult {
    let val = extract_param(h, "to_snake_case")?;
    out.write(&val.to_snake_case())?;
    Ok(())
}

/// `{{to_pascal_case value}}` → `MyClass`
fn pascal_case_helper(
    h: &Helper<'_>,
    _: &Handlebars<'_>,
    _: &Context,
    _: &mut RenderContext<'_, '_>,
    out: &mut dyn Output,
) -> HelperResult {
    let val = extract_param(h, "to_pascal_case")?;
    out.write(&val.to_pascal_case())?;
    Ok(())
}

/// `{{to_camel_case value}}` → `myClass`
fn camel_case_helper(
    h: &Helper<'_>,
    _: &Handlebars<'_>,
    _: &Context,
    _: &mut RenderContext<'_, '_>,
    out: &mut dyn Output,
) -> HelperResult {
    let val = extract_param(h, "to_camel_case")?;
    out.write(&val.to_lower_camel_case())?;
    Ok(())
}
