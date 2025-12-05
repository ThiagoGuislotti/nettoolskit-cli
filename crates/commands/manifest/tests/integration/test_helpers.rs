//! Shared test helpers and fixtures for integration tests

use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

pub fn create_temp_dir() -> TempDir {
    TempDir::new().expect("Failed to create temp directory")
}

pub fn create_minimal_manifest(path: &PathBuf, namespace: &str) -> std::io::Result<()> {
    // Create templates directory next to manifest
    if let Some(parent) = path.parent() {
        let templates_dir = parent.join("templates");
        fs::create_dir_all(&templates_dir)?;
    }

    let manifest_content = format!(
        r#"apiVersion: ntk/v1
kind: solution
meta:
  name: test-manifest
solution:
  root: ./
  slnFile: TestSolution.sln
conventions:
  namespaceRoot: {}
  targetFramework: net9.0
  policy:
    collision: fail
    insertTodoWhenMissing: true
    strict: false
apply:
  mode: feature
  feature:
    context: Orders
    include: []
contexts:
  - name: Orders
    aggregates:
      - name: Order
        valueObjects:
          - name: OrderId
            fields:
              - name: value
                type: Guid
                nullable: false
        entities:
          - name: OrderItem
            fields:
              - name: quantity
                type: int
                nullable: false
                key: false
                columnName: null
            isRoot: false
    useCases:
      - name: CreateOrder
        type: command
        input:
          - name: customerId
            type: Guid
            nullable: false
            key: false
            columnName: null
        output:
          - name: orderId
            type: Guid
            nullable: false
            key: false
            columnName: null
"#,
        namespace
    );

    fs::write(path, manifest_content)
}