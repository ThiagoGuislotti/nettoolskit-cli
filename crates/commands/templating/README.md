# nettoolskit-templating

> High-performance async template rendering for NetToolsKit CLI.

---

## Introduction

nettoolskit-templating provides async-first template resolution and rendering primitives built around Handlebars, plus utilities for language-specific conventions and parallel batch generation.

---

## Features

-   ✅ Async template resolution with caching
-   ✅ Async rendering with compiled-template caching
-   ✅ Parallel batch rendering with bounded concurrency
-   ✅ Strategy-based language conventions and path normalization

---

## Contents

- [Introduction](#introduction)
- [Features](#features)
- [Contents](#contents)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Usage Examples](#usage-examples)
  - [Example 1: Resolve and render a template](#example-1-resolve-and-render-a-template)
  - [Example 2: Batch rendering](#example-2-batch-rendering)
- [API Reference](#api-reference)
  - [TemplateEngine](#templateengine)
  - [TemplateResolver](#templateresolver)
  - [BatchRenderer](#batchrenderer)
- [References](#references)
- [License](#license)

---

## Installation

Add as a workspace/path dependency:

```toml
[dependencies]
nettoolskit-templating = { path = "../commands/templating" }
```

---

## Quick Start

Minimal usage in 3–5 lines:

```rust
use nettoolskit_templating::TemplateEngine;
use serde_json::json;
use std::path::Path;

# #[tokio::main]
# async fn main() -> Result<(), Box<dyn std::error::Error>> {
let engine = TemplateEngine::new();
let rendered = engine
    .render_from_file(Path::new("templates/hello.hbs"), &json!({"name":"World"}))
    .await?;

println!("{rendered}");
# Ok(())
# }
```

---

## Usage Examples

### Example 1: Resolve and render a template

```rust
use nettoolskit_templating::{TemplateEngine, TemplateResolver};
use serde_json::json;

# #[tokio::main]
# async fn main() -> Result<(), Box<dyn std::error::Error>> {
let resolver = TemplateResolver::new("templates");
let template_path = resolver.resolve("dotnet/Domain/Entity.cs.hbs").await?;

let engine = TemplateEngine::new();
let content = engine.render_from_file(&template_path, &json!({"name": "User"})).await?;

println!("{content}");
# Ok(())
# }
```

### Example 2: Batch rendering

```rust
use nettoolskit_templating::{BatchRenderer, RenderRequest};
use serde_json::json;
use std::path::PathBuf;

# #[tokio::main]
# async fn main() -> Result<(), Box<dyn std::error::Error>> {
let renderer = BatchRenderer::new("templates");

let result = renderer
    .render_batch(vec![RenderRequest {
        template: "dotnet/Domain/Entity.cs.hbs".to_string(),
        data: json!({"name":"User"}),
        output: PathBuf::from("target/out/User.cs"),
    }])
    .await?;

println!("Succeeded: {}", result.succeeded);
# Ok(())
# }
```

---

## API Reference

### TemplateEngine

```rust
pub struct TemplateEngine;

impl TemplateEngine {
    pub fn new() -> Self;

    pub async fn render_from_file<P: AsRef<std::path::Path>, T: serde::Serialize>(
        &self,
        template_path: P,
        data: &T,
    ) -> TemplateResult<String>;

    pub async fn render_from_string<T: serde::Serialize>(
        &self,
        template_source: &str,
        data: &T,
        template_name: String,
    ) -> TemplateResult<String>;
}
```

### TemplateResolver

```rust
pub struct TemplateResolver;

impl TemplateResolver {
    pub fn new<P: AsRef<std::path::Path>>(templates_root: P) -> Self;
    pub async fn resolve(&self, template: &str) -> TemplateResult<std::path::PathBuf>;
}
```

### BatchRenderer

```rust
#[derive(Debug, Clone)]
pub struct RenderRequest<T> {
    pub template: String,
    pub data: T,
    pub output: std::path::PathBuf,
}

#[derive(Debug)]
pub struct BatchRenderResult {
    pub succeeded: usize,
    pub failed: usize,
    pub errors: Vec<(String, TemplateError)>,
    pub duration: std::time::Duration,
}

pub struct BatchRenderer;

impl BatchRenderer {
    pub fn new<P: AsRef<std::path::Path>>(templates_root: P) -> Self;

    pub async fn render_batch<T>(
        &self,
        requests: Vec<RenderRequest<T>>,
    ) -> TemplateResult<BatchRenderResult>
    where
        T: serde::Serialize + Send + Sync + 'static;
}
```

---

## References

- https://docs.rs/handlebars
- https://docs.rs/tokio

---

## License

This project is licensed under the MIT License. See the LICENSE file at the repository root for details.

---
