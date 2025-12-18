# nettoolskit-help

> Help and workspace discovery commands for NetToolsKit CLI.

---

## Introduction

nettoolskit-help provides discovery-oriented helpers used by the NetToolsKit CLI, such as locating and listing manifest files inside a workspace.

---

## Features

-   ✅ Discover manifest files in a workspace folder
-   ✅ Parse manifests (best-effort) and summarize their metadata
-   ✅ Render a user-friendly manifest list to the terminal

---

## Contents

- [Introduction](#introduction)
- [Features](#features)
- [Contents](#contents)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Usage Examples](#usage-examples)
  - [Example 1: Discover manifests](#example-1-discover-manifests)
  - [Example 2: Discover and display](#example-2-discover-and-display)
- [API Reference](#api-reference)
  - [Handlers](#handlers)
  - [Models](#models)
- [References](#references)
- [License](#license)

---

## Installation

Add as a workspace/path dependency:

```toml
[dependencies]
nettoolskit-help = { path = "../commands/help" }
```

---

## Quick Start

Minimal usage in 3–5 lines:

```rust
use nettoolskit_help::discover_manifests;

# #[tokio::main]
# async fn main() {
let manifests = discover_manifests(None).await;
println!("Found {} manifest(s)", manifests.len());
# }
```

---

## Usage Examples

### Example 1: Discover manifests

```rust
use nettoolskit_help::discover_manifests;
use std::path::PathBuf;

# #[tokio::main]
# async fn main() {
let root = Some(PathBuf::from("."));
let manifests = discover_manifests(root).await;

for manifest in &manifests {
    println!("{} -> {}", manifest.project_name, manifest.path.display());
}
# }
```

### Example 2: Discover and display

```rust
use nettoolskit_help::{discover_manifests, display_manifests};

# #[tokio::main]
# async fn main() {
let manifests = discover_manifests(None).await;
display_manifests(&manifests);
# }
```

---

## API Reference

### Handlers

```rust
pub async fn discover_manifests(root: Option<std::path::PathBuf>) -> Vec<ManifestInfo>;

pub fn display_manifests(manifests: &[ManifestInfo]);
```

### Models

```rust
#[derive(Debug, Clone)]
pub struct ManifestInfo {
    pub path: std::path::PathBuf,
    pub project_name: String,
    pub language: String,
    pub context_count: usize,
}
```

---

## References

- https://docs.rs/tokio
- https://docs.rs/walkdir

---

## License

This project is licensed under the MIT License. See the LICENSE file at the repository root for details.

---
