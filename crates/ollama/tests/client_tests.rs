use nettoolskit_ollama::{GenerateRequest, GenerateResponse, OllamaClient};

#[test]
fn test_ollama_client_new() {
    let client = OllamaClient::new(None);

    // Should use default URL
    let debug_str = format!("{:?}", client);
    assert!(debug_str.contains("localhost:11434"));
}

#[test]
fn test_ollama_client_new_with_custom_url() {
    let custom_url = "http://custom:8080".to_string();
    let client = OllamaClient::new(Some(custom_url.clone()));

    let debug_str = format!("{:?}", client);
    assert!(debug_str.contains("custom:8080"));
}

#[test]
fn test_ollama_client_default() {
    let client = OllamaClient::default();

    let debug_str = format!("{:?}", client);
    assert!(debug_str.contains("localhost:11434"));
}

#[test]
fn test_ollama_client_clone() {
    let client = OllamaClient::new(None);
    let cloned = client.clone();

    let original_debug = format!("{:?}", client);
    let cloned_debug = format!("{:?}", cloned);

    // Should have similar structure
    assert!(original_debug.contains("localhost:11434"));
    assert!(cloned_debug.contains("localhost:11434"));
}

#[test]
fn test_generate_request_creation() {
    let request = GenerateRequest {
        model: "llama2:latest".to_string(),
        prompt: "Hello world".to_string(),
        stream: false,
    };

    assert_eq!(request.model, "llama2:latest");
    assert_eq!(request.prompt, "Hello world");
    assert!(!request.stream);
}

#[test]
fn test_generate_request_debug() {
    let request = GenerateRequest {
        model: "test-model".to_string(),
        prompt: "test prompt".to_string(),
        stream: true,
    };

    let debug_str = format!("{:?}", request);
    assert!(debug_str.contains("GenerateRequest"));
    assert!(debug_str.contains("test-model"));
    assert!(debug_str.contains("test prompt"));
    assert!(debug_str.contains("true"));
}

#[test]
fn test_generate_response_creation() {
    // We'll test with mock JSON data to validate deserialization structure
    let json_data = r#"{"response": "Test response", "done": true}"#;
    let response: Result<GenerateResponse, _> = serde_json::from_str(json_data);

    assert!(response.is_ok());
    let response = response.unwrap();
    assert_eq!(response.response, "Test response");
    assert!(response.done);
}

#[test]
fn test_generate_response_debug() {
    let json_data = r#"{"response": "Debug test", "done": false}"#;
    let response: GenerateResponse = serde_json::from_str(json_data).unwrap();

    let debug_str = format!("{:?}", response);
    assert!(debug_str.contains("GenerateResponse"));
    assert!(debug_str.contains("Debug test"));
    assert!(debug_str.contains("false"));
}

#[tokio::test]
async fn test_client_construction_stability() {
    // Test that creating multiple clients doesn't panic
    let client1 = OllamaClient::new(None);
    let client2 = OllamaClient::new(Some("http://test:1234".to_string()));
    let client3 = OllamaClient::default();

    // All should be valid
    assert!(format!("{:?}", client1).contains("OllamaClient"));
    assert!(format!("{:?}", client2).contains("OllamaClient"));
    assert!(format!("{:?}", client3).contains("OllamaClient"));
}

#[test]
fn test_generate_request_serialization() {
    let request = GenerateRequest {
        model: "serialization-test".to_string(),
        prompt: "Test serialization".to_string(),
        stream: false,
    };

    let json = serde_json::to_string(&request).unwrap();

    assert!(json.contains("serialization-test"));
    assert!(json.contains("Test serialization"));
    assert!(json.contains("false"));
}

#[test]
fn test_generate_response_with_incomplete_json() {
    // Test partial response structure
    let json_data = r#"{"response": "Partial"}"#;
    let response: Result<GenerateResponse, _> = serde_json::from_str(json_data);

    // Should fail due to missing 'done' field
    assert!(response.is_err());
}

#[test]
fn test_empty_strings_handling() {
    let request = GenerateRequest {
        model: "".to_string(),
        prompt: "".to_string(),
        stream: false,
    };

    // Should not panic with empty strings
    assert_eq!(request.model, "");
    assert_eq!(request.prompt, "");

    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("\"model\":\"\""));
    assert!(json.contains("\"prompt\":\"\""));
}

#[test]
fn test_url_variations() {
    let urls = vec![
        "http://localhost:11434",
        "https://remote.server:8080",
        "http://127.0.0.1:11434",
        "https://ollama.example.com",
    ];

    for url in urls {
        let client = OllamaClient::new(Some(url.to_string()));
        let debug_str = format!("{:?}", client);
        assert!(debug_str.contains("OllamaClient"));
    }
}
