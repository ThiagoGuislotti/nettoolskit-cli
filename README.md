# NetToolsKit CLI

> Interactive command-line interface for .NET development with templates, manifests, and automation tools

---

## Introduction

NetToolsKit CLI solves the problem of fragmented .NET development workflows by providing a unified, interactive command-line interface for project scaffolding, template management, and development automation. The technical approach follows a modular Rust-based architecture inspired by GitHub Codex CLI, featuring an interactive command palette system and high-performance async operations.

---

## Features

✅ Interactive terminal interface with `/` command palette activation
✅ Real-time command filtering with intuitive arrow navigation
✅ Modular workspace architecture with 9 specialized crates
✅ High-performance async I/O with concurrent operations support
✅ Template scaffolding and project generation system
✅ Manifest-driven solution configuration and application
✅ OpenTelemetry integration for observability and metrics
✅ Ollama integration for AI-powered development assistance

---

## Table of Contents

- [Introduction](#introduction)
- [Features](#features)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Usage Examples](#usage-examples)
  - [Interactive Mode](#interactive-mode)
  - [Direct Commands](#direct-commands)
  - [Template Operations](#template-operations)
- [API Reference](#api-reference)
  - [Slash Commands](#slash-commands)
  - [CLI Arguments](#cli-arguments)
  - [Configuration](#configuration)
- [Build and Tests](#build-and-tests)
- [Contributing](#contributing)
- [Dependencies](#dependencies)
- [References](#references)

---

## Installation

### Via Cargo Install

```bash
cargo install nettoolskit-cli
```

### From Source

```bash
# Clone the repository
git clone https://github.com/ThiagoGuislotti/NetToolsKit.git

# Navigate to the CLI directory
cd NetToolsKit/tools/nettoolskit-cli

# Build and install
cargo install --path cli
```

### Binary Releases

Download pre-built binaries from [GitHub Releases](https://github.com/ThiagoGuislotti/NetToolsKit/releases).

---

## Quick Start

Start the interactive CLI and create your first project:

```bash
# Launch interactive mode
ntk

# Use the command palette (type / to activate)
/new
```

---

## Usage Examples

### Interactive Mode

Launch the interactive CLI for an enhanced development experience:

```bash
# Start interactive mode
ntk
```

**Command Palette Usage:**
1. Type `/` to open the command palette
2. Continue typing to filter commands in real-time
3. Use `↑` and `↓` arrows to navigate options
4. Press `Enter` or `Tab` to select a command
5. Press `Esc` to cancel and return

### Direct Commands

Execute commands directly without interactive mode:

```bash
# List available templates
ntk list --filter "dotnet"

# Create new project from template
ntk new dotnet-api --name "MyAPI" --output "./my-api"

# Validate manifest or template
ntk check manifest.yml --strict

# Render template preview
ntk render dotnet-api --vars variables.json

# Apply manifest to existing solution (outputs to ./target/ntk-output by default)
ntk apply manifest.yml --output ./generated-solution
```

### Template Operations

Advanced template management and project scaffolding:

```bash
# List templates with filtering
ntk list --category web --language csharp

# Create project with custom variables
ntk new microservice-template \
    --name "UserService" \
    --namespace "MyCompany.Services" \
    --output "./services/user-service" \
    --vars '{"Port": 5001, "Database": "PostgreSQL"}'

# Validate template structure
ntk check ./my-custom-template --validate-schema

# Preview generated files without creating them
ntk render web-api-template \
    --vars ./config.json \
    --preview-only
```

---

## API Reference

### Slash Commands

Available interactive commands accessible through the `/` command palette:

| Command | Description | Arguments |
|---------|-------------|-----------|
| `/list` | List available templates | `--filter`, `--category` |
| `/new` | Create project from template | `--name`, `--output`, `--vars` |
| `/check` | Validate manifest or template | `--strict`, `--schema` |
| `/render` | Render template preview | `--vars`, `--preview-only` |
| `/apply` | Apply manifest to existing solution | `--output`, `--force` |
| `/quit` | Exit NetToolsKit CLI | none |

### CLI Arguments

Global arguments available across all commands:

```bash
ntk [GLOBAL_OPTIONS] <COMMAND> [COMMAND_OPTIONS]
```

**Global Options:**
- `--log-level <LEVEL>`: Set logging level (off, error, warn, info, debug, trace)
- `--config <PATH>`: Path to configuration file
- `--verbose, -v`: Enable verbose output

### Configuration

The CLI can be configured via configuration file or environment variables:

```toml
# ntk.toml
[templates]
search_paths = ["./templates", "~/.ntk/templates"]
default_language = "csharp"

[ollama]
base_url = "http://localhost:11434"
default_model = "codellama"

[telemetry]
enabled = true
endpoint = "http://localhost:4317"
```

**Architecture Overview:**

```
nettoolskit-cli/
├── crates/
│   ├── cli/                      # Main CLI entry point
│   ├── commands/                 # Feature dispatcher (thin orchestrator)
│   │   ├── src/                  # Command routing and processors
│   │   ├── templating/           # Code generation feature (33 tests ✅)
│   │   └── README.md             # Commands architecture details
│   ├── core/                     # Core types and shared functionality
│   ├── ui/                       # Terminal UI components (ratatui)
│   ├── otel/                     # OpenTelemetry observability
│   ├── ollama/                   # AI integration via Ollama
│   └── shared/                   # Shared utilities
│       ├── async-utils/          # Async utilities and timeout management
│       ├── file-search/          # File discovery and filtering
│       └── utils/                # String utilities and helpers
└── Cargo.toml                    # Workspace configuration (10 crates)
```

**Key Architecture Principles:**
- ✅ **Modular Monolith**: Workspace with 10 crates
- ✅ **Feature-First**: Commands dispatcher with feature sub-crates
- ✅ **Clean Architecture**: Ports/Adapters within features
- ✅ **Shared Infrastructure**: Common utilities in `shared/`

---

## Build and Tests

```bash
# Build the project
cargo build

# Run all tests
cargo test --all

# Run specific test categories
cargo test --all --release

# Format code
cargo fmt

# Run linter
cargo clippy --all-targets --all-features

# Generate documentation
cargo doc --no-deps --open
```

---

## Contributing

We follow semantic versioning and conventional commits. Please ensure your contributions:

1. **Follow Git Flow**: Create feature branches from `main`
2. **Write Tests**: Maintain 100% test coverage for new features
3. **Use Semantic Commits**: Follow conventional commit format
4. **Update Documentation**: Keep README and inline docs current

```bash
# Example workflow
git checkout -b feature/new-template-engine
# ... make changes ...
git commit -m "feat: add handlebars template engine support"
git push origin feature/new-template-engine
# ... create pull request ...
```

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
