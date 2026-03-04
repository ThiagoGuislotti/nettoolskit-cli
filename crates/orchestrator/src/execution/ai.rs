//! AI provider abstraction for CLI assistant workflows.
//!
//! This module defines request/response contracts for AI integrations and
//! provides a deterministic mock provider used by tests and local development.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fmt::{Display, Formatter};
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Boxed future returned by AI providers.
pub type AiProviderFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Role used by a conversation message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiRole {
    /// Instruction-level context (usually hidden from end user).
    System,
    /// Message authored by user.
    User,
    /// Message authored by assistant.
    Assistant,
}

/// Conversation message entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiMessage {
    /// Message role.
    pub role: AiRole,
    /// Message content.
    pub content: String,
}

impl AiMessage {
    /// Build a message from role + content.
    #[must_use]
    pub fn new(role: AiRole, content: impl Into<String>) -> Self {
        Self {
            role,
            content: content.into(),
        }
    }
}

/// AI request payload.
#[derive(Debug, Clone, PartialEq)]
pub struct AiRequest {
    /// Target model identifier.
    pub model: Option<String>,
    /// Ordered conversation messages.
    pub messages: Vec<AiMessage>,
    /// Optional output token cap.
    pub max_output_tokens: Option<u32>,
    /// Optional temperature value.
    pub temperature: Option<f32>,
    /// Request streaming response when supported.
    pub stream: bool,
}

impl AiRequest {
    /// Build a simple request from a single user prompt.
    #[must_use]
    pub fn from_user_prompt(prompt: impl Into<String>) -> Self {
        Self {
            model: None,
            messages: vec![AiMessage::new(AiRole::User, prompt)],
            max_output_tokens: None,
            temperature: None,
            stream: false,
        }
    }
}

/// Provider usage stats for a response.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AiUsage {
    /// Prompt/input token estimate.
    pub input_tokens: u32,
    /// Generated/output token estimate.
    pub output_tokens: u32,
}

/// AI response payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiResponse {
    /// Model identifier used by provider.
    pub model: String,
    /// Final assistant output.
    pub output_text: String,
    /// Finish reason (`stop`, `length`, etc.).
    pub finish_reason: String,
    /// Optional usage info.
    pub usage: AiUsage,
}

impl AiResponse {
    /// Build a response with default finish reason (`stop`) and zero usage.
    #[must_use]
    pub fn new(model: impl Into<String>, output_text: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            output_text: output_text.into(),
            finish_reason: "stop".to_string(),
            usage: AiUsage::default(),
        }
    }
}

/// Streaming chunk item for incremental output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiChunk {
    /// Incremental chunk text.
    pub content: String,
    /// Indicates final chunk.
    pub done: bool,
}

/// AI provider error contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AiProviderError {
    /// Request payload is invalid before provider call.
    InvalidRequest(String),
    /// Provider returned malformed or empty response payload.
    InvalidResponse(String),
    /// Request timed out.
    Timeout {
        /// Timeout budget that was exceeded.
        timeout: Duration,
    },
    /// Provider endpoint is unavailable.
    Unavailable(String),
    /// Transport/protocol error.
    Transport(String),
}

impl Display for AiProviderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidRequest(msg) => write!(f, "invalid AI request: {msg}"),
            Self::InvalidResponse(msg) => write!(f, "invalid AI response: {msg}"),
            Self::Timeout { timeout } => {
                write!(f, "AI provider timeout after {}ms", timeout.as_millis())
            }
            Self::Unavailable(msg) => write!(f, "AI provider unavailable: {msg}"),
            Self::Transport(msg) => write!(f, "AI provider transport error: {msg}"),
        }
    }
}

impl std::error::Error for AiProviderError {}

/// Provider abstraction used by orchestrator.
pub trait AiProvider: Send + Sync {
    /// Stable provider id.
    fn id(&self) -> &'static str;

    /// Complete request with a single final response.
    fn complete(
        &self,
        request: AiRequest,
    ) -> AiProviderFuture<'_, Result<AiResponse, AiProviderError>>;

    /// Stream request output as ordered chunks.
    fn stream(
        &self,
        request: AiRequest,
    ) -> AiProviderFuture<'_, Result<Vec<AiChunk>, AiProviderError>> {
        Box::pin(async move {
            let response = self.complete(request).await?;
            Ok(vec![AiChunk {
                content: response.output_text,
                done: true,
            }])
        })
    }
}

/// Scripted outcomes for deterministic mock provider behavior.
#[derive(Debug, Clone)]
pub enum MockAiOutcome {
    /// Return a complete response.
    Complete(AiResponse),
    /// Return streaming chunks.
    Stream(Vec<AiChunk>),
    /// Return an explicit error.
    Error(AiProviderError),
}

/// Deterministic provider for tests and local development.
#[derive(Clone)]
pub struct MockAiProvider {
    scripted: Arc<Mutex<VecDeque<MockAiOutcome>>>,
    default_response: AiResponse,
    delay: Duration,
}

impl MockAiProvider {
    /// Build mock provider with a default response used when scripts are empty.
    #[must_use]
    pub fn new(default_response: AiResponse) -> Self {
        Self {
            scripted: Arc::new(Mutex::new(VecDeque::new())),
            default_response,
            delay: Duration::ZERO,
        }
    }

    /// Build mock provider with scripted outcomes consumed in FIFO order.
    #[must_use]
    pub fn with_scripted(default_response: AiResponse, outcomes: Vec<MockAiOutcome>) -> Self {
        Self {
            scripted: Arc::new(Mutex::new(VecDeque::from(outcomes))),
            default_response,
            delay: Duration::ZERO,
        }
    }

    /// Configure deterministic artificial delay for each call.
    #[must_use]
    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }

    fn pop_outcome(&self) -> Option<MockAiOutcome> {
        let mut queue = self
            .scripted
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        queue.pop_front()
    }
}

/// Configuration for an OpenAI-compatible chat completion provider.
#[derive(Debug, Clone)]
pub struct OpenAiCompatibleProviderConfig {
    /// Endpoint URL for chat completions.
    pub endpoint: String,
    /// Optional API key used as bearer token.
    pub api_key: Option<String>,
    /// Default model used when request model is empty.
    pub default_model: String,
    /// Request timeout budget.
    pub timeout: Duration,
    /// Deterministic fallback text used for transport/unavailability errors.
    pub fallback_output_text: Option<String>,
}

impl Default for OpenAiCompatibleProviderConfig {
    fn default() -> Self {
        Self {
            endpoint: "https://api.openai.com/v1/chat/completions".to_string(),
            api_key: None,
            default_model: "gpt-4.1-mini".to_string(),
            timeout: Duration::from_secs(30),
            fallback_output_text: None,
        }
    }
}

impl OpenAiCompatibleProviderConfig {
    fn validate(&self) -> Result<(), AiProviderError> {
        if self.endpoint.trim().is_empty() {
            return Err(AiProviderError::InvalidRequest(
                "OpenAI-compatible endpoint must not be empty".to_string(),
            ));
        }
        if self.default_model.trim().is_empty() {
            return Err(AiProviderError::InvalidRequest(
                "OpenAI-compatible default model must not be empty".to_string(),
            ));
        }
        if self.timeout.is_zero() {
            return Err(AiProviderError::InvalidRequest(
                "OpenAI-compatible timeout must be greater than zero".to_string(),
            ));
        }
        Ok(())
    }
}

/// OpenAI-compatible provider implementation using chat completions schema.
pub struct OpenAiCompatibleProvider {
    config: OpenAiCompatibleProviderConfig,
    client: reqwest::Client,
}

impl OpenAiCompatibleProvider {
    /// Build provider from configuration.
    ///
    /// # Errors
    ///
    /// Returns error when config is invalid or HTTP client setup fails.
    pub fn new(config: OpenAiCompatibleProviderConfig) -> Result<Self, AiProviderError> {
        config.validate()?;

        let client = reqwest::Client::builder()
            .build()
            .map_err(|error| AiProviderError::Transport(error.to_string()))?;

        Ok(Self { config, client })
    }

    fn resolve_model(&self, request: &AiRequest) -> String {
        request
            .model
            .as_deref()
            .map(str::trim)
            .filter(|model| !model.is_empty())
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| self.config.default_model.clone())
    }

    fn maybe_fallback(
        &self,
        model: &str,
        error: AiProviderError,
    ) -> Result<AiResponse, AiProviderError> {
        if matches!(
            error,
            AiProviderError::Timeout { .. }
                | AiProviderError::Unavailable(_)
                | AiProviderError::Transport(_)
        ) {
            if let Some(fallback_output) = self.config.fallback_output_text.as_deref() {
                return Ok(AiResponse {
                    model: model.to_string(),
                    output_text: fallback_output.to_string(),
                    finish_reason: "fallback".to_string(),
                    usage: AiUsage::default(),
                });
            }
        }

        Err(error)
    }

    async fn send_chat_completion(
        &self,
        payload: &OpenAiChatCompletionRequest,
    ) -> Result<(reqwest::StatusCode, String), AiProviderError> {
        let mut builder = self
            .client
            .post(&self.config.endpoint)
            .header(reqwest::header::CONTENT_TYPE, "application/json");

        if let Some(api_key) = self.config.api_key.as_deref().map(str::trim) {
            if !api_key.is_empty() {
                builder = builder.bearer_auth(api_key);
            }
        }

        let response = builder
            .json(payload)
            .send()
            .await
            .map_err(|error| AiProviderError::Transport(error.to_string()))?;
        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|error| AiProviderError::Transport(error.to_string()))?;
        Ok((status, body))
    }
}

impl AiProvider for MockAiProvider {
    fn id(&self) -> &'static str {
        "mock"
    }

    fn complete(
        &self,
        request: AiRequest,
    ) -> AiProviderFuture<'_, Result<AiResponse, AiProviderError>> {
        Box::pin(async move {
            validate_request(&request)?;
            if !self.delay.is_zero() {
                tokio::time::sleep(self.delay).await;
            }

            match self.pop_outcome() {
                Some(MockAiOutcome::Complete(response)) => {
                    validate_response(&response)?;
                    Ok(response)
                }
                Some(MockAiOutcome::Stream(chunks)) => {
                    let text = flatten_chunks(&chunks)?;
                    let mut response = self.default_response.clone();
                    response.output_text = text;
                    validate_response(&response)?;
                    Ok(response)
                }
                Some(MockAiOutcome::Error(error)) => Err(error),
                None => {
                    validate_response(&self.default_response)?;
                    Ok(self.default_response.clone())
                }
            }
        })
    }

    fn stream(
        &self,
        request: AiRequest,
    ) -> AiProviderFuture<'_, Result<Vec<AiChunk>, AiProviderError>> {
        Box::pin(async move {
            validate_request(&request)?;
            if !self.delay.is_zero() {
                tokio::time::sleep(self.delay).await;
            }

            match self.pop_outcome() {
                Some(MockAiOutcome::Stream(chunks)) => {
                    validate_chunks(&chunks)?;
                    Ok(chunks)
                }
                Some(MockAiOutcome::Complete(response)) => {
                    validate_response(&response)?;
                    Ok(vec![AiChunk {
                        content: response.output_text,
                        done: true,
                    }])
                }
                Some(MockAiOutcome::Error(error)) => Err(error),
                None => {
                    validate_response(&self.default_response)?;
                    Ok(vec![AiChunk {
                        content: self.default_response.output_text.clone(),
                        done: true,
                    }])
                }
            }
        })
    }
}

impl AiProvider for OpenAiCompatibleProvider {
    fn id(&self) -> &'static str {
        "openai-compatible"
    }

    fn complete(
        &self,
        request: AiRequest,
    ) -> AiProviderFuture<'_, Result<AiResponse, AiProviderError>> {
        Box::pin(async move {
            validate_request(&request)?;
            let model = self.resolve_model(&request);

            let payload = OpenAiChatCompletionRequest::from_request(&model, &request);
            let result =
                tokio::time::timeout(self.config.timeout, self.send_chat_completion(&payload))
                    .await;

            let (status, body) = match result {
                Ok(Ok(value)) => value,
                Ok(Err(error)) => return self.maybe_fallback(&model, error),
                Err(_) => {
                    return self.maybe_fallback(
                        &model,
                        AiProviderError::Timeout {
                            timeout: self.config.timeout,
                        },
                    )
                }
            };

            if !status.is_success() {
                let message = format!(
                    "HTTP {}: {}",
                    status.as_u16(),
                    body.trim().replace('\n', " ")
                );
                let error = if status.is_server_error()
                    || status == reqwest::StatusCode::TOO_MANY_REQUESTS
                {
                    AiProviderError::Unavailable(message)
                } else {
                    AiProviderError::Transport(message)
                };
                return self.maybe_fallback(&model, error);
            }

            let parsed: OpenAiChatCompletionResponse = serde_json::from_str(&body)
                .map_err(|error| AiProviderError::InvalidResponse(error.to_string()))?;
            let choice = parsed.choices.first().ok_or_else(|| {
                AiProviderError::InvalidResponse("response.choices must not be empty".to_string())
            })?;

            let output_text = choice
                .message
                .as_ref()
                .and_then(|message| message.content.clone())
                .or_else(|| choice.text.clone())
                .unwrap_or_default();

            let finish_reason = choice
                .finish_reason
                .clone()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| "stop".to_string());

            let response = AiResponse {
                model: parsed
                    .model
                    .filter(|value| !value.trim().is_empty())
                    .unwrap_or(model),
                output_text,
                finish_reason,
                usage: AiUsage {
                    input_tokens: parsed
                        .usage
                        .as_ref()
                        .and_then(|usage| usage.prompt_tokens)
                        .unwrap_or(0),
                    output_tokens: parsed
                        .usage
                        .as_ref()
                        .and_then(|usage| usage.completion_tokens)
                        .unwrap_or(0),
                },
            };
            validate_response(&response)?;
            Ok(response)
        })
    }

    fn stream(
        &self,
        mut request: AiRequest,
    ) -> AiProviderFuture<'_, Result<Vec<AiChunk>, AiProviderError>> {
        Box::pin(async move {
            request.stream = true;
            let response = self.complete(request).await?;
            Ok(vec![AiChunk {
                content: response.output_text,
                done: true,
            }])
        })
    }
}

#[derive(Debug, Clone, Serialize)]
struct OpenAiChatCompletionRequest {
    model: String,
    messages: Vec<OpenAiChatCompletionMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    stream: bool,
}

impl OpenAiChatCompletionRequest {
    fn from_request(model: &str, request: &AiRequest) -> Self {
        Self {
            model: model.to_string(),
            messages: request
                .messages
                .iter()
                .map(OpenAiChatCompletionMessage::from_ai_message)
                .collect(),
            max_tokens: request.max_output_tokens,
            temperature: request.temperature,
            stream: request.stream,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct OpenAiChatCompletionMessage {
    role: &'static str,
    content: String,
}

impl OpenAiChatCompletionMessage {
    fn from_ai_message(message: &AiMessage) -> Self {
        Self {
            role: match message.role {
                AiRole::System => "system",
                AiRole::User => "user",
                AiRole::Assistant => "assistant",
            },
            content: message.content.clone(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct OpenAiChatCompletionResponse {
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    choices: Vec<OpenAiChatCompletionChoice>,
    #[serde(default)]
    usage: Option<OpenAiChatCompletionUsage>,
}

#[derive(Debug, Clone, Deserialize)]
struct OpenAiChatCompletionChoice {
    #[serde(default)]
    message: Option<OpenAiChatCompletionMessageResponse>,
    #[serde(default)]
    finish_reason: Option<String>,
    #[serde(default)]
    text: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct OpenAiChatCompletionMessageResponse {
    #[serde(default)]
    content: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct OpenAiChatCompletionUsage {
    #[serde(default)]
    prompt_tokens: Option<u32>,
    #[serde(default)]
    completion_tokens: Option<u32>,
}

fn validate_request(request: &AiRequest) -> Result<(), AiProviderError> {
    if request.messages.is_empty() {
        return Err(AiProviderError::InvalidRequest(
            "request.messages must include at least one message".to_string(),
        ));
    }

    if request
        .messages
        .iter()
        .all(|message| message.content.trim().is_empty())
    {
        return Err(AiProviderError::InvalidRequest(
            "request.messages must include non-empty content".to_string(),
        ));
    }

    Ok(())
}

fn validate_response(response: &AiResponse) -> Result<(), AiProviderError> {
    if response.model.trim().is_empty() {
        return Err(AiProviderError::InvalidResponse(
            "response.model must not be empty".to_string(),
        ));
    }

    if response.output_text.trim().is_empty() {
        return Err(AiProviderError::InvalidResponse(
            "response.output_text must not be empty".to_string(),
        ));
    }

    Ok(())
}

fn validate_chunks(chunks: &[AiChunk]) -> Result<(), AiProviderError> {
    if chunks.is_empty() {
        return Err(AiProviderError::InvalidResponse(
            "stream must include at least one chunk".to_string(),
        ));
    }

    if !chunks.iter().any(|chunk| chunk.done) {
        return Err(AiProviderError::InvalidResponse(
            "stream must include a final chunk with done=true".to_string(),
        ));
    }

    Ok(())
}

fn flatten_chunks(chunks: &[AiChunk]) -> Result<String, AiProviderError> {
    validate_chunks(chunks)?;

    let mut content = String::new();
    for chunk in chunks {
        content.push_str(&chunk.content);
    }

    if content.trim().is_empty() {
        return Err(AiProviderError::InvalidResponse(
            "stream produced empty output".to_string(),
        ));
    }

    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;
    use tokio::task::JoinHandle;

    fn default_response() -> AiResponse {
        AiResponse::new("mock-model", "default response")
    }

    #[tokio::test]
    async fn mock_provider_returns_scripted_responses_in_order() {
        let provider = MockAiProvider::with_scripted(
            default_response(),
            vec![
                MockAiOutcome::Complete(AiResponse::new("mock-model", "first")),
                MockAiOutcome::Complete(AiResponse::new("mock-model", "second")),
            ],
        );

        let first = provider
            .complete(AiRequest::from_user_prompt("a"))
            .await
            .expect("first response should succeed");
        let second = provider
            .complete(AiRequest::from_user_prompt("b"))
            .await
            .expect("second response should succeed");

        assert_eq!(first.output_text, "first");
        assert_eq!(second.output_text, "second");
    }

    #[tokio::test]
    async fn mock_provider_returns_scripted_error_variants() {
        let provider = MockAiProvider::with_scripted(
            default_response(),
            vec![
                MockAiOutcome::Error(AiProviderError::Timeout {
                    timeout: Duration::from_secs(5),
                }),
                MockAiOutcome::Error(AiProviderError::Unavailable(
                    "provider maintenance".to_string(),
                )),
            ],
        );

        let timeout_error = provider
            .complete(AiRequest::from_user_prompt("a"))
            .await
            .expect_err("first call should fail");
        let unavailable_error = provider
            .complete(AiRequest::from_user_prompt("b"))
            .await
            .expect_err("second call should fail");

        assert!(matches!(
            timeout_error,
            AiProviderError::Timeout { timeout } if timeout == Duration::from_secs(5)
        ));
        assert!(matches!(
            unavailable_error,
            AiProviderError::Unavailable(msg) if msg.contains("maintenance")
        ));
    }

    #[tokio::test]
    async fn mock_provider_rejects_invalid_request_without_messages() {
        let provider = MockAiProvider::new(default_response());
        let request = AiRequest {
            model: None,
            messages: Vec::new(),
            max_output_tokens: None,
            temperature: None,
            stream: false,
        };

        let error = provider
            .complete(request)
            .await
            .expect_err("request must fail validation");

        assert!(matches!(error, AiProviderError::InvalidRequest(_)));
    }

    #[tokio::test]
    async fn mock_provider_flags_invalid_response_payloads() {
        let provider = MockAiProvider::with_scripted(
            default_response(),
            vec![MockAiOutcome::Complete(AiResponse::new("mock-model", ""))],
        );

        let error = provider
            .complete(AiRequest::from_user_prompt("hello"))
            .await
            .expect_err("empty output should fail");

        assert!(matches!(error, AiProviderError::InvalidResponse(_)));
    }

    #[tokio::test]
    async fn mock_provider_streams_scripted_chunks() {
        let provider = MockAiProvider::with_scripted(
            default_response(),
            vec![MockAiOutcome::Stream(vec![
                AiChunk {
                    content: "hello ".to_string(),
                    done: false,
                },
                AiChunk {
                    content: "world".to_string(),
                    done: true,
                },
            ])],
        );

        let chunks = provider
            .stream(AiRequest::from_user_prompt("stream"))
            .await
            .expect("stream should succeed");

        assert_eq!(chunks.len(), 2);
        assert!(!chunks[0].done);
        assert!(chunks[1].done);
    }

    #[tokio::test]
    async fn mock_provider_applies_deterministic_delay() {
        let provider =
            MockAiProvider::new(default_response()).with_delay(Duration::from_millis(20));
        let started = tokio::time::Instant::now();

        let _ = provider
            .complete(AiRequest::from_user_prompt("latency"))
            .await
            .expect("default response should succeed");

        assert!(
            started.elapsed() >= Duration::from_millis(20),
            "delay should be applied to provider calls"
        );
    }

    fn openai_response_body(content: &str) -> String {
        format!(
            r#"{{
  "id": "chatcmpl-test",
  "object": "chat.completion",
  "model": "gpt-4o-mini",
  "choices": [
    {{
      "index": 0,
      "message": {{
        "role": "assistant",
        "content": "{content}"
      }},
      "finish_reason": "stop"
    }}
  ],
  "usage": {{
    "prompt_tokens": 12,
    "completion_tokens": 7
  }}
}}"#
        )
    }

    async fn spawn_single_response_server(
        status_code: u16,
        body: String,
        delay: Option<Duration>,
    ) -> (String, JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("listener should bind");
        let address = listener.local_addr().expect("listener must have address");

        let handle = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.expect("accept should pass");

            let mut request_buffer = [0_u8; 4096];
            let _ = socket.read(&mut request_buffer).await;

            if let Some(delay_value) = delay {
                tokio::time::sleep(delay_value).await;
            }

            let reason = match status_code {
                200 => "OK",
                429 => "Too Many Requests",
                503 => "Service Unavailable",
                _ => "Error",
            };

            let response = format!(
                "HTTP/1.1 {status_code} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = socket.write_all(response.as_bytes()).await;
            let _ = socket.shutdown().await;
        });

        (format!("http://{address}/v1/chat/completions"), handle)
    }

    fn openai_config(
        endpoint: String,
        timeout: Duration,
        fallback_output_text: Option<&str>,
    ) -> OpenAiCompatibleProviderConfig {
        OpenAiCompatibleProviderConfig {
            endpoint,
            api_key: Some("test-key".to_string()),
            default_model: "gpt-4o-mini".to_string(),
            timeout,
            fallback_output_text: fallback_output_text.map(ToOwned::to_owned),
        }
    }

    #[tokio::test]
    async fn openai_provider_parses_success_response() {
        let (endpoint, server_handle) =
            spawn_single_response_server(200, openai_response_body("assistant output"), None).await;
        let provider =
            OpenAiCompatibleProvider::new(openai_config(endpoint, Duration::from_secs(1), None))
                .expect("provider config should be valid");

        let mut request = AiRequest::from_user_prompt("hello");
        request.temperature = Some(0.1);
        request.max_output_tokens = Some(128);

        let response = provider
            .complete(request)
            .await
            .expect("provider should parse response");
        server_handle.await.expect("server should complete");

        assert_eq!(response.model, "gpt-4o-mini");
        assert_eq!(response.output_text, "assistant output");
        assert_eq!(response.finish_reason, "stop");
        assert_eq!(response.usage.input_tokens, 12);
        assert_eq!(response.usage.output_tokens, 7);
    }

    #[tokio::test]
    async fn openai_provider_returns_timeout_error_without_fallback() {
        let (endpoint, server_handle) = spawn_single_response_server(
            200,
            openai_response_body("slow output"),
            Some(Duration::from_millis(80)),
        )
        .await;
        let provider =
            OpenAiCompatibleProvider::new(openai_config(endpoint, Duration::from_millis(10), None))
                .expect("provider config should be valid");

        let error = provider
            .complete(AiRequest::from_user_prompt("timeout"))
            .await
            .expect_err("request should timeout");
        server_handle.await.expect("server should complete");

        assert!(matches!(error, AiProviderError::Timeout { .. }));
    }

    #[tokio::test]
    async fn openai_provider_uses_fallback_on_transport_failure() {
        let provider = OpenAiCompatibleProvider::new(openai_config(
            "http://127.0.0.1:1/v1/chat/completions".to_string(),
            Duration::from_millis(100),
            Some("offline fallback"),
        ))
        .expect("provider config should be valid");

        let response = provider
            .complete(AiRequest::from_user_prompt("transport fail"))
            .await
            .expect("fallback should be used");

        assert_eq!(response.output_text, "offline fallback");
        assert_eq!(response.finish_reason, "fallback");
    }

    #[tokio::test]
    async fn openai_provider_uses_fallback_on_service_unavailable() {
        let (endpoint, server_handle) = spawn_single_response_server(
            503,
            r#"{"error":"service unavailable"}"#.to_string(),
            None,
        )
        .await;
        let provider = OpenAiCompatibleProvider::new(openai_config(
            endpoint,
            Duration::from_secs(1),
            Some("service fallback"),
        ))
        .expect("provider config should be valid");

        let response = provider
            .complete(AiRequest::from_user_prompt("retry"))
            .await
            .expect("fallback should be returned");
        server_handle.await.expect("server should complete");

        assert_eq!(response.output_text, "service fallback");
        assert_eq!(response.finish_reason, "fallback");
    }

    #[tokio::test]
    async fn openai_provider_rejects_malformed_success_payload() {
        let (endpoint, server_handle) =
            spawn_single_response_server(200, r#"{"choices":[]}"#.to_string(), None).await;
        let provider = OpenAiCompatibleProvider::new(openai_config(
            endpoint,
            Duration::from_secs(1),
            Some("fallback should not apply"),
        ))
        .expect("provider config should be valid");

        let error = provider
            .complete(AiRequest::from_user_prompt("invalid payload"))
            .await
            .expect_err("invalid payload should fail");
        server_handle.await.expect("server should complete");

        assert!(matches!(error, AiProviderError::InvalidResponse(_)));
    }
}
