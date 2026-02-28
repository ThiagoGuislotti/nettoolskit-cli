# Contributing to NetToolsKit CLI

Thank you for your interest in contributing to NetToolsKit CLI. This document outlines the development workflow, coding standards, and contribution process.

---

## Prerequisites

- **Rust** 1.75.0+ (see `rust-toolchain.toml` or install via [rustup](https://rustup.rs/))
- **Git** 2.30+
- A terminal that supports UTF-8 (Windows Terminal, iTerm2, or equivalent)

---

## Setup

```bash
# Clone the repository
git clone https://github.com/ThiagoGuislotti/nettoolskit-cli.git
cd nettoolskit-cli

# Build all crates
cargo build --workspace

# Run all tests
cargo test --workspace

# Verify formatting and lints
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
```

---

## Development Workflow

1. **Create a feature branch** from `main`:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes** following the coding standards below.

3. **Run the validation suite** before committing:
   ```bash
   cargo build --workspace
   cargo test --workspace
   cargo clippy --workspace -- -D warnings
   cargo fmt --all -- --check
   ```

4. **Commit** using conventional commit format (see below).

5. **Open a Pull Request** against `main`.

---

## Coding Standards

### Rust Style

- Follow `rustfmt` configuration (see `rustfmt.toml`)
- Prefer `sealed` types when inheritance is not intended
- Use `PascalCase` for types, `snake_case` for functions and variables
- Keep `use` statements clean and organized
- All public APIs must have `///` documentation comments
- Workspace enforces `#![forbid(unsafe_code)]`

### Architecture

- Follow the layered crate dependency graph (see README architecture diagram)
- `core` and `ui` are leaf crates — they must not depend on higher-level crates
- `commands` sub-crates depend on `core`, `ui`, and `otel` only
- `orchestrator` coordinates `commands` and `ui`
- `cli` is the top-level entry point

### Testing

- Unit tests go in `#[cfg(test)]` modules within each source file
- Integration tests go in `crates/<crate>/tests/`
- Use `insta` for snapshot testing of rendered output
- Tests modifying global state (e.g., capability overrides) must use synchronization
- Test categories: use descriptive test names that indicate the behavior being tested

---

## Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

[optional body]
```

Types: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`, `perf`, `ci`

Examples:
```
feat(manifest): add YAML manifest validation
fix(ui): resolve resize flickering on Windows Terminal
docs(readme): update architecture diagram
test(core): add config loader edge cases
```

---

## Pull Request Process

1. Ensure all CI checks pass (build, test, clippy, fmt)
2. Provide a clear description of the change and its rationale
3. Reference related issues if applicable
4. Keep PRs focused — one logical change per PR
5. Update documentation if the change affects public APIs or behavior

---

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

---