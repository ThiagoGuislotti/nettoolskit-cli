# NetToolsKit CLI - Architecture Migration Plan

**Project:** NetToolsKit CLI
**Target Architecture:** Workspace-based Modular Monolith (Codex-inspired)
**Planning Date:** 2025-11-06
**Version:** 1.0.0

---

## 📋 Executive Summary

### **Current State**
```
nettoolskit-cli/
├── cli/           # CLI + interactive mode
├── commands/      # Command processors
├── core/          # Core types
├── ui/            # Terminal UI
├── file-search/   # File search
├── ollama/        # Ollama integration
├── otel/          # Observability
├── async-utils/   # Async utilities
└── utils/         # String utilities
```

### **Target State**
```
nettoolskit-cli/
├── crates/
│   ├── core/              # Domain + Ports
│   ├── cli/               # CLI entry point
│   ├── ui/                # TUI (legacy + modern)
│   ├── formatting/        # Feature: Code formatting
│   ├── testing/           # Feature: Test coverage
│   ├── templating/        # Feature: Code generation
│   ├── file-system/       # Infrastructure: File operations
│   ├── otel/              # Observability
│   └── shared/            # Shared utilities
│       ├── async-utils/
│       ├── string-utils/
│       └── path-utils/
└── tests/
    ├── integration/
    └── e2e/
```

---

## 🎯 Migration Goals

### **Primary Objectives**
1. ✅ **Scalability**: Support 10+ new features without architectural changes
2. ✅ **Maintainability**: Clear boundaries between domains
3. ✅ **Testability**: Isolated testing per feature
4. ✅ **Reusability**: Crates can be used independently
5. ✅ **Clean Architecture**: Proper dependency inversion

### **Success Metrics**
- [ ] Zero circular dependencies
- [ ] 100% compile after migration
- [ ] All existing tests passing
- [ ] Documentation coverage ≥ 80%
- [ ] Feature isolation (each feature = 1 crate)

---

## 📅 Migration Phases

### **Phase 0: Preparation & Analysis** (1-2 days)
**Goal:** Understand current dependencies and prepare workspace structure

#### Tasks:
- [x] ✅ Analyze Codex architecture (DONE - see codex-architecture-analysis.md)
- [ ] Map current modules to target crates
- [ ] Identify circular dependencies
- [ ] Create dependency graph
- [ ] Define workspace structure
- [ ] Setup new `crates/` directory

#### Deliverables:
- [ ] Dependency graph diagram
- [ ] Module mapping document
- [ ] New Cargo.toml (workspace root)

---

### **Phase 1: Workspace Setup** (1 day)
**Goal:** Create workspace structure without breaking existing code

#### Tasks:
1. [ ] Create `crates/` directory
2. [ ] Update root `Cargo.toml` to workspace manifest
3. [ ] Define `[workspace.dependencies]`
4. [ ] Create placeholder crates:
   - [ ] `crates/core/`
   - [ ] `crates/cli/`
   - [ ] `crates/ui/`
   - [ ] `crates/formatting/`
   - [ ] `crates/testing/`
   - [ ] `crates/templating/`
   - [ ] `crates/file-system/`
   - [ ] `crates/shared/`
5. [ ] Keep existing structure in parallel (no deletion yet)

#### File Changes:
```toml
# NEW: Cargo.toml (workspace root)
[workspace]
members = [
    "crates/core",
    "crates/cli",
    "crates/ui",
    "crates/formatting",
    "crates/testing",
    "crates/templating",
    "crates/file-system",
    "crates/shared/async-utils",
    "crates/shared/string-utils",
    "crates/shared/path-utils",
    "crates/otel",
]
resolver = "2"

[workspace.package]
version = "0.2.0"
edition = "2021"
authors = ["NetToolsKit Team"]
license = "MIT"

[workspace.dependencies]
# Internal crates
nettoolskit-core = { path = "crates/core" }
nettoolskit-cli = { path = "crates/cli" }
nettoolskit-ui = { path = "crates/ui" }
nettoolskit-formatting = { path = "crates/formatting" }
nettoolskit-testing = { path = "crates/testing" }
nettoolskit-templating = { path = "crates/templating" }
nettoolskit-file-system = { path = "crates/file-system" }
nettoolskit-otel = { path = "crates/otel" }
nettoolskit-async-utils = { path = "crates/shared/async-utils" }
nettoolskit-string-utils = { path = "crates/shared/string-utils" }

# External dependencies
tokio = { version = "1", features = ["full"] }
anyhow = "1"
thiserror = "2"
serde = { version = "1", features = ["derive"] }
clap = { version = "4", features = ["derive"] }
crossterm = "0.28"
ratatui = "0.28"
handlebars = "6"
```

#### Validation:
```bash
cargo build --workspace
cargo test --workspace
```

---

### **Phase 2: Core Domain Migration** (2-3 days)
**Goal:** Extract domain logic and ports to `crates/core/`

#### Tasks:

#### **2.1 Domain Types**
- [ ] Create `crates/core/src/domain/`
  - [ ] `template.rs` - Template domain entity
  - [ ] `manifest.rs` - Manifest domain entity
  - [ ] `test_result.rs` - Test result value object
  - [ ] `file_descriptor.rs` - File metadata value object
  - [ ] `project_context.rs` - Project context aggregate

#### **2.2 Ports (Traits)**
- [ ] Create `crates/core/src/ports/`
  - [ ] `template_repository.rs` - Template storage contract
  - [ ] `file_system.rs` - File operations contract
  - [ ] `test_runner.rs` - Test execution contract
  - [ ] `code_formatter.rs` - Formatting contract
  - [ ] `template_engine.rs` - Template rendering contract

#### **2.3 Errors**
- [ ] Create `crates/core/src/error.rs`
  - [ ] Use `thiserror` for domain errors
  - [ ] Define `Result<T>` type alias

#### **2.4 Core Use Cases**
- [ ] Create `crates/core/src/use_cases/`
  - [ ] `apply_template.rs` - Template application logic
  - [ ] `format_code.rs` - Code formatting logic
  - [ ] `run_coverage.rs` - Test coverage logic

#### Structure:
```
crates/core/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── error.rs
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── template.rs
│   │   ├── manifest.rs
│   │   ├── test_result.rs
│   │   ├── file_descriptor.rs
│   │   └── project_context.rs
│   ├── ports/
│   │   ├── mod.rs
│   │   ├── template_repository.rs
│   │   ├── file_system.rs
│   │   ├── test_runner.rs
│   │   ├── code_formatter.rs
│   │   └── template_engine.rs
│   └── use_cases/
│       ├── mod.rs
│       ├── apply_template.rs
│       ├── format_code.rs
│       └── run_coverage.rs
└── tests/
    └── domain_tests.rs
```

#### Example Code:
```rust
// crates/core/src/domain/template.rs
#[derive(Debug, Clone)]
pub struct Template {
    pub name: String,
    pub path: PathBuf,
    pub technology: Technology,
    pub variables: HashMap<String, String>,
}

impl Template {
    pub fn new(name: String, path: PathBuf) -> Self {
        Self {
            name,
            path,
            technology: Technology::DotNet,
            variables: HashMap::new(),
        }
    }
}

// crates/core/src/ports/template_repository.rs
use async_trait::async_trait;
use crate::domain::Template;
use crate::error::Result;

#[async_trait]
pub trait TemplateRepository {
    async fn find_by_name(&self, name: &str) -> Result<Template>;
    async fn list_all(&self) -> Result<Vec<Template>>;
    async fn save(&self, template: &Template) -> Result<()>;
}

// crates/core/src/error.rs
use thiserror::Error;

pub type Result<T> = std::result::Result<T, DomainError>;

#[derive(Error, Debug)]
pub enum DomainError {
    #[error("Template not found: {0}")]
    TemplateNotFound(String),

    #[error("Invalid manifest: {0}")]
    InvalidManifest(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

#### Validation:
```bash
cd crates/core
cargo test
cargo doc --open
```

---

### **Phase 3: Feature Crates Migration** (3-5 days)
**Goal:** Extract features to independent crates

#### **3.1 Templating Feature**

**Tasks:**
- [ ] Create `crates/templating/`
- [ ] Move template-related logic from `commands/src/`
- [ ] Implement `TemplateRepository` trait
- [ ] Create Handlebars adapter

**Structure:**
```
crates/templating/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── domain/
│   │   ├── manifest_parser.rs
│   │   └── template_model.rs
│   ├── use_cases/
│   │   ├── apply_template.rs
│   │   └── render_code.rs
│   └── adapters/
│       ├── handlebars_engine.rs
│       ├── file_template_repository.rs
│       └── template_validator.rs
└── tests/
    ├── integration/
    │   └── template_application_tests.rs
    └── unit/
        └── manifest_parser_tests.rs
```

**Example:**
```rust
// crates/templating/src/adapters/file_template_repository.rs
use async_trait::async_trait;
use nettoolskit_core::ports::TemplateRepository;
use nettoolskit_core::domain::Template;
use nettoolskit_core::error::Result;

pub struct FileTemplateRepository {
    base_path: PathBuf,
}

#[async_trait]
impl TemplateRepository for FileTemplateRepository {
    async fn find_by_name(&self, name: &str) -> Result<Template> {
        let path = self.base_path.join(name);
        if !path.exists() {
            return Err(DomainError::TemplateNotFound(name.to_string()));
        }
        Ok(Template::new(name.to_string(), path))
    }
}
```

---

#### **3.2 Formatting Feature**

**Tasks:**
- [ ] Create `crates/formatting/`
- [ ] Implement `CodeFormatter` trait
- [ ] Support Rust, YAML, JSON, TOML

**Structure:**
```
crates/formatting/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── domain/
│   │   └── format_config.rs
│   ├── use_cases/
│   │   ├── format_file.rs
│   │   └── format_project.rs
│   └── adapters/
│       ├── rust_formatter.rs
│       ├── yaml_formatter.rs
│       └── prettier_adapter.rs
└── tests/
```

---

#### **3.3 Testing Feature**

**Tasks:**
- [ ] Create `crates/testing/`
- [ ] Implement `TestRunner` trait
- [ ] Support .NET, Rust test execution

**Structure:**
```
crates/testing/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── domain/
│   │   ├── test_result.rs
│   │   └── coverage_report.rs
│   ├── use_cases/
│   │   ├── run_coverage.rs
│   │   └── generate_report.rs
│   └── adapters/
│       ├── dotnet_test_runner.rs
│       ├── cargo_tarpaulin.rs
│       └── coverage_reporter.rs
└── tests/
```

---

### **Phase 4: Infrastructure Migration** (2-3 days)
**Goal:** Move infrastructure concerns to dedicated crates

#### **4.1 File System Adapter**

**Tasks:**
- [ ] Create `crates/file-system/`
- [ ] Implement `FileSystem` trait
- [ ] Handle file reading/writing

**Structure:**
```
crates/file-system/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── reader.rs
│   ├── writer.rs
│   └── watcher.rs
└── tests/
```

**Example:**
```rust
// crates/file-system/src/lib.rs
use async_trait::async_trait;
use nettoolskit_core::ports::FileSystem;
use nettoolskit_core::error::Result;

pub struct LocalFileSystem;

#[async_trait]
impl FileSystem for LocalFileSystem {
    async fn read_to_string(&self, path: &Path) -> Result<String> {
        tokio::fs::read_to_string(path)
            .await
            .map_err(Into::into)
    }

    async fn write(&self, path: &Path, content: &str) -> Result<()> {
        tokio::fs::write(path, content)
            .await
            .map_err(Into::into)
    }
}
```

---

#### **4.2 Move Existing Crates**

**Tasks:**
- [ ] Move `otel/` → `crates/otel/`
- [ ] Move `ollama/` → `crates/ollama/` (or remove if unused)
- [ ] Move `file-search/` → `crates/file-search/`

---

### **Phase 5: Shared Utilities Reorganization** (1-2 days)
**Goal:** Organize utilities as namespace

#### Tasks:
- [ ] Create `crates/shared/`
- [ ] Move `async-utils/` → `crates/shared/async-utils/`
- [ ] Move `utils/` → `crates/shared/string-utils/`
- [ ] Create `crates/shared/path-utils/`

**Structure:**
```
crates/shared/
├── async-utils/
│   ├── Cargo.toml
│   └── src/
│       ├── cancellation.rs
│       └── timeout.rs
├── string-utils/
│   ├── Cargo.toml
│   └── src/
│       └── string.rs
└── path-utils/
    ├── Cargo.toml
    └── src/
        └── normalize.rs
```

---

### **Phase 6: CLI & UI Migration** (2-3 days)
**Goal:** Migrate entry points and UI

#### **6.1 CLI Entry Point**

**Tasks:**
- [ ] Move `cli/` → `crates/cli/`
- [ ] Update to use new crates
- [ ] Dependency injection setup

**Structure:**
```
crates/cli/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── commands/
│   │   ├── mod.rs
│   │   ├── format_cmd.rs
│   │   ├── test_cmd.rs
│   │   └── template_cmd.rs
│   └── di.rs          # Dependency injection
└── tests/
```

**Example:**
```rust
// crates/cli/src/di.rs
use nettoolskit_core::ports::*;
use nettoolskit_templating::FileTemplateRepository;
use nettoolskit_file_system::LocalFileSystem;
use std::sync::Arc;

pub struct AppContainer {
    pub template_repo: Arc<dyn TemplateRepository>,
    pub file_system: Arc<dyn FileSystem>,
}

impl AppContainer {
    pub fn new() -> Self {
        Self {
            template_repo: Arc::new(FileTemplateRepository::new("./templates")),
            file_system: Arc::new(LocalFileSystem),
        }
    }
}
```

---

#### **6.2 UI Migration**

**Tasks:**
- [ ] Move `ui/` → `crates/ui/`
- [ ] Keep legacy + modern structure
- [ ] Update imports

---

### **Phase 7: Testing & Validation** (2-3 days)
**Goal:** Ensure everything works

#### Tasks:

#### **7.1 Unit Tests**
- [ ] Run all unit tests: `cargo test --workspace`
- [ ] Fix broken tests
- [ ] Add missing tests for new boundaries

#### **7.2 Integration Tests**
- [ ] Create `tests/integration/`
  - [ ] `formatting_integration_tests.rs`
  - [ ] `testing_integration_tests.rs`
  - [ ] `templating_integration_tests.rs`

#### **7.3 E2E Tests**
- [ ] Create `tests/e2e/`
  - [ ] `cli_workflow_tests.rs`

#### **7.4 Performance Tests**
- [ ] Benchmark template rendering
- [ ] Benchmark file operations

---

### **Phase 8: Documentation** (1-2 days)
**Goal:** Document new architecture

#### Tasks:
- [ ] Update README.md (root)
- [ ] Create README.md for each crate
- [ ] Document architecture decisions (ADR)
- [ ] Update CHANGELOG.md
- [ ] Generate API docs: `cargo doc --workspace --no-deps --open`

#### Documentation Structure:
```
.docs/
├── planning/
│   ├── architecture-migration-plan.md (THIS FILE)
│   ├── codex-architecture-analysis.md
│   └── translation-plan.md
├── architecture/
│   ├── overview.md
│   ├── dependency-graph.md
│   └── adr/
│       ├── 001-workspace-structure.md
│       ├── 002-hexagonal-architecture.md
│       └── 003-error-handling-strategy.md
└── guides/
    ├── development.md
    └── testing.md
```

---

### **Phase 9: Cleanup** (1 day)
**Goal:** Remove old structure

#### Tasks:
- [ ] Delete old root-level crates (after validation)
  - [ ] `commands/`
  - [ ] `core/`
  - [ ] Old `cli/`
  - [ ] Old `ui/`
- [ ] Update CI/CD pipelines
- [ ] Update build scripts
- [ ] Clean up unused dependencies

---

### **Phase 10: Release** (1 day)
**Goal:** Publish new version

#### Tasks:
- [ ] Tag version `v0.2.0`
- [ ] Update CHANGELOG.md
- [ ] Create GitHub Release
- [ ] Announce architecture changes

---

## 📊 Timeline Summary

| **Phase** | **Duration** | **Dependencies** |
|-----------|--------------|------------------|
| Phase 0: Preparation | 1-2 days | None |
| Phase 1: Workspace Setup | 1 day | Phase 0 |
| Phase 2: Core Domain | 2-3 days | Phase 1 |
| Phase 3: Features | 3-5 days | Phase 2 |
| Phase 4: Infrastructure | 2-3 days | Phase 2 |
| Phase 5: Shared Utils | 1-2 days | Phase 1 |
| Phase 6: CLI & UI | 2-3 days | Phases 2-5 |
| Phase 7: Testing | 2-3 days | Phase 6 |
| Phase 8: Documentation | 1-2 days | Phase 7 |
| Phase 9: Cleanup | 1 day | Phase 8 |
| Phase 10: Release | 1 day | Phase 9 |

**Total Estimated Time:** 17-28 days (3-5 weeks)

---

## 🎯 Critical Success Factors

### **Must-Have**
1. ✅ Zero breaking changes for end users
2. ✅ All tests passing
3. ✅ Backward compatibility maintained
4. ✅ Documentation updated

### **Nice-to-Have**
1. ✅ Performance improvements
2. ✅ Reduced compilation time
3. ✅ Better IDE support
4. ✅ Example projects for each feature

---

## 🚨 Risks & Mitigation

### **Risk 1: Circular Dependencies**
- **Mitigation:** Use dependency graph analysis before migration
- **Tool:** `cargo depgraph` or `cargo-modules`

### **Risk 2: Breaking Changes**
- **Mitigation:** Keep old structure in parallel during migration
- **Validation:** Run old tests against new structure

### **Risk 3: Performance Regression**
- **Mitigation:** Benchmark before and after migration
- **Tool:** `criterion` for benchmarking

### **Risk 4: Lost Functionality**
- **Mitigation:** Comprehensive testing before cleanup
- **Validation:** E2E tests covering all workflows

---

## 📋 Acceptance Criteria

### **Phase Completion Checklist**
- [ ] All phases completed
- [ ] `cargo build --workspace --release` succeeds
- [ ] `cargo test --workspace` passes (100%)
- [ ] `cargo clippy --workspace -- -D warnings` passes
- [ ] `cargo doc --workspace --no-deps` generates docs
- [ ] CI/CD green
- [ ] Documentation complete
- [ ] Old structure removed
- [ ] Version tagged

---

## 📚 References

- [Codex Architecture Analysis](./codex-architecture-analysis.md)
- [Cargo Workspaces](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html)
- [Clean Architecture in Rust](https://www.qovery.com/blog/clean-architecture-in-rust/)
- [Hexagonal Architecture](https://alistair.cockburn.us/hexagonal-architecture/)

---

## 🔄 Next Steps

1. **Review this plan** with the team
2. **Start Phase 0** (Preparation & Analysis)
3. **Create feature branch**: `feature/architecture-migration`
4. **Track progress** in GitHub Projects/Issues
5. **Weekly sync** to adjust timeline

---

**Status:** 📝 DRAFT - Awaiting approval
**Owner:** NetToolsKit Team
**Last Updated:** 2025-11-06
