/// Tests for ExecutionConfig and ManifestExecutor
use nettoolskit_manifest::{ExecutionConfig, ManifestExecutor};
use std::path::PathBuf;

#[test]
fn test_execution_config_creation() {
    let config = ExecutionConfig {
        manifest_path: PathBuf::from("test-manifest.yml"),
        output_root: PathBuf::from("output"),
        dry_run: false,
    };

    assert_eq!(config.manifest_path, PathBuf::from("test-manifest.yml"));
    assert_eq!(config.output_root, PathBuf::from("output"));
    assert!(!config.dry_run);
}

#[test]
fn test_execution_config_clone() {
    let config = ExecutionConfig {
        manifest_path: PathBuf::from("manifest.yml"),
        output_root: PathBuf::from("target"),
        dry_run: true,
    };

    let cloned = config.clone();
    assert_eq!(cloned.manifest_path, config.manifest_path);
    assert_eq!(cloned.output_root, config.output_root);
    assert_eq!(cloned.dry_run, config.dry_run);
}

#[test]
fn test_execution_config_debug() {
    let config = ExecutionConfig {
        manifest_path: PathBuf::from("test.yml"),
        output_root: PathBuf::from("out"),
        dry_run: false,
    };

    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("test.yml"));
    assert!(debug_str.contains("out"));
    assert!(debug_str.contains("false"));
}

#[test]
fn test_manifest_executor_creation() {
    let executor = ManifestExecutor::new();

    // Should be able to create executor
    let _ = executor;
}

#[test]
fn test_execution_config_dry_run_flag() {
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

    assert!(config_dry.dry_run);
    assert!(!config_normal.dry_run);
}

#[test]
fn test_execution_config_paths() {
    let config = ExecutionConfig {
        manifest_path: PathBuf::from("manifests/domain.yml"),
        output_root: PathBuf::from("target/generated"),
        dry_run: false,
    };

    assert!(config.manifest_path.to_str().unwrap().contains("domain.yml"));
    assert!(config.output_root.to_str().unwrap().contains("generated"));
}

#[tokio::test]
async fn test_executor_missing_manifest_file() {
    let executor = ManifestExecutor::new();
    let config = ExecutionConfig {
        manifest_path: PathBuf::from("nonexistent-file-12345.yml"),
        output_root: PathBuf::from("output"),
        dry_run: false,
    };

    let result = executor.execute(config).await;
    assert!(result.is_err());
}

#[test]
fn test_multiple_executors_independent() {
    let executor1 = ManifestExecutor::new();
    let executor2 = ManifestExecutor::new();

    // Each executor should be independent
    let _ = (executor1, executor2);
}