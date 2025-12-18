# nettoolskit-commands

> Command collection and dispatcher for NetToolsKit CLI.

---

## Introduction

nettoolskit-commands aggregates command subcrates used by the CLI and re-exports them for convenient consumption from other workspace crates.

---

## Features

-   ✅ Single import point for command subcrates
-   ✅ Re-exports command packages used by the CLI

---

## Contents

- [Introduction](#introduction)
- [Features](#features)
- [Contents](#contents)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Usage Examples](#usage-examples)
  - [Example 1: Import command crates](#example-1-import-command-crates)
- [API Reference](#api-reference)
- [References](#references)
- [License](#license)

---

## Installation

Add as a workspace/path dependency:

```toml
[dependencies]
nettoolskit-commands = { path = "../commands" }
```

---

## Quick Start

Minimal usage in 3–5 lines:

```rust
use nettoolskit_commands::{nettoolskit_help, nettoolskit_manifest, nettoolskit_translate};

let _ = (nettoolskit_help::discover_manifests, nettoolskit_manifest::ManifestExecutor::new, nettoolskit_translate::handle_translate);
```

---

## Usage Examples

### Example 1: Import command crates

```rust
use nettoolskit_commands::{nettoolskit_help, nettoolskit_manifest};

# #[tokio::main]
# async fn main() {
let manifests = nettoolskit_help::discover_manifests(None).await;
println!("{}", manifests.len());

let _executor = nettoolskit_manifest::ManifestExecutor::new();
# }
```

---

## API Reference

This crate re-exports the command subcrates:

- `nettoolskit_help`
- `nettoolskit_manifest`
- `nettoolskit_translate`

---

## References

- https://doc.rust-lang.org/cargo/reference/workspaces.html

---

## License

This project is licensed under the MIT License. See the LICENSE file at the repository root for details.

---
