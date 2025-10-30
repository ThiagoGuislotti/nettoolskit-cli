use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Ollama client for interacting with local Ollama instance
#[derive(Debug, Clone)]
pub struct OllamaClient {
    client: Client,
    base_url: String,
}

impl OllamaClient {
    /// Create a new Ollama client with optimized settings for concurrent operations
    pub fn new(base_url: Option<String>) -> Self {
        let base_url = base_url.unwrap_or_else(|| "http://localhost:11434".to_string());
        let client = Client::builder()
            .timeout(Duration::from_secs(30)) // Reduced timeout for faster failure detection
            .connect_timeout(Duration::from_secs(5)) // Connection timeout
            .pool_max_idle_per_host(10) // Enable connection pooling
            .pool_idle_timeout(Duration::from_secs(90)) // Pool idle timeout
            .http2_prior_knowledge() // Use HTTP/2 for better multiplexing
            .build()
            .expect("Failed to create HTTP client");

        Self { client, base_url }
    }

    /// Check if Ollama is available
    pub async fn is_available(&self) -> bool {
        match self.client.get(&format!("{}/api/tags", self.base_url)).send().await {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }

    /// List available models
    pub async fn list_models(&self) -> Result<Vec<String>> {
        let response = self
            .client
            .get(&format!("{}/api/tags", self.base_url))
            .send()
            .await?;

        let models: ModelsResponse = response.json().await?;
        Ok(models.models.into_iter().map(|m| m.name).collect())
    }

    /// Generate completion using Ollama
    pub async fn generate(&self, request: &GenerateRequest) -> Result<GenerateResponse> {
        let response = self
            .client
            .post(&format!("{}/api/generate", self.base_url))
            .json(request)
            .send()
            .await?;

        let generate_response: GenerateResponse = response.json().await?;
        Ok(generate_response)
    }

    /// Generate multiple completions concurrently
    pub async fn generate_concurrent(&self, requests: Vec<GenerateRequest>) -> Result<Vec<Result<GenerateResponse>>> {
        use futures::future::join_all;

        let futures: Vec<_> = requests
            .iter()
            .map(|request| self.generate(request))
            .collect();

        let results = join_all(futures).await;
        Ok(results)
    }
}

impl Default for OllamaClient {
    fn default() -> Self {
        Self::new(None)
    }
}

#[derive(Debug, Deserialize)]
struct ModelsResponse {
    models: Vec<ModelInfo>,
}

#[derive(Debug, Deserialize)]
struct ModelInfo {
    name: String,
}

/// Request for generating completion
#[derive(Debug, Serialize)]
pub struct GenerateRequest {
    pub model: String,
    pub prompt: String,
    pub stream: bool,
}

/// Response from generate endpoint
#[derive(Debug, Deserialize)]
pub struct GenerateResponse {
    pub response: String,
    pub done: bool,
}