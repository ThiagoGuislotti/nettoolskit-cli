# nettoolskit-translate

> Template translation commands for NetToolsKit CLI.

---

## Introduction

nettoolskit-translate provides the `/translate` command handler used by the CLI to translate templates between language conventions.

---

## Features

-   ✅ Parse and validate source/target language identifiers
-   ✅ Validate template path existence before processing
-   ✅ Translate templates to `.NET` (currently implemented target)
-   ✅ Report status through `ExitStatus` for CLI integration

---

## Contents

- [Introduction](#introduction)
- [Features](#features)
- [Contents](#contents)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Usage Examples](#usage-examples)
  - [Example 1: Run translation handler](#example-1-run-translation-handler)
- [API Reference](#api-reference)
  - [Core Types](#core-types)
  - [Handlers](#handlers)
- [References](#references)
- [License](#license)

---

## Installation

Add as a workspace/path dependency:

```toml
[dependencies]
nettoolskit-translate = { path = "../commands/translate" }
```

---

## Quick Start

Minimal usage in 3–5 lines:

```rust
use nettoolskit_translate::{handle_translate, TranslateRequest};

# #[tokio::main]
# async fn main() {
let status = handle_translate(TranslateRequest {
    from: "rust".to_string(),
    to: "dotnet".to_string(),
    path: "templates/Entity.cs.hbs".to_string(),
})
.await;

println!("Exit: {status:?}");
# }
```

---

## Usage Examples

### Example 1: Run translation handler

```rust
use nettoolskit_translate::{handle_translate, TranslateRequest};

# #[tokio::main]
# async fn main() {
let request = TranslateRequest {
    from: "python".to_string(),
    to: "dotnet".to_string(),
    path: "templates/Domain/Entity.cs.hbs".to_string(),
};

let status = handle_translate(request).await;
println!("{status:?}");
# }
```

---

## API Reference

### Core Types

```rust
#[derive(Debug, Clone)]
pub struct TranslateRequest {
    pub from: String,
    pub to: String,
    pub path: String,
}

#[derive(Debug, thiserror::Error)]
pub enum TranslateError {
    #[error("Unknown source language: {0}")]
    UnknownSourceLanguage(String),

    #[error("Unknown target language: {0}")]
    UnknownTargetLanguage(String),

    #[error("Template file not found: {0}")]
    TemplateNotFound(String),

    #[error("Translation from {from} to {to} is not supported")]
    UnsupportedTranslation { from: String, to: String },
}
```

### Handlers

```rust
pub async fn handle_translate(request: TranslateRequest) -> nettoolskit_core::ExitStatus;
```

---

## References

- https://docs.rs/thiserror

---

## License

This project is licensed under the MIT License. See the LICENSE file at the repository root for details.

---
