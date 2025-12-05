use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

pub fn create_temp_dir() -> TempDir {
    TempDir::new().expect("Failed to create temp directory")
}

pub fn create_test_manifest(path: &PathBuf, context_count: usize) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        let templates_dir = parent.join("templates");
        fs::create_dir_all(&templates_dir)?;
    }

    let mut contexts_yaml = String::new();
    for i in 0..context_count {
        contexts_yaml.push_str(&format!(
            r#"
  - name: Context{}
    aggregates:
      - name: Aggregate{}
        valueObjects:
          - name: Id{}
            fields:
              - name: value
                type: Guid
                nullable: false
"#,
            i, i, i
        ));
    }

    let manifest_content = format!(
        r#"apiVersion: ntk/v1
kind: solution
meta:
  name: async-test-manifest
solution:
  root: ./
  slnFile: AsyncTest.sln
conventions:
  namespaceRoot: AsyncApp
  targetFramework: net9.0
  policy:
    collision: fail
    insertTodoWhenMissing: true
    strict: false
apply:
  mode: feature
  feature:
    include: []
contexts:{}
"#,
        contexts_yaml
    );

    fs::write(path, manifest_content)
}