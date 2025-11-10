use nettoolskit_ollama::{GenerateRequest, ModelConfig, ModelType, OllamaClient};

#[tokio::test]
async fn test_integration_client_with_model_config() {
    let _client = OllamaClient::default();
    let config = ModelConfig::code_generation();

    // Create request using model configuration
    let request = GenerateRequest {
        model: config.name.clone(),
        prompt: "Generate a simple function".to_string(),
        stream: false,
    };

    assert_eq!(request.model, "codellama:latest");
    assert_eq!(request.prompt, "Generate a simple function");
}

#[test]
fn test_integration_all_model_types_with_client() {
    let _client = OllamaClient::new(Some("http://test:8080".to_string()));

    for model_type in ModelType::all() {
        let config = model_type.config();

        let request = GenerateRequest {
            model: config.name.clone(),
            prompt: format!("Test prompt for {:?}", model_type),
            stream: false,
        };

        // Verify the request is properly constructed
        assert!(!request.model.is_empty());
        assert!(request.prompt.contains("Test prompt"));

        // Verify serialization works
        let json = serde_json::to_string(&request);
        assert!(json.is_ok());
    }
}

#[test]
fn test_integration_model_config_parameters() {
    let all_configs = vec![
        (ModelType::CodeGeneration, ModelConfig::code_generation()),
        (ModelType::Chat, ModelConfig::chat()),
        (ModelType::Documentation, ModelConfig::documentation()),
    ];

    for (model_type, config) in all_configs {
        // Verify configuration consistency
        assert!(!config.name.is_empty());
        assert!(config.temperature >= 0.0 && config.temperature <= 1.0);

        if let Some(max_tokens) = config.max_tokens {
            assert!(max_tokens > 0);
        }

        // Verify model type mapping works
        let mapped_config = model_type.config();
        assert_eq!(config.name, mapped_config.name);
        assert_eq!(config.temperature, mapped_config.temperature);
        assert_eq!(config.max_tokens, mapped_config.max_tokens);
    }
}

#[test]
fn test_integration_client_url_handling() {
    let test_cases = vec![
        (None, "localhost:11434"),
        (Some("http://custom:9999".to_string()), "custom:9999"),
        (
            Some("https://secure.example.com".to_string()),
            "secure.example.com",
        ),
    ];

    for (url_input, expected_content) in test_cases {
        let client = OllamaClient::new(url_input);
        let debug_str = format!("{:?}", client);

        assert!(debug_str.contains(expected_content));
        assert!(debug_str.contains("OllamaClient"));
    }
}

#[test]
fn test_integration_request_response_cycle() {
    // Test the complete request-response data flow
    let config = ModelConfig::chat();

    let request = GenerateRequest {
        model: config.name,
        prompt: "Hello, how are you?".to_string(),
        stream: false,
    };

    // Serialize request
    let request_json = serde_json::to_string(&request).unwrap();
    assert!(request_json.contains("llama2:latest"));
    assert!(request_json.contains("Hello, how are you?"));

    // Simulate response
    let response_json = r#"{"response": "I'm doing well, thank you!", "done": true}"#;
    let response = serde_json::from_str(response_json).unwrap();

    match response {
        serde_json::Value::Object(obj) => {
            assert!(obj.contains_key("response"));
            assert!(obj.contains_key("done"));
        }
        _ => panic!("Expected JSON object"),
    }
}

#[test]
fn test_integration_error_handling_scenarios() {
    // Test various error scenarios that might occur in integration

    // Invalid JSON response
    let invalid_json = r#"{"response": "test""#; // Missing closing brace
    let result: Result<serde_json::Value, _> = serde_json::from_str(invalid_json);
    assert!(result.is_err());

    // Missing required fields
    let incomplete_json = r#"{"response": "test"}"#; // Missing 'done' field
    let result: Result<nettoolskit_ollama::GenerateResponse, _> =
        serde_json::from_str(incomplete_json);
    assert!(result.is_err());

    // Empty response handling
    let empty_response = r#"{"response": "", "done": true}"#;
    let result: Result<nettoolskit_ollama::GenerateResponse, _> =
        serde_json::from_str(empty_response);
    assert!(result.is_ok());

    if let Ok(response) = result {
        assert_eq!(response.response, "");
        assert!(response.done);
    }
}

#[test]
fn test_integration_concurrent_client_usage() {
    // Test that multiple clients can be created without conflicts
    let clients: Vec<OllamaClient> = (0..5)
        .map(|i| OllamaClient::new(Some(format!("http://server{}:808{}", i, i))))
        .collect();

    assert_eq!(clients.len(), 5);

    for (i, client) in clients.iter().enumerate() {
        let debug_str = format!("{:?}", client);
        assert!(debug_str.contains(&format!("server{}", i)));
    }
}

#[test]
fn test_integration_model_type_completeness() {
    // Ensure all model types are properly integrated
    let all_types = ModelType::all();

    // Verify we have expected number of types
    assert_eq!(all_types.len(), 3);

    // Verify each type can generate a valid config
    for model_type in all_types {
        let config = model_type.config();

        // Each config should be usable with client
        let request = GenerateRequest {
            model: config.name,
            prompt: "Integration test".to_string(),
            stream: false,
        };

        // Should serialize without errors
        let json = serde_json::to_string(&request);
        assert!(json.is_ok());
    }
}
