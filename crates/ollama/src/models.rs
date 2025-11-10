/// Common model configurations for Ollama
#[derive(Debug, Clone)]
pub struct ModelConfig {
    pub name: String,
    pub temperature: f32,
    pub max_tokens: Option<u32>,
}

impl ModelConfig {
    /// Create configuration for code generation model
    pub fn code_generation() -> Self {
        Self {
            name: "codellama:latest".to_string(),
            temperature: 0.1,
            max_tokens: Some(2048),
        }
    }

    /// Create configuration for general chat model
    pub fn chat() -> Self {
        Self {
            name: "llama2:latest".to_string(),
            temperature: 0.7,
            max_tokens: Some(1024),
        }
    }

    /// Create configuration for documentation model
    pub fn documentation() -> Self {
        Self {
            name: "llama2:latest".to_string(),
            temperature: 0.3,
            max_tokens: Some(1536),
        }
    }
}

/// Available model types for different tasks
#[derive(Debug, Clone, Copy)]
pub enum ModelType {
    CodeGeneration,
    Chat,
    Documentation,
}

impl ModelType {
    pub fn config(self) -> ModelConfig {
        match self {
            ModelType::CodeGeneration => ModelConfig::code_generation(),
            ModelType::Chat => ModelConfig::chat(),
            ModelType::Documentation => ModelConfig::documentation(),
        }
    }

    pub fn all() -> &'static [ModelType] {
        &[
            ModelType::CodeGeneration,
            ModelType::Chat,
            ModelType::Documentation,
        ]
    }
}
