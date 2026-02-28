//! Translation handler implementation

use crate::core::TranslateRequest;
use nettoolskit_core::ExitStatus;
use nettoolskit_otel::Metrics;
use nettoolskit_templating::{Language, LanguageStrategy, LanguageStrategyFactory};
use nettoolskit_ui::Color;
use owo_colors::OwoColorize;
use std::path::Path;
use std::sync::Arc;
use tracing::{error, info};

/// Handle /translate command - Translate templates between languages
///
/// # Arguments
///
/// * `request` - Translation request with source, target, and path
///
/// # Returns
///
/// Returns `ExitStatus` indicating success or failure
pub async fn handle_translate(request: TranslateRequest) -> ExitStatus {
    let metrics = Metrics::new();
    metrics.increment_counter("translate_command_usage");

    info!(
        from = %request.from,
        to = %request.to,
        path = %request.path,
        "Starting template translation"
    );

    println!("{}", "🏭 Translating Template...".color(Color::PURPLE));
    println!("  📋 Source Language: {}", request.from.yellow());
    println!("  🎯 Target Language: {}", request.to.cyan());
    println!("  📄 Template Path: {}", request.path.green());

    // Step 1: Parse source and target languages
    let source_lang = match Language::parse(&request.from) {
        Some(lang) => lang,
        None => {
            error!("Unknown source language: {}", request.from);
            println!(
                "{}",
                format!("❌ Unknown source language: {}", request.from).red()
            );
            println!("   Supported languages: dotnet, java, go, python, rust, clojure, typescript");
            metrics.increment_counter("translate_invalid_source_language");
            return ExitStatus::Error;
        }
    };

    let target_lang = match Language::parse(&request.to) {
        Some(lang) => lang,
        None => {
            error!("Unknown target language: {}", request.to);
            println!(
                "{}",
                format!("❌ Unknown target language: {}", request.to).red()
            );
            println!("   Supported languages: dotnet, java, go, python, rust, clojure, typescript");
            metrics.increment_counter("translate_invalid_target_language");
            return ExitStatus::Error;
        }
    };

    // Step 2: Validate source and target are different
    if source_lang == target_lang {
        error!(
            "Source and target languages are the same: {:?}",
            source_lang
        );
        println!(
            "{}",
            "❌ Source and target languages must be different".red()
        );
        metrics.increment_counter("translate_same_language");
        return ExitStatus::Error;
    }

    // Step 3: Validate template file exists
    let template_path = Path::new(&request.path);
    if !template_path.exists() {
        error!("Template file not found: {}", request.path);
        println!(
            "{}",
            format!("❌ Template file not found: {}", request.path).red()
        );
        metrics.increment_counter("translate_missing_template");
        return ExitStatus::Error;
    }

    // Step 4: Get language strategies
    let factory = LanguageStrategyFactory::new();

    let source_strategy = match factory.get_strategy(source_lang) {
        Some(strategy) => strategy,
        None => {
            error!(
                "No strategy available for source language: {:?}",
                source_lang
            );
            println!(
                "{}",
                format!("❌ No strategy for language: {:?}", source_lang).red()
            );
            metrics.increment_counter("translate_no_source_strategy");
            return ExitStatus::Error;
        }
    };

    let target_strategy = match factory.get_strategy(target_lang) {
        Some(strategy) => strategy,
        None => {
            error!(
                "No strategy available for target language: {:?}",
                target_lang
            );
            println!(
                "{}",
                format!("❌ No strategy for language: {:?}", target_lang).red()
            );
            metrics.increment_counter("translate_no_target_strategy");
            return ExitStatus::Error;
        }
    };

    // Step 5: Perform translation based on target language
    println!(
        "Translating {} → {} ({})",
        source_strategy.language_id(),
        target_strategy.language_id(),
        request.path
    );

    match target_lang {
        Language::DotNet => {
            match translate_to_dotnet(&request.path, &source_strategy, &target_strategy).await {
                Ok(output_path) => {
                    println!("Created: {}", output_path.green());
                    metrics.increment_counter("translate_success");
                    info!("Translation completed: {}", output_path);
                    ExitStatus::Success
                }
                Err(e) => {
                    error!("Translation failed: {}", e);
                    println!("{}", format!("Error: {}", e).red());
                    metrics.increment_counter("translate_error");
                    ExitStatus::Error
                }
            }
        }
        Language::Java => {
            match translate_to_java(&request.path, &source_strategy, &target_strategy).await {
                Ok(output_path) => {
                    println!("Created: {}", output_path.green());
                    metrics.increment_counter("translate_success");
                    info!("Translation completed: {}", output_path);
                    ExitStatus::Success
                }
                Err(e) => {
                    error!("Translation failed: {}", e);
                    println!("{}", format!("Error: {}", e).red());
                    metrics.increment_counter("translate_error");
                    ExitStatus::Error
                }
            }
        }
        Language::Python => {
            match translate_to_python(&request.path, &source_strategy, &target_strategy).await {
                Ok(output_path) => {
                    println!("Created: {}", output_path.green());
                    metrics.increment_counter("translate_success");
                    info!("Translation completed: {}", output_path);
                    ExitStatus::Success
                }
                Err(e) => {
                    error!("Translation failed: {}", e);
                    println!("{}", format!("Error: {}", e).red());
                    metrics.increment_counter("translate_error");
                    ExitStatus::Error
                }
            }
        }
        Language::Go => {
            match translate_to_go(&request.path, &source_strategy, &target_strategy).await {
                Ok(output_path) => {
                    println!("Created: {}", output_path.green());
                    metrics.increment_counter("translate_success");
                    info!("Translation completed: {}", output_path);
                    ExitStatus::Success
                }
                Err(e) => {
                    error!("Translation failed: {}", e);
                    println!("{}", format!("Error: {}", e).red());
                    metrics.increment_counter("translate_error");
                    ExitStatus::Error
                }
            }
        }
        Language::Rust => {
            match translate_to_rust(&request.path, &source_strategy, &target_strategy).await {
                Ok(output_path) => {
                    println!("Created: {}", output_path.green());
                    metrics.increment_counter("translate_success");
                    info!("Translation completed: {}", output_path);
                    ExitStatus::Success
                }
                Err(e) => {
                    error!("Translation failed: {}", e);
                    println!("{}", format!("Error: {}", e).red());
                    metrics.increment_counter("translate_error");
                    ExitStatus::Error
                }
            }
        }
        Language::Clojure | Language::TypeScript => {
            error!(
                "Translation to {} not yet implemented",
                target_strategy.language_id()
            );
            println!(
                "{}",
                format!(
                    "❌ Translation to {} not yet implemented",
                    target_strategy.language_id()
                )
                .red()
            );
            metrics.increment_counter("translate_not_implemented");
            ExitStatus::Error
        }
    }
}

/// Translate template to .NET/C# format
async fn translate_to_dotnet(
    template_path: &str,
    source_strategy: &Arc<dyn LanguageStrategy>,
    target_strategy: &Arc<dyn LanguageStrategy>,
) -> Result<String, String> {
    translate_pipeline(
        template_path,
        source_strategy,
        target_strategy,
        convert_to_dotnet_conventions,
    )
    .await
}

/// Translate template to Java format
async fn translate_to_java(
    template_path: &str,
    source_strategy: &Arc<dyn LanguageStrategy>,
    target_strategy: &Arc<dyn LanguageStrategy>,
) -> Result<String, String> {
    translate_pipeline(
        template_path,
        source_strategy,
        target_strategy,
        convert_to_java_conventions,
    )
    .await
}

/// Translate template to Python format
async fn translate_to_python(
    template_path: &str,
    source_strategy: &Arc<dyn LanguageStrategy>,
    target_strategy: &Arc<dyn LanguageStrategy>,
) -> Result<String, String> {
    translate_pipeline(
        template_path,
        source_strategy,
        target_strategy,
        convert_to_python_conventions,
    )
    .await
}

/// Translate template to Go format
async fn translate_to_go(
    template_path: &str,
    source_strategy: &Arc<dyn LanguageStrategy>,
    target_strategy: &Arc<dyn LanguageStrategy>,
) -> Result<String, String> {
    translate_pipeline(
        template_path,
        source_strategy,
        target_strategy,
        convert_to_go_conventions,
    )
    .await
}

/// Translate template to Rust format
async fn translate_to_rust(
    template_path: &str,
    source_strategy: &Arc<dyn LanguageStrategy>,
    target_strategy: &Arc<dyn LanguageStrategy>,
) -> Result<String, String> {
    translate_pipeline(
        template_path,
        source_strategy,
        target_strategy,
        convert_to_rust_conventions,
    )
    .await
}

/// Shared translation pipeline used by all language-specific translators
async fn translate_pipeline(
    template_path: &str,
    source_strategy: &Arc<dyn LanguageStrategy>,
    target_strategy: &Arc<dyn LanguageStrategy>,
    convert_fn: fn(&str) -> String,
) -> Result<String, String> {
    info!(
        "Starting {} translation from {}",
        target_strategy.language_id(),
        template_path
    );

    let loader = TemplateLoader;
    let content = loader.load(template_path)?;

    let extractor = PlaceholderExtractor;
    let placeholders = extractor.extract(&content);
    info!("Found {} placeholders", placeholders.len());

    let converted = convert_fn(&content);

    let output_path = calculate_output_path(template_path, target_strategy.file_extension());
    info!("Output path: {}", output_path);

    // Avoid unused variable warnings for strategies kept for future extensibility
    let _ = source_strategy;

    let writer = OutputWriter;
    writer.write(&converted, &output_path)?;

    info!("Translation completed successfully");
    Ok(output_path)
}

// Internal helpers

struct TemplateLoader;
impl TemplateLoader {
    fn load(&self, path: &str) -> Result<String, String> {
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read template: {}", e))
    }
}

struct PlaceholderExtractor;
impl PlaceholderExtractor {
    fn extract(&self, content: &str) -> Vec<String> {
        extract_placeholders(content)
    }
}

struct OutputWriter;
impl OutputWriter {
    fn write(&self, content: &str, path: &str) -> Result<(), String> {
        std::fs::write(path, content).map_err(|e| format!("Failed to write output: {}", e))
    }
}

fn extract_placeholders(content: &str) -> Vec<String> {
    let mut placeholders = Vec::new();
    let mut chars = content.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '{' {
            if let Some(&'{') = chars.peek() {
                chars.next();
                let mut name = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch == '}' {
                        chars.next();
                        if let Some(&'}') = chars.peek() {
                            chars.next();
                            placeholders.push(name.trim().to_string());
                            break;
                        }
                    } else {
                        name.push(ch);
                        chars.next();
                    }
                }
            }
        }
    }

    placeholders.sort();
    placeholders.dedup();
    placeholders
}

fn convert_to_dotnet_conventions(content: &str) -> String {
    let mut result = content.to_string();

    result = result
        .replace("{{class_name}}", "{{ClassName}}")
        .replace("{{namespace}}", "{{Namespace}}")
        .replace("{{property_name}}", "{{PropertyName}}")
        .replace("{{method_name}}", "{{MethodName}}")
        .replace("{{interface_name}}", "{{InterfaceName}}")
        .replace("{{base_class}}", "{{BaseClass}}")
        .replace("{{entity_name}}", "{{EntityName}}")
        .replace("{{service_name}}", "{{ServiceName}}")
        .replace("{{repository_name}}", "{{RepositoryName}}")
        .replace("{{dto_name}}", "{{DtoName}}")
        .replace("{{command_name}}", "{{CommandName}}")
        .replace("{{query_name}}", "{{QueryName}}")
        .replace("{{validator_name}}", "{{ValidatorName}}")
        .replace("{{handler_name}}", "{{HandlerName}}")
        .replace("{{controller_name}}", "{{ControllerName}}");

    if !result.contains("/// <summary>") && result.contains("class") {
        result = format!(
            "/// <summary>\n/// Generated class from template translation\n/// </summary>\n{}",
            result
        );
    }

    result
}

/// Convert placeholders to Java naming conventions (camelCase for fields/methods, PascalCase for classes)
fn convert_to_java_conventions(content: &str) -> String {
    let mut result = content.to_string();

    result = result
        .replace("{{ClassName}}", "{{className}}")
        .replace("{{class_name}}", "{{className}}")
        .replace("{{Namespace}}", "{{packageName}}")
        .replace("{{namespace}}", "{{packageName}}")
        .replace("{{PropertyName}}", "{{fieldName}}")
        .replace("{{property_name}}", "{{fieldName}}")
        .replace("{{MethodName}}", "{{methodName}}")
        .replace("{{method_name}}", "{{methodName}}")
        .replace("{{InterfaceName}}", "{{interfaceName}}")
        .replace("{{interface_name}}", "{{interfaceName}}")
        .replace("{{BaseClass}}", "{{baseClass}}")
        .replace("{{base_class}}", "{{baseClass}}")
        .replace("{{EntityName}}", "{{entityName}}")
        .replace("{{entity_name}}", "{{entityName}}")
        .replace("{{ServiceName}}", "{{serviceName}}")
        .replace("{{service_name}}", "{{serviceName}}")
        .replace("{{RepositoryName}}", "{{repositoryName}}")
        .replace("{{repository_name}}", "{{repositoryName}}")
        .replace("{{DtoName}}", "{{dtoName}}")
        .replace("{{dto_name}}", "{{dtoName}}")
        .replace("{{CommandName}}", "{{commandName}}")
        .replace("{{command_name}}", "{{commandName}}")
        .replace("{{QueryName}}", "{{queryName}}")
        .replace("{{query_name}}", "{{queryName}}")
        .replace("{{ValidatorName}}", "{{validatorName}}")
        .replace("{{validator_name}}", "{{validatorName}}")
        .replace("{{HandlerName}}", "{{handlerName}}")
        .replace("{{handler_name}}", "{{handlerName}}")
        .replace("{{ControllerName}}", "{{controllerName}}")
        .replace("{{controller_name}}", "{{controllerName}}");

    result
}

/// Convert placeholders to Python naming conventions (snake_case for functions/fields, PascalCase for classes)
fn convert_to_python_conventions(content: &str) -> String {
    let mut result = content.to_string();

    result = result
        .replace("{{ClassName}}", "{{class_name}}")
        .replace("{{className}}", "{{class_name}}")
        .replace("{{Namespace}}", "{{module_name}}")
        .replace("{{namespace}}", "{{module_name}}")
        .replace("{{PropertyName}}", "{{property_name}}")
        .replace("{{propertyName}}", "{{property_name}}")
        .replace("{{MethodName}}", "{{method_name}}")
        .replace("{{methodName}}", "{{method_name}}")
        .replace("{{InterfaceName}}", "{{interface_name}}")
        .replace("{{interfaceName}}", "{{interface_name}}")
        .replace("{{BaseClass}}", "{{base_class}}")
        .replace("{{baseClass}}", "{{base_class}}")
        .replace("{{EntityName}}", "{{entity_name}}")
        .replace("{{entityName}}", "{{entity_name}}")
        .replace("{{ServiceName}}", "{{service_name}}")
        .replace("{{serviceName}}", "{{service_name}}")
        .replace("{{RepositoryName}}", "{{repository_name}}")
        .replace("{{repositoryName}}", "{{repository_name}}")
        .replace("{{DtoName}}", "{{dto_name}}")
        .replace("{{dtoName}}", "{{dto_name}}")
        .replace("{{CommandName}}", "{{command_name}}")
        .replace("{{commandName}}", "{{command_name}}")
        .replace("{{QueryName}}", "{{query_name}}")
        .replace("{{queryName}}", "{{query_name}}")
        .replace("{{ValidatorName}}", "{{validator_name}}")
        .replace("{{validatorName}}", "{{validator_name}}")
        .replace("{{HandlerName}}", "{{handler_name}}")
        .replace("{{handlerName}}", "{{handler_name}}")
        .replace("{{ControllerName}}", "{{controller_name}}")
        .replace("{{controllerName}}", "{{controller_name}}");

    result
}

/// Convert placeholders to Go naming conventions (PascalCase for exported, camelCase for unexported)
fn convert_to_go_conventions(content: &str) -> String {
    let mut result = content.to_string();

    result = result
        .replace("{{class_name}}", "{{StructName}}")
        .replace("{{ClassName}}", "{{StructName}}")
        .replace("{{className}}", "{{StructName}}")
        .replace("{{namespace}}", "{{packageName}}")
        .replace("{{Namespace}}", "{{packageName}}")
        .replace("{{property_name}}", "{{FieldName}}")
        .replace("{{PropertyName}}", "{{FieldName}}")
        .replace("{{propertyName}}", "{{FieldName}}")
        .replace("{{method_name}}", "{{MethodName}}")
        .replace("{{methodName}}", "{{MethodName}}")
        .replace("{{interface_name}}", "{{InterfaceName}}")
        .replace("{{interfaceName}}", "{{InterfaceName}}")
        .replace("{{base_class}}", "{{EmbeddedStruct}}")
        .replace("{{BaseClass}}", "{{EmbeddedStruct}}")
        .replace("{{baseClass}}", "{{EmbeddedStruct}}")
        .replace("{{entity_name}}", "{{EntityName}}")
        .replace("{{entityName}}", "{{EntityName}}")
        .replace("{{service_name}}", "{{ServiceName}}")
        .replace("{{serviceName}}", "{{ServiceName}}")
        .replace("{{repository_name}}", "{{RepositoryName}}")
        .replace("{{repositoryName}}", "{{RepositoryName}}")
        .replace("{{dto_name}}", "{{DtoName}}")
        .replace("{{dtoName}}", "{{DtoName}}")
        .replace("{{command_name}}", "{{CommandName}}")
        .replace("{{commandName}}", "{{CommandName}}")
        .replace("{{query_name}}", "{{QueryName}}")
        .replace("{{queryName}}", "{{QueryName}}")
        .replace("{{validator_name}}", "{{ValidatorName}}")
        .replace("{{validatorName}}", "{{ValidatorName}}")
        .replace("{{handler_name}}", "{{HandlerName}}")
        .replace("{{handlerName}}", "{{HandlerName}}")
        .replace("{{controller_name}}", "{{ControllerName}}")
        .replace("{{controllerName}}", "{{ControllerName}}");

    result
}

/// Convert placeholders to Rust naming conventions (snake_case for functions/fields, PascalCase for types)
fn convert_to_rust_conventions(content: &str) -> String {
    let mut result = content.to_string();

    result = result
        .replace("{{ClassName}}", "{{struct_name}}")
        .replace("{{className}}", "{{struct_name}}")
        .replace("{{class_name}}", "{{struct_name}}")
        .replace("{{Namespace}}", "{{module_name}}")
        .replace("{{namespace}}", "{{module_name}}")
        .replace("{{PropertyName}}", "{{field_name}}")
        .replace("{{propertyName}}", "{{field_name}}")
        .replace("{{property_name}}", "{{field_name}}")
        .replace("{{MethodName}}", "{{method_name}}")
        .replace("{{methodName}}", "{{method_name}}")
        .replace("{{InterfaceName}}", "{{trait_name}}")
        .replace("{{interfaceName}}", "{{trait_name}}")
        .replace("{{interface_name}}", "{{trait_name}}")
        .replace("{{BaseClass}}", "{{base_trait}}")
        .replace("{{baseClass}}", "{{base_trait}}")
        .replace("{{base_class}}", "{{base_trait}}")
        .replace("{{EntityName}}", "{{entity_name}}")
        .replace("{{entityName}}", "{{entity_name}}")
        .replace("{{ServiceName}}", "{{service_name}}")
        .replace("{{serviceName}}", "{{service_name}}")
        .replace("{{RepositoryName}}", "{{repository_name}}")
        .replace("{{repositoryName}}", "{{repository_name}}")
        .replace("{{DtoName}}", "{{dto_name}}")
        .replace("{{dtoName}}", "{{dto_name}}")
        .replace("{{CommandName}}", "{{command_name}}")
        .replace("{{commandName}}", "{{command_name}}")
        .replace("{{QueryName}}", "{{query_name}}")
        .replace("{{queryName}}", "{{query_name}}")
        .replace("{{ValidatorName}}", "{{validator_name}}")
        .replace("{{validatorName}}", "{{validator_name}}")
        .replace("{{HandlerName}}", "{{handler_name}}")
        .replace("{{handlerName}}", "{{handler_name}}")
        .replace("{{ControllerName}}", "{{controller_name}}")
        .replace("{{controllerName}}", "{{controller_name}}");

    result
}

fn calculate_output_path(input_path: &str, target_extension: &str) -> String {
    let path = Path::new(input_path);
    let file_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("output");

    let clean_name = file_name.strip_suffix(".hbs").unwrap_or(file_name);

    let output_name = if clean_name.ends_with(&format!(".{}", target_extension)) {
        clean_name.to_string()
    } else {
        let stem = clean_name
            .rsplit_once('.')
            .map(|(s, _)| s)
            .unwrap_or(clean_name);
        format!("{}.{}", stem, target_extension)
    };

    let parent = path.parent().unwrap_or(Path::new("."));
    parent.join(output_name).to_string_lossy().to_string()
}
