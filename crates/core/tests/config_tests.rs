//! Tests for configuration types and utilities
//!
//! This test file validates the Config struct, including
//! default values, serialization, and field access.

use nettoolskit_core::config::Config;

// =============================================================================
// Constructor and Default Tests
// =============================================================================

#[test]
fn test_config_default_has_correct_name() {
    let config = Config::default();
    assert_eq!(config.name, "NetToolsKit CLI");
}

#[test]
fn test_config_default_has_version() {
    let config = Config::default();
    assert!(!config.version.is_empty());
    // Version should match CARGO_PKG_VERSION
    assert_eq!(config.version, env!("CARGO_PKG_VERSION"));
}

#[test]
fn test_config_default_creates_valid_instance() {
    let config = Config::default();
    assert_eq!(config.name, "NetToolsKit CLI");
    assert!(!config.version.is_empty());
}

// =============================================================================
// Field Access Tests
// =============================================================================

#[test]
fn test_config_name_field_access() {
    let config = Config {
        name: "Test Application".to_string(),
        version: "1.0.0".to_string(),
    };
    assert_eq!(config.name, "Test Application");
}

#[test]
fn test_config_version_field_access() {
    let config = Config {
        name: "Test".to_string(),
        version: "2.0.0".to_string(),
    };
    assert_eq!(config.version, "2.0.0");
}

#[test]
fn test_config_can_be_created_with_custom_values() {
    let config = Config {
        name: "Custom CLI".to_string(),
        version: "0.1.0".to_string(),
    };
    assert_eq!(config.name, "Custom CLI");
    assert_eq!(config.version, "0.1.0");
}

// =============================================================================
// Serialization Tests
// =============================================================================

#[test]
fn test_config_serializes_to_json() {
    let config = Config {
        name: "Test".to_string(),
        version: "1.0.0".to_string(),
    };
    let json = serde_json::to_string(&config).unwrap();
    assert!(json.contains("Test"));
    assert!(json.contains("1.0.0"));
}

#[test]
fn test_config_deserializes_from_json() {
    let json = r#"{"name":"Test","version":"1.0.0"}"#;
    let config: Config = serde_json::from_str(json).unwrap();
    assert_eq!(config.name, "Test");
    assert_eq!(config.version, "1.0.0");
}

#[test]
fn test_config_roundtrip_serialization() {
    let original = Config {
        name: "Original".to_string(),
        version: "2.0.0".to_string(),
    };
    let json = serde_json::to_string(&original).unwrap();
    let deserialized: Config = serde_json::from_str(&json).unwrap();
    assert_eq!(original.name, deserialized.name);
    assert_eq!(original.version, deserialized.version);
}

// =============================================================================
// Trait Implementation Tests
// =============================================================================

#[test]
fn test_config_debug_format() {
    let config = Config {
        name: "Debug Test".to_string(),
        version: "1.0.0".to_string(),
    };
    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("Config"));
    assert!(debug_str.contains("Debug Test"));
    assert!(debug_str.contains("1.0.0"));
}

#[test]
fn test_config_clone() {
    let config = Config {
        name: "Clone Test".to_string(),
        version: "1.0.0".to_string(),
    };
    let cloned = config.clone();
    assert_eq!(config.name, cloned.name);
    assert_eq!(config.version, cloned.version);
}

#[test]
fn test_config_clone_independence() {
    let mut config = Config {
        name: "Original".to_string(),
        version: "1.0.0".to_string(),
    };
    let cloned = config.clone();
    config.name = "Modified".to_string();
    assert_eq!(cloned.name, "Original");
    assert_eq!(config.name, "Modified");
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn test_config_with_empty_name() {
    let config = Config {
        name: String::new(),
        version: "1.0.0".to_string(),
    };
    assert!(config.name.is_empty());
}

#[test]
fn test_config_with_empty_version() {
    let config = Config {
        name: "Test".to_string(),
        version: String::new(),
    };
    assert!(config.version.is_empty());
}

#[test]
fn test_config_with_unicode_name() {
    let config = Config {
        name: "NetToolsKit 🚀 CLI".to_string(),
        version: "1.0.0".to_string(),
    };
    assert_eq!(config.name, "NetToolsKit 🚀 CLI");
}

#[test]
fn test_config_with_long_name() {
    let long_name = "A".repeat(1000);
    let config = Config {
        name: long_name.clone(),
        version: "1.0.0".to_string(),
    };
    assert_eq!(config.name.len(), 1000);
    assert_eq!(config.name, long_name);
}

#[test]
fn test_config_with_semantic_version() {
    let config = Config {
        name: "Test".to_string(),
        version: "1.2.3-alpha+build.123".to_string(),
    };
    assert_eq!(config.version, "1.2.3-alpha+build.123");
}

#[test]
fn test_config_with_special_characters_in_name() {
    let config = Config {
        name: "Net-Tools_Kit.CLI".to_string(),
        version: "1.0.0".to_string(),
    };
    assert_eq!(config.name, "Net-Tools_Kit.CLI");
}

#[test]
fn test_config_json_with_extra_fields_ignored() {
    let json = r#"{"name":"Test","version":"1.0.0","extra":"ignored"}"#;
    let result: Result<Config, _> = serde_json::from_str(json);
    assert!(result.is_ok());
}

#[test]
fn test_config_json_with_missing_field_errors() {
    let json = r#"{"name":"Test"}"#; // Missing version
    let result: Result<Config, _> = serde_json::from_str(json);
    assert!(result.is_err());
}
