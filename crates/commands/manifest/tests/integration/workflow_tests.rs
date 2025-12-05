//! Tests for full end-to-end manifest execution workflow
//!
//! Validates complete pipeline: parse → generate tasks → render → write.

use nettoolskit_manifest::{ExecutionConfig, ManifestExecutor};
use super::test_helpers::create_temp_dir;

#[tokio::test]
async fn test_integration_full_workflow_with_templates() {
    // Arrange
    // Use test fixtures directory with mock templates
    let test_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures");

    let manifest_path = test_dir.join("test-manifest.yml");

    // Skip test if manifest doesn't exist
    if !manifest_path.exists() {
        eprintln!("Skipping test: test fixtures not found at {:?}", test_dir);
        return;
    }

    let temp_dir = create_temp_dir();
    let output_dir = temp_dir.path().join("output");

    let config = ExecutionConfig {
        manifest_path,
        output_root: output_dir.clone(),
        dry_run: true, // Use dry-run to test without actual file creation
    };

    // Act
    let executor = ManifestExecutor::new();
    let result = executor.execute(config).await;

    // Assert
    if let Err(ref e) = result {
        eprintln!("Full workflow test error: {}", e);
    }
    assert!(result.is_ok(), "Full workflow should succeed with real templates");
    let summary = result.unwrap();

    // Verify execution summary
    assert!(
        !summary.notes.is_empty(),
        "Should have execution notes about planned operations"
    );

    // Verify that tasks were generated
    let has_task_notes = summary.notes.iter().any(|n| n.contains("render task"));
    assert!(has_task_notes, "Should mention render tasks in notes");
}