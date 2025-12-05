use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

pub fn create_test_template() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let template_path = temp_dir.path().join("test-template.cs.hbs");

    fs::write(
        &template_path,
        "namespace {{namespace}} {\n    public class {{className}} {}\n}",
    )
    .expect("Failed to write template");

    (temp_dir, template_path)
}