//! Tracing setup tests
//!
//! Tests for tracing configuration and initialization.

use nettoolskit_otel::tracing_setup::TracingConfig;

#[test]
fn test_tracing_config_default() {
    let config = TracingConfig::default();

    assert!(!config.verbose);
    assert!(!config.json_format);
    assert!(!config.with_file);
    assert!(config.with_line_numbers);
    assert_eq!(config.service_name, "nettoolskit-cli");
    assert!(!config.service_version.is_empty());
    assert!(!config.interactive_mode);
}

#[test]
fn test_tracing_config_custom() {
    let config = TracingConfig {
        verbose: true,
        json_format: true,
        with_file: true,
        with_line_numbers: false,
        service_name: "custom-service".to_string(),
        service_version: "1.0.0".to_string(),
        interactive_mode: true,
    };

    assert!(config.verbose);
    assert!(config.json_format);
    assert!(config.with_file);
    assert!(!config.with_line_numbers);
    assert_eq!(config.service_name, "custom-service");
    assert_eq!(config.service_version, "1.0.0");
    assert!(config.interactive_mode);
}

#[test]
fn test_tracing_config_clone() {
    let config1 = TracingConfig::default();
    let config2 = config1.clone();

    assert_eq!(config1.verbose, config2.verbose);
    assert_eq!(config1.json_format, config2.json_format);
    assert_eq!(config1.service_name, config2.service_name);
}
