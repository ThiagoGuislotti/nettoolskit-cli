/// Tests for ExecutionConfig and ManifestExecutor
use nettoolskit_manifest::{ExecutionConfig, ManifestExecutor};
use std::path::PathBuf;

// ExecutionConfig Tests

#[test]
fn test_execution_config_creation() {
    // Arrange
    let manifest_path = PathBuf::from("test-manifest.yml");
    let output_root = PathBuf::from("output");

    // Act
    let config = ExecutionConfig {
        manifest_path: manifest_path.clone(),
        output_root: output_root.clone(),
        dry_run: false,
    };

    // Assert
    assert_eq!(config.manifest_path, manifest_path);
    assert_eq!(config.output_root, output_root);
    assert!(!config.dry_run);
}

#[test]
fn test_execution_config_clone() {
    // Arrange
    let config = ExecutionConfig {
        manifest_path: PathBuf::from("manifest.yml"),
        output_root: PathBuf::from("target"),
        dry_run: true,
    };

    // Act
    let cloned = config.clone();

    // Assert
    assert_eq!(cloned.manifest_path, config.manifest_path);
    assert_eq!(cloned.output_root, config.output_root);
    assert_eq!(cloned.dry_run, config.dry_run);
}

#[test]
fn test_execution_config_debug() {
    // Arrange
    let config = ExecutionConfig {
        manifest_path: PathBuf::from("test.yml"),
        output_root: PathBuf::from("out"),
        dry_run: false,
    };

    // Act
    let debug_str = format!("{:?}", config);

    // Assert
    assert!(debug_str.contains("test.yml"));
    assert!(debug_str.contains("out"));
    assert!(debug_str.contains("false"));
}

#[test]
fn test_execution_config_dry_run_flag() {
    // Arrange
    let config_dry = ExecutionConfig {
        manifest_path: PathBuf::from("manifest.yml"),
        output_root: PathBuf::from("output"),
        dry_run: true,
    };
    let config_normal = ExecutionConfig {
        manifest_path: PathBuf::from("manifest.yml"),
        output_root: PathBuf::from("output"),
        dry_run: false,
    };

    // Assert
    assert!(config_dry.dry_run);
    assert!(!config_normal.dry_run);
}

#[test]
fn test_execution_config_paths() {
    // Arrange & Act
    let config = ExecutionConfig {
        manifest_path: PathBuf::from("manifests/domain.yml"),
        output_root: PathBuf::from("target/generated"),
        dry_run: false,
    };

    // Assert
    assert!(config.manifest_path.to_str().unwrap().contains("domain.yml"));
    assert!(config.output_root.to_str().unwrap().contains("generated"));
}

// ManifestExecutor Tests

#[test]
fn test_manifest_executor_creation() {
    // Act
    let executor = ManifestExecutor::new();

    // Assert
    let _ = executor;
}

#[tokio::test]
async fn test_executor_missing_manifest_file() {
    // Arrange
    let executor = ManifestExecutor::new();
    let config = ExecutionConfig {
        manifest_path: PathBuf::from("nonexistent-file-12345.yml"),
        output_root: PathBuf::from("output"),
        dry_run: false,
    };

    // Act
    let result = executor.execute(config).await;

    // Assert
    assert!(result.is_err());
}

#[test]
fn test_multiple_executors_independent() {
    // Act
    let executor1 = ManifestExecutor::new();
    let executor2 = ManifestExecutor::new();

    // Assert
    let _ = (executor1, executor2);
}