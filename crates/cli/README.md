# nettoolskit-cli

> Interactive command-line interface (UI layer) for NetToolsKit.

---

## Introduction

nettoolskit-cli provides the interactive terminal experience for NetToolsKit, integrating UI rendering, event handling, telemetry, and orchestration to run commands.

---

## Features

-   ✅ Interactive mode entry point for the `ntk` binary
-   ✅ Terminal raw-mode guarding and event-driven input
-   ✅ Integrates orchestrator, UI, and telemetry crates

---

## Contents

- [Introduction](#introduction)
- [Features](#features)
- [Contents](#contents)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Usage Examples](#usage-examples)
  - [Example 1: Run interactive mode](#example-1-run-interactive-mode)
- [API Reference](#api-reference)
  - [Entry Points](#entry-points)
- [References](#references)
- [License](#license)

---

## Installation

Add as a workspace/path dependency:

```toml
[dependencies]
nettoolskit-cli = { path = "../cli" }
```

---

## Quick Start

Minimal usage in 3–5 lines:

```rust
use nettoolskit_cli::interactive_mode;

# #[tokio::main]
# async fn main() {
let status = interactive_mode(false).await;
println!("{status:?}");
# }
```

---

## Usage Examples

### Example 1: Run interactive mode

```rust
use nettoolskit_cli::interactive_mode;

# #[tokio::main]
# async fn main() {
let verbose = true;
let exit = interactive_mode(verbose).await;
println!("{exit:?}");
# }
```

---

## API Reference

### Entry Points

```rust
pub async fn interactive_mode(verbose: bool) -> nettoolskit_orchestrator::ExitStatus;
```

---

## References

- https://docs.rs/crossterm
- https://docs.rs/clap

---

## License

This project is licensed under the MIT License. See the LICENSE file at the repository root for details.

---
