//! Tests for configuration types and utilities
//!
//! Validates the Config struct including default values,
//! serialization, deserialization, and field access.

use nettoolskit_core::config::Config;

// Happy Path Tests - Default Construction

#[test]
fn test_config_default_has_correct_name() {
    // Arrange
    let expected_name = "NetToolsKit CLI";

    // Act
    let config = Config::default();

    // Assert
    assert_eq!(config.name, expected_name);
}

#[test]
fn test_config_default_has_version() {
    // Arrange
    let expected_version = env!("CARGO_PKG_VERSION");

    // Act
    let config = Config::default();

    // Assert
    assert!(!config.version.is_empty());
    assert_eq!(config.version, expected_version);
}

#[test]
fn test_config_default_creates_valid_instance() {
    // Act
    let config = Config::default();

    // Assert
    assert_eq!(config.name, "NetToolsKit CLI");
    assert!(!config.version.is_empty());
}

// Happy Path Tests - Field Access

#[test]
fn test_config_name_field_access() {
    // Arrange
    let config = Config {
        name: "Test Application".to_string(),
        version: "1.0.0".to_string(),
    };

    // Act
    let name = &config.name;

    // Assert
    assert_eq!(name, "Test Application");
}

#[test]
fn test_config_version_field_access() {
    // Arrange
    let config = Config {
        name: "Test".to_string(),
        version: "2.0.0".to_string(),
    };

    // Act
    let version = &config.version;

    // Assert
    assert_eq!(version, "2.0.0");
}

#[test]
fn test_config_can_be_created_with_custom_values() {
    // Arrange
    let custom_name = "Custom CLI".to_string();
    let custom_version = "0.1.0".to_string();

    // Act
    let config = Config {
        name: custom_name.clone(),
        version: custom_version.clone(),
    };

    // Assert
    assert_eq!(config.name, custom_name);
    assert_eq!(config.version, custom_version);
}

// Happy Path Tests - JSON Serialization

#[test]
fn test_config_serializes_to_json() {
    // Arrange
    let config = Config {
        name: "Test".to_string(),
        version: "1.0.0".to_string(),
    };

    // Act
    let json = serde_json::to_string(&config).unwrap();

    // Assert
    assert!(json.contains("Test"));
    assert!(json.contains("1.0.0"));
}

#[test]
fn test_config_deserializes_from_json() {
    // Arrange
    let json = r#"{"name":"Test","version":"1.0.0"}"#;

    // Act
    let config: Config = serde_json::from_str(json).unwrap();

    // Assert
    assert_eq!(config.name, "Test");
    assert_eq!(config.version, "1.0.0");
}

#[test]
fn test_config_roundtrip_serialization() {
    // Arrange
    let original = Config {
        name: "Original".to_string(),
        version: "2.0.0".to_string(),
    };

    // Act
    let json = serde_json::to_string(&original).unwrap();
    let deserialized: Config = serde_json::from_str(&json).unwrap();

    // Assert
    assert_eq!(original.name, deserialized.name);
    assert_eq!(original.version, deserialized.version);
}

// Happy Path Tests - Trait Implementations

#[test]
fn test_config_debug_format() {
    // Arrange
    let config = Config {
        name: "Debug Test".to_string(),
        version: "1.0.0".to_string(),
    };

    // Act
    let debug_str = format!("{:?}", config);

    // Assert
    assert!(debug_str.contains("Config"));
    assert!(debug_str.contains("Debug Test"));
    assert!(debug_str.contains("1.0.0"));
}

#[test]
fn test_config_clone() {
    // Arrange
    let config = Config {
        name: "Clone Test".to_string(),
        version: "1.0.0".to_string(),
    };

    // Act
    let cloned = config.clone();

    // Assert
    assert_eq!(config.name, cloned.name);
    assert_eq!(config.version, cloned.version);
}

#[test]
fn test_config_clone_independence() {
    // Arrange
    let mut config = Config {
        name: "Original".to_string(),
        version: "1.0.0".to_string(),
    };

    // Act
    let cloned = config.clone();
    config.name = "Modified".to_string();

    // Assert
    assert_eq!(cloned.name, "Original");
    assert_eq!(config.name, "Modified");
}

// Edge Cases

#[test]
fn test_config_with_empty_name() {
    // Arrange
    // (empty name is valid)

    // Act
    let config = Config {
        name: String::new(),
        version: "1.0.0".to_string(),
    };

    // Assert
    assert!(config.name.is_empty());
}

#[test]
fn test_config_with_empty_version() {
    // Arrange
    // (empty version is valid)

    // Act
    let config = Config {
        name: "Test".to_string(),
        version: String::new(),
    };

    // Assert
    assert!(config.version.is_empty());
}

#[test]
fn test_config_with_unicode_name() {
    // Arrange
    let unicode_name = "NetToolsKit 🚀 CLI".to_string();

    // Act
    let config = Config {
        name: unicode_name.clone(),
        version: "1.0.0".to_string(),
    };

    // Assert
    assert_eq!(config.name, unicode_name);
}

#[test]
fn test_config_with_long_name() {
    // Arrange
    let long_name = "A".repeat(1000);

    // Act
    let config = Config {
        name: long_name.clone(),
        version: "1.0.0".to_string(),
    };

    // Assert
    assert_eq!(config.name.len(), 1000);
    assert_eq!(config.name, long_name);
}

#[test]
fn test_config_with_semantic_version() {
    // Arrange
    let semver = "1.2.3-alpha+build.123".to_string();

    // Act
    let config = Config {
        name: "Test".to_string(),
        version: semver.clone(),
    };

    // Assert
    assert_eq!(config.version, semver);
}

#[test]
fn test_config_with_special_characters_in_name() {
    // Arrange
    let special_name = "Net-Tools_Kit.CLI".to_string();

    // Act
    let config = Config {
        name: special_name.clone(),
        version: "1.0.0".to_string(),
    };

    // Assert
    assert_eq!(config.name, special_name);
}

#[test]
fn test_config_json_with_extra_fields_ignored() {
    // Arrange
    let json = r#"{"name":"Test","version":"1.0.0","extra":"ignored"}"#;

    // Act
    let result: Result<Config, _> = serde_json::from_str(json);

    // Assert
    assert!(result.is_ok());
}

#[test]
fn test_config_json_with_missing_field_errors() {
    // Arrange
    let json = r#"{"name":"Test"}"#; // Missing version

    // Act
    let result: Result<Config, _> = serde_json::from_str(json);

    // Assert
    assert!(result.is_err());
}
