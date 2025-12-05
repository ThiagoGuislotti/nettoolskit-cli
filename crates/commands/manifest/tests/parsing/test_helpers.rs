use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

pub fn create_temp_manifest(content: &str) -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("test-manifest.yml");
    fs::write(&manifest_path, content).unwrap();
    (temp_dir, manifest_path)
}