# nettoolskit-manifest

> Manifest parsing and application for NetToolsKit CLI.

---

## Introduction

nettoolskit-manifest is a manifest-driven engine that parses `ntk/v1` YAML manifests and applies them to a target output root by rendering templates and producing a change plan.

---

## Features

-   ✅ Parse and validate `ntk/v1` manifest YAML files
-   ✅ Execute manifests with a dry-run option
-   ✅ Render templates and apply file changes with collision policies
-   ✅ Provide a summarized execution report

---

## Contents

- [Introduction](#introduction)
- [Features](#features)
- [Contents](#contents)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Usage Examples](#usage-examples)
  - [Example 1: Parse and validate a manifest](#example-1-parse-and-validate-a-manifest)
  - [Example 2: Execute apply (programmatic)](#example-2-execute-apply-programmatic)
- [API Reference](#api-reference)
  - [Parsing](#parsing)
  - [Execution](#execution)
  - [Handlers](#handlers)
- [References](#references)
- [License](#license)

---

## Installation

Add as a workspace/path dependency:

```toml
[dependencies]
nettoolskit-manifest = { path = "../commands/manifest" }
```

---

## Quick Start

Minimal usage in 3–5 lines:

```rust
use nettoolskit_manifest::{ExecutionConfig, ManifestExecutor};
use std::path::PathBuf;

# #[tokio::main]
# async fn main() -> Result<(), Box<dyn std::error::Error>> {
let executor = ManifestExecutor::new();
let summary = executor
    .execute(ExecutionConfig {
        manifest_path: PathBuf::from("ntk-manifest.yml"),
        output_root: PathBuf::from("."),
        dry_run: true,
    })
    .await?;

println!("Created: {}", summary.created.len());
# Ok(())
# }
```

---

## Usage Examples

### Example 1: Parse and validate a manifest

```rust
use nettoolskit_manifest::parsing::ManifestParser;
use std::path::Path;

let manifest = ManifestParser::from_file(Path::new("ntk-manifest.yml"))?;
ManifestParser::validate(&manifest)?;
```

### Example 2: Execute apply (programmatic)

```rust
use nettoolskit_manifest::{ExecutionConfig, ManifestExecutor};
use std::path::PathBuf;

# #[tokio::main]
# async fn main() -> Result<(), Box<dyn std::error::Error>> {
let executor = ManifestExecutor::new();
let summary = executor
    .execute(ExecutionConfig {
        manifest_path: PathBuf::from("ntk-manifest.yml"),
        output_root: PathBuf::from("target/out"),
        dry_run: false,
    })
    .await?;

println!("Updated: {}", summary.updated.len());
# Ok(())
# }
```

---

## API Reference

### Parsing

```rust
pub struct ManifestParser;

impl ManifestParser {
    pub fn from_file(path: &std::path::Path) -> ManifestResult<ManifestDocument>;
    pub fn validate(manifest: &ManifestDocument) -> ManifestResult<()>;
}
```

### Execution

```rust
#[derive(Debug, Clone)]
pub struct ExecutionConfig {
    pub manifest_path: std::path::PathBuf,
    pub output_root: std::path::PathBuf,
    pub dry_run: bool,
}

pub struct ManifestExecutor;

impl ManifestExecutor {
    pub fn new() -> Self;
    pub async fn execute(&self, config: ExecutionConfig) -> ManifestResult<ExecutionSummary>;
}
```

### Handlers

```rust
pub async fn execute_apply(
    manifest_path: std::path::PathBuf,
    output_root: Option<std::path::PathBuf>,
    dry_run: bool,
) -> nettoolskit_core::ExitStatus;
```

---

## References

- https://docs.rs/serde_yaml
- https://handlebarsjs.com/

---

## License

This project is licensed under the MIT License. See the LICENSE file at the repository root for details.

---
