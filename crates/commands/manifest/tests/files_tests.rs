/// Tests for file operations module (files/executor.rs)
///
/// Validates file system operations including directory creation,
/// file writing, dry-run mode, and execution summary tracking.
use nettoolskit_manifest::models::{ExecutionSummary, FileChange, FileChangeKind};
use std::fs;
use tempfile::TempDir;

// Test Helpers

fn create_temp_dir() -> TempDir {
    TempDir::new().expect("Failed to create temp directory")
}

// ensure_directory Tests

#[test]
fn test_ensure_directory_creates_new_directory() {
    // Arrange
    let temp_dir = create_temp_dir();
    let new_dir = temp_dir.path().join("test_dir");
    let mut summary = ExecutionSummary::default();

    // Act
    let result = nettoolskit_manifest::files::ensure_directory(
        &new_dir,
        false, // not dry-run
        &mut summary,
        "test directory",
    );

    // Assert
    assert!(result.is_ok());
    assert!(new_dir.exists());
    assert!(new_dir.is_dir());
}

#[test]
fn test_ensure_directory_dry_run_does_not_create() {
    // Arrange
    let temp_dir = create_temp_dir();
    let new_dir = temp_dir.path().join("dry_run_dir");
    let mut summary = ExecutionSummary::default();

    // Act
    let result = nettoolskit_manifest::files::ensure_directory(
        &new_dir,
        true, // dry-run
        &mut summary,
        "test directory",
    );

    // Assert
    assert!(result.is_ok());
    assert!(!new_dir.exists(), "Dry-run should NOT create directory");
    assert!(summary.notes.iter().any(|n| n.contains("dry-run")));
}

#[test]
fn test_ensure_directory_nested_paths() {
    // Arrange
    let temp_dir = create_temp_dir();
    let nested_dir = temp_dir.path().join("level1").join("level2").join("level3");
    let mut summary = ExecutionSummary::default();

    // Act
    let result = nettoolskit_manifest::files::ensure_directory(
        &nested_dir,
        false,
        &mut summary,
        "nested directory",
    );

    // Assert
    assert!(result.is_ok());
    assert!(nested_dir.exists());
    assert!(nested_dir.is_dir());
}

// execute_plan Tests

#[test]
fn test_execute_plan_creates_new_file() {
    // Arrange
    let temp_dir = create_temp_dir();
    let file_path = temp_dir.path().join("test.txt");
    let changes = vec![FileChange {
        path: file_path.clone(),
        content: "Hello, World!".to_string(),
        kind: FileChangeKind::Create,
        note: None,
    }];
    let mut summary = ExecutionSummary::default();

    // Act
    let result = nettoolskit_manifest::files::execute_plan(changes, false, &mut summary);

    // Assert
    assert!(result.is_ok());
    assert!(file_path.exists());
    assert_eq!(summary.created.len(), 1);
    let content = fs::read_to_string(&file_path).expect("Failed to read file");
    assert_eq!(content, "Hello, World!");
}

#[test]
fn test_execute_plan_updates_existing_file() {
    // Arrange
    let temp_dir = create_temp_dir();
    let file_path = temp_dir.path().join("existing.txt");
    fs::write(&file_path, "Original content").expect("Failed to create test file");

    let changes = vec![FileChange {
        path: file_path.clone(),
        content: "Updated content".to_string(),
        kind: FileChangeKind::Update,
        note: None,
    }];
    let mut summary = ExecutionSummary::default();

    // Act
    let result = nettoolskit_manifest::files::execute_plan(changes, false, &mut summary);

    // Assert
    assert!(result.is_ok());
    assert_eq!(summary.updated.len(), 1);
    let content = fs::read_to_string(&file_path).expect("Failed to read file");
    assert_eq!(content, "Updated content");
}

#[test]
fn test_execute_plan_dry_run_does_not_write() {
    // Arrange
    let temp_dir = create_temp_dir();
    let file_path = temp_dir.path().join("dry_run.txt");
    let changes = vec![FileChange {
        path: file_path.clone(),
        content: "Should not be written".to_string(),
        kind: FileChangeKind::Create,
        note: None,
    }];
    let mut summary = ExecutionSummary::default();

    // Act
    let result = nettoolskit_manifest::files::execute_plan(changes, true, &mut summary);

    // Assert
    assert!(result.is_ok());
    assert!(!file_path.exists(), "Dry-run should NOT create file");
    assert!(summary.notes.iter().any(|n| n.contains("would create")));
}

#[test]
fn test_execute_plan_creates_parent_directories() {
    // Arrange
    let temp_dir = create_temp_dir();
    let file_path = temp_dir.path().join("nested").join("path").join("file.txt");
    let changes = vec![FileChange {
        path: file_path.clone(),
        content: "Nested file content".to_string(),
        kind: FileChangeKind::Create,
        note: None,
    }];
    let mut summary = ExecutionSummary::default();

    // Act
    let result = nettoolskit_manifest::files::execute_plan(changes, false, &mut summary);

    // Assert
    assert!(result.is_ok());
    assert!(file_path.exists());
    assert!(file_path.parent().unwrap().exists());
}

#[test]
fn test_execute_plan_multiple_files() {
    // Arrange
    let temp_dir = create_temp_dir();
    let file1 = temp_dir.path().join("file1.txt");
    let file2 = temp_dir.path().join("file2.txt");

    let changes = vec![
        FileChange {
            path: file1.clone(),
            content: "Content 1".to_string(),
            kind: FileChangeKind::Create,
            note: None,
        },
        FileChange {
            path: file2.clone(),
            content: "Content 2".to_string(),
            kind: FileChangeKind::Create,
            note: None,
        },
    ];
    let mut summary = ExecutionSummary::default();

    // Act
    let result = nettoolskit_manifest::files::execute_plan(changes, false, &mut summary);

    // Assert
    assert!(result.is_ok());
    assert_eq!(summary.created.len(), 2);
    assert!(file1.exists());
    assert!(file2.exists());
}

#[test]
fn test_execute_plan_empty_changes() {
    // Arrange
    let changes: Vec<FileChange> = Vec::new();
    let mut summary = ExecutionSummary::default();

    // Act
    let result = nettoolskit_manifest::files::execute_plan(changes, false, &mut summary);

    // Assert
    assert!(result.is_ok());
    assert!(summary.created.is_empty());
    assert!(summary.updated.is_empty());
}

#[test]
fn test_execute_plan_mixed_create_and_update() {
    // Arrange
    let temp_dir = create_temp_dir();
    let new_file = temp_dir.path().join("new.txt");
    let existing_file = temp_dir.path().join("existing.txt");
    fs::write(&existing_file, "Old").expect("Failed to create test file");

    let changes = vec![
        FileChange {
            path: new_file.clone(),
            content: "New content".to_string(),
            kind: FileChangeKind::Create,
            note: None,
        },
        FileChange {
            path: existing_file.clone(),
            content: "Updated content".to_string(),
            kind: FileChangeKind::Update,
            note: None,
        },
    ];
    let mut summary = ExecutionSummary::default();

    // Act
    let result = nettoolskit_manifest::files::execute_plan(changes, false, &mut summary);

    // Assert
    assert!(result.is_ok());
    assert_eq!(summary.created.len(), 1);
    assert_eq!(summary.updated.len(), 1);
    assert!(new_file.exists());
    assert!(existing_file.exists());
}
