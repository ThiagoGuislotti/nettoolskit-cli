# NetToolsKit CLI

> Interactive command-line interface for .NET development with templates, manifests, and automation tools

---

## Introduction

NetToolsKit CLI solves the problem of fragmented .NET development workflows by providing a unified, interactive command-line interface for project scaffolding, template management, and development automation. The technical approach follows a modular Rust-based architecture inspired by GitHub Codex CLI, featuring an interactive command palette system and high-performance async operations.

---

## Features

-   ✅ Interactive terminal interface with `/` command palette activation
-   ✅ Real-time command filtering with intuitive arrow navigation
-   ✅ Modular workspace architecture with specialized crates
-   ✅ High-performance async I/O with concurrent operations support
-   ✅ Template scaffolding and project generation system
-   ✅ Manifest-driven solution configuration and application
-   ✅ OpenTelemetry integration for observability and metrics
-   ✅ Ollama integration for AI-powered development assistance

---

## Contents

- [Introduction](#introduction)
- [Features](#features)
- [Contents](#contents)
  - [Crates](#crates)
- [Build and Tests](#build-and-tests)
- [Contributing](#contributing)
- [Dependencies](#dependencies)
- [References](#references)
- [License](#license)

---

### Crates

This workspace is organized as a multi-crate Rust project. Each crate has its own README with scoped documentation.

- `cli`: Interactive entry point and UI loop. See [crates/cli/README.md](crates/cli/README.md).
- `commands`: Command dispatch layer and feature crates. See [crates/commands/README.md](crates/commands/README.md).
  - `help`: Help and manifest discovery utilities. See [crates/commands/help/README.md](crates/commands/help/README.md).
  - `manifest`: Manifest parsing, validation, and execution. See [crates/commands/manifest/README.md](crates/commands/manifest/README.md).
  - `templating`: Template rendering and resolution. See [crates/commands/templating/README.md](crates/commands/templating/README.md).
  - `translate`: Template translation handler and types. See [crates/commands/translate/README.md](crates/commands/translate/README.md).
- `core`: Shared domain types and utilities. See [crates/core/README.md](crates/core/README.md).
- `orchestrator`: High-level command orchestration and execution. See [crates/orchestrator/README.md](crates/orchestrator/README.md).
- `otel`: Observability and OpenTelemetry integration. See [crates/otel/README.md](crates/otel/README.md).
- `ui`: Terminal UI primitives and helpers. See [crates/ui/README.md](crates/ui/README.md).

---

## Build and Tests

This repository uses standard Cargo workflows for building, testing, formatting, linting, and documentation generation.

---

## Contributing

We follow semantic versioning and conventional commits. Please ensure your contributions:

1. **Follow Git Flow**: Create feature branches from `main`
2. **Write Tests**: Maintain 100% test coverage for new features
3. **Use Semantic Commits**: Follow conventional commit format
4. **Update Documentation**: Keep README and inline docs current

Keep commit messages aligned with Conventional Commits.

---

## Dependencies

### Runtime Dependencies
- `tokio` (1.34+) - Async runtime with multi-threading support
- `clap` (4.4+) - Command-line argument parsing
- `serde` (1.0+) - Serialization framework
- `reqwest` (0.11+) - HTTP client for Ollama integration
- `crossterm` (0.27+) - Cross-platform terminal manipulation

### Development Dependencies
- `anyhow` (1.0+) - Error handling
- `tracing` (0.1+) - Structured logging
- `opentelemetry` (0.21+) - Observability and metrics

---

## References

- Crate documentation:
  - [crates/cli/README.md](crates/cli/README.md)
  - [crates/commands/README.md](crates/commands/README.md)
  - [crates/commands/help/README.md](crates/commands/help/README.md)
  - [crates/commands/manifest/README.md](crates/commands/manifest/README.md)
  - [crates/commands/templating/README.md](crates/commands/templating/README.md)
  - [crates/commands/translate/README.md](crates/commands/translate/README.md)
  - [crates/core/README.md](crates/core/README.md)
  - [crates/orchestrator/README.md](crates/orchestrator/README.md)
  - [crates/otel/README.md](crates/otel/README.md)
  - [crates/ui/README.md](crates/ui/README.md)

- [Rust Async Programming](https://rust-lang.github.io/async-book/)
- [Clap CLI Framework](https://docs.rs/clap/latest/clap/)
- [Tokio Async Runtime](https://tokio.rs/tokio/tutorial)
- [OpenTelemetry Rust](https://opentelemetry.io/docs/instrumentation/rust/)
- [GitHub Codex CLI Design](https://github.com/github/copilot-cli) (inspiration)
- [GitHub Issues](https://github.com/ThiagoGuislotti/NetToolsKit/issues)

---

## License

This project is licensed under the MIT License. See the LICENSE file at the repository root for details.

---
