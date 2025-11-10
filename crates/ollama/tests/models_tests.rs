use nettoolskit_ollama::{ModelConfig, ModelType};

#[test]
fn test_model_config_code_generation() {
    let config = ModelConfig::code_generation();

    assert_eq!(config.name, "codellama:latest");
    assert_eq!(config.temperature, 0.1);
    assert_eq!(config.max_tokens, Some(2048));
}

#[test]
fn test_model_config_chat() {
    let config = ModelConfig::chat();

    assert_eq!(config.name, "llama2:latest");
    assert_eq!(config.temperature, 0.7);
    assert_eq!(config.max_tokens, Some(1024));
}

#[test]
fn test_model_config_documentation() {
    let config = ModelConfig::documentation();

    assert_eq!(config.name, "llama2:latest");
    assert_eq!(config.temperature, 0.3);
    assert_eq!(config.max_tokens, Some(1536));
}

#[test]
fn test_model_type_config_mapping() {
    let code_config = ModelType::CodeGeneration.config();
    assert_eq!(code_config.name, "codellama:latest");

    let chat_config = ModelType::Chat.config();
    assert_eq!(chat_config.name, "llama2:latest");

    let doc_config = ModelType::Documentation.config();
    assert_eq!(doc_config.name, "llama2:latest");
}

#[test]
fn test_model_type_all() {
    let all_types = ModelType::all();

    assert_eq!(all_types.len(), 3);
    assert!(matches!(all_types[0], ModelType::CodeGeneration));
    assert!(matches!(all_types[1], ModelType::Chat));
    assert!(matches!(all_types[2], ModelType::Documentation));
}

#[test]
fn test_model_config_debug() {
    let config = ModelConfig::code_generation();
    let debug_str = format!("{:?}", config);

    assert!(debug_str.contains("ModelConfig"));
    assert!(debug_str.contains("codellama:latest"));
    assert!(debug_str.contains("0.1"));
}

#[test]
fn test_model_config_clone() {
    let config = ModelConfig::chat();
    let cloned = config.clone();

    assert_eq!(config.name, cloned.name);
    assert_eq!(config.temperature, cloned.temperature);
    assert_eq!(config.max_tokens, cloned.max_tokens);
}

#[test]
fn test_model_type_debug() {
    let model_type = ModelType::CodeGeneration;
    let debug_str = format!("{:?}", model_type);

    assert!(debug_str.contains("CodeGeneration"));
}

#[test]
fn test_model_type_copy() {
    let original = ModelType::Chat;
    let copied = original;

    // Both should match
    assert!(matches!(original, ModelType::Chat));
    assert!(matches!(copied, ModelType::Chat));
}

#[test]
fn test_model_config_temperature_ranges() {
    let code_config = ModelConfig::code_generation();
    assert!(code_config.temperature >= 0.0 && code_config.temperature <= 1.0);

    let chat_config = ModelConfig::chat();
    assert!(chat_config.temperature >= 0.0 && chat_config.temperature <= 1.0);

    let doc_config = ModelConfig::documentation();
    assert!(doc_config.temperature >= 0.0 && doc_config.temperature <= 1.0);
}

#[test]
fn test_model_config_max_tokens_presence() {
    let code_config = ModelConfig::code_generation();
    assert!(code_config.max_tokens.is_some());
    assert!(code_config.max_tokens.unwrap() > 0);

    let chat_config = ModelConfig::chat();
    assert!(chat_config.max_tokens.is_some());
    assert!(chat_config.max_tokens.unwrap() > 0);
}

#[test]
fn test_model_name_consistency() {
    let chat_config = ModelConfig::chat();
    let doc_config = ModelConfig::documentation();

    // Chat and documentation use same model
    assert_eq!(chat_config.name, doc_config.name);

    let code_config = ModelConfig::code_generation();
    // Code generation uses different model
    assert_ne!(code_config.name, chat_config.name);
}
