# Apply.rs Refactoring Analysis
**File:** `commands/src/apply.rs`
**Current Size:** 2,205 lines (confirmed: 1,979 actual code + tests)
**Target:** Split into `crates/commands/manifest/` with SOLID principles

---

## 📊 File Structure Analysis

### 1. Imports & Constants (Lines 1-20)
- **Lines:** ~20
- **Content:** Dependencies, regex, constants
- **Target:** `crates/commands/manifest/src/lib.rs` + config module

### 2. CLI Args (Lines 23-40)
- **Struct:** `ApplyArgs`
- **Target:** `crates/commands/manifest/src/lib.rs` (public API)

### 3. Entry Points (Lines 42-165)
- **Functions:**
  - `run()` - CLI entry point
  - `execute_apply()` - Async wrapper
  - `resolve_manifest_path()` - Path resolution
  - `resolve_output_root()` - Output directory resolution
- **Target:** `crates/commands/manifest/src/lib.rs` (public API)

---

## 📦 Data Models (Lines 126-560)

### Core Configuration
| Struct | Purpose | Target Module | Lines |
|--------|---------|---------------|-------|
| `ApplyConfig` | Configuration | `models/config.rs` | ~10 |
| `ApplySummary` | Result summary | `models/summary.rs` | ~60 |

### Manifest Data Structures (Domain Models)
| Struct | Purpose | Target Module | Lines |
|--------|---------|---------------|-------|
| `ManifestDocument` | Root document | `models/document.rs` | ~80 |
| `ManifestMeta` | Metadata | `models/meta.rs` | ~10 |
| `ManifestConventions` | Conventions | `models/meta.rs` | ~10 |
| `ManifestSolution` | Solution config | `models/solution.rs` | ~10 |
| `ManifestGuards` | Guards config | `models/solution.rs` | ~10 |
| `ManifestProject` | Project config | `models/project.rs` | ~30 |
| `ManifestPolicy` | Collision policy | `models/policy.rs` | ~30 |
| `ManifestContext` | Bounded context | `models/context.rs` | ~10 |
| `ManifestAggregate` | Aggregate root | `models/domain.rs` | ~20 |
| `ManifestValueObject` | Value object | `models/domain.rs` | ~10 |
| `ManifestEntity` | Entity | `models/domain.rs` | ~10 |
| `ManifestDomainEvent` | Domain event | `models/domain.rs` | ~10 |
| `ManifestRepository` | Repository | `models/application.rs` | ~10 |
| `ManifestRepositoryMethod` | Repository method | `models/application.rs` | ~15 |
| `ManifestMethodArgument` | Method argument | `models/application.rs` | ~10 |
| `ManifestUseCase` | Use case | `models/application.rs` | ~20 |
| `ManifestEndpoint` | API endpoint | `models/api.rs` | ~20 |
| `ManifestEnum` | Enumeration | `models/enums.rs` | ~10 |
| `ManifestEnumValue` | Enum value | `models/enums.rs` | ~10 |
| `ManifestField` | Field definition | `models/common.rs` | ~15 |
| `ManifestTemplates` | Templates config | `models/templates.rs` | ~30 |
| `TemplateMapping` | Template mapping | `models/templates.rs` | ~20 |

**Total Models Lines:** ~400 lines
**Models Organization:**
```
models/
├── mod.rs               # Re-exports
├── config.rs            # ApplyConfig
├── summary.rs           # ApplySummary
├── document.rs          # ManifestDocument (root)
├── meta.rs              # ManifestMeta, ManifestKind, ManifestConventions
├── solution.rs          # ManifestSolution, ManifestGuards
├── project.rs           # ManifestProject, ProjectLayerKind
├── policy.rs            # ManifestPolicy, CollisionPolicy
├── context.rs           # ManifestContext
├── domain.rs            # Aggregate, Entity, ValueObject, DomainEvent
├── application.rs       # Repository, RepositoryMethod, UseCase
├── api.rs               # ManifestEndpoint
├── enums.rs             # ManifestEnum, ManifestEnumValue
├── common.rs            # ManifestField
└── templates.rs         # ManifestTemplates, TemplateMapping, ArtifactKind
```

---

## 🎯 Core Logic (Lines 560-1850)

### Main Orchestration (Lines 618-792)
| Function | Purpose | Target Module | Lines |
|----------|---------|---------------|-------|
| `apply_sync()` | Main orchestrator | `orchestrator.rs` | ~170 |
| `locate_templates_root()` | Find templates | `orchestrator.rs` | ~30 |
| `ensure_directory()` | Create dirs | `files/utils.rs` | ~30 |

**Orchestrator Responsibilities:**
1. Load manifest
2. Validate
3. Locate templates
4. Collect render tasks
5. Render templates
6. Execute file changes
7. Return summary

**Target:** `orchestrator.rs` (main workflow, uses DIP)

---

### Task Building System (Lines 842-1285)

| Function | Purpose | Target Module | Lines |
|----------|---------|---------------|-------|
| `collect_render_tasks()` | Collect all tasks | `tasks/collector.rs` | ~200 |
| `select_contexts()` | Select contexts | `tasks/collector.rs` | ~15 |
| `append_domain_tasks()` | Domain tasks | `tasks/builders/domain.rs` | ~120 |
| `append_application_tasks()` | Application tasks | `tasks/builders/application.rs` | ~40 |
| `append_api_tasks()` | API tasks | `tasks/builders/api.rs` | ~75 |

**Task Builders:** ~450 lines
**Organization:**
```
tasks/
├── mod.rs
├── render_task.rs          # RenderTask struct (from line 188)
├── collector.rs            # collect_render_tasks, select_contexts
└── builders/
    ├── mod.rs
    ├── domain.rs           # append_domain_tasks
    ├── application.rs      # append_application_tasks
    └── api.rs              # append_api_tasks
```

---

### Artifact Locators (Lines 1285-1480)

| Function | Purpose | Target Module | Lines |
|----------|---------|---------------|-------|
| `find_value_object()` | Locate VO | `tasks/locators.rs` | ~35 |
| `find_entity()` | Locate entity | `tasks/locators.rs` | ~35 |
| `find_domain_event()` | Locate event | `tasks/locators.rs` | ~35 |
| `find_repository()` | Locate repository | `tasks/locators.rs` | ~30 |
| `find_use_case()` | Locate use case | `tasks/locators.rs` | ~35 |
| `find_enum()` | Locate enum | `tasks/locators.rs` | ~35 |

**Locators:** ~200 lines
**Purpose:** Find artifacts across contexts (cross-cutting queries)
**Target:** `tasks/locators.rs`

---

### Serializers (Lines 1480-1523)

| Function | Purpose | Target Module | Lines |
|----------|---------|---------------|-------|
| `serialize_field()` | Field → JSON | `tasks/serializers.rs` | ~25 |
| `serialize_repository_method()` | Method → JSON | `tasks/serializers.rs` | ~20 |

**Serializers:** ~45 lines
**Purpose:** Convert domain models to JSON for template rendering
**Target:** `tasks/serializers.rs`

---

### Task Builders (Lines 1523-1783)

| Function | Purpose | Target Module | Lines |
|----------|---------|---------------|-------|
| `build_enum_task()` | Enum task | `tasks/builders/domain.rs` | ~45 |
| `build_value_object_task()` | VO task | `tasks/builders/domain.rs` | ~35 |
| `build_entity_task()` | Entity task | `tasks/builders/domain.rs` | ~40 |
| `build_domain_event_task()` | Event task | `tasks/builders/domain.rs` | ~30 |
| `build_repository_task()` | Repository task | `tasks/builders/application.rs` | ~40 |
| `build_use_case_task()` | UseCase task | `tasks/builders/application.rs` | ~45 |
| `build_endpoint_task()` | Endpoint task | `tasks/builders/api.rs` | ~40 |

**Builders:** ~275 lines
**Purpose:** Create individual RenderTask instances
**Already mapped above** to `tasks/builders/*`

---

### Template Rendering (Lines 1783-1900)

| Function | Purpose | Target Module | Lines |
|----------|---------|---------------|-------|
| `render_template()` | Render template | `crates/commands/templating/src/engine.rs` | ~45 |
| `locate_template_file()` | Find template | `crates/commands/templating/src/resolver.rs` | ~30 |
| `resolve_destination()` | Resolve path | `files/utils.rs` | ~20 |
| `lookup_placeholder()` | Placeholder | `files/utils.rs` | ~20 |
| `normalize_line_endings()` | Line endings | `crates/shared/string-utils/` | ~5 |
| `to_lower_camel()` | Camel case | `crates/shared/string-utils/` | ~15 |

**Note:** Template rendering functions → **separate crate** `crates/commands/templating/`

---

### File Operations (Lines 1900-1990)

| Function | Purpose | Target Module | Lines |
|----------|---------|---------------|-------|
| `execute_plan()` | Write files | `files/executor.rs` | ~45 |
| `build_solution_stub()` | Solution stub | `stubs/solution.rs` | ~15 |
| `build_project_stub()` | Project stub | `stubs/project.rs` | ~15 |
| `build_project_payload()` | Project payload | `stubs/project.rs` | ~35 |

**File Operations:** ~110 lines
**Organization:**
```
files/
├── mod.rs
├── changes.rs           # FileChange struct (from line 203)
├── executor.rs          # execute_plan
└── utils.rs             # resolve_destination, lookup_placeholder

stubs/
├── mod.rs
├── solution.rs          # build_solution_stub
└── project.rs           # build_project_stub, build_project_payload
```

---

### Tests (Lines 1990-2205)

| Test | Purpose | Target |
|------|---------|--------|
| `manifest_path()` | Helper | Keep in tests |
| `dry_run_artifact_manifest_generates_plan()` | Dry run | `tests/dry_run_tests.rs` |
| `apply_artifact_manifest_writes_files()` | Apply | `tests/apply_tests.rs` |
| `acceptance_manifest_generates_cross_layer_assets()` | Cross-layer | `tests/integration_tests.rs` |
| `acceptance_manifest_exports_to_target()` | Export | `tests/integration_tests.rs` |

**Tests:** ~215 lines
**Organization:**
```
tests/
├── dry_run_tests.rs
├── apply_tests.rs
└── integration_tests.rs
```

---

## 🏗️ SOLID Principles Application

### Ports (Traits) - NEW CODE

Create `ports/` module with traits for Dependency Inversion:

```rust
// ports/manifest_parser.rs
#[async_trait]
pub trait ManifestParser: Send + Sync {
    async fn parse(&self, path: &Path) -> Result<ManifestDocument>;
    async fn validate(&self, doc: &ManifestDocument) -> Result<()>;
}

// ports/template_renderer.rs
#[async_trait]
pub trait TemplateRenderer: Send + Sync {
    async fn render(&self, template: &str, context: &Value) -> Result<String>;
    async fn locate_template(&self, name: &str) -> Result<PathBuf>;
}

// ports/file_writer.rs
#[async_trait]
pub trait FileWriter: Send + Sync {
    async fn write(&self, changes: Vec<FileChange>) -> Result<()>;
    async fn ensure_directory(&self, path: &Path) -> Result<()>;
}

// ports/language_adapter.rs (for future multi-language)
#[async_trait]
pub trait LanguageAdapter: Send + Sync {
    fn language(&self) -> TargetLanguage;
    fn map_type(&self, generic_type: &str) -> String;
    fn template_dir(&self) -> &str;
    fn file_extension(&self) -> &str;
    async fn validate(&self, manifest: &ManifestDocument) -> Result<Vec<ValidationError>>;
}
```

### Adapters (Implementations) - NEW CODE

```rust
// adapters/yaml_parser.rs
pub struct YamlManifestParser;

#[async_trait]
impl ManifestParser for YamlManifestParser {
    async fn parse(&self, path: &Path) -> Result<ManifestDocument> {
        // Current ManifestDocument::from_path logic
    }
    async fn validate(&self, doc: &ManifestDocument) -> Result<()> {
        // Current validate logic
    }
}

// adapters/handlebars_renderer.rs
pub struct HandlebarsRenderer {
    templates_root: PathBuf,
}

#[async_trait]
impl TemplateRenderer for HandlebarsRenderer {
    async fn render(&self, template: &str, context: &Value) -> Result<String> {
        // Current render_template logic
    }
    async fn locate_template(&self, name: &str) -> Result<PathBuf> {
        // Current locate_template_file logic
    }
}

// adapters/fs_writer.rs
pub struct FileSystemWriter {
    dry_run: bool,
}

#[async_trait]
impl FileWriter for FileSystemWriter {
    async fn write(&self, changes: Vec<FileChange>) -> Result<()> {
        // Current execute_plan logic
    }
    async fn ensure_directory(&self, path: &Path) -> Result<()> {
        // Current ensure_directory logic
    }
}

// adapters/languages/dotnet.rs
pub struct DotNetAdapter {
    config: DotNetConfig,
}

#[async_trait]
impl LanguageAdapter for DotNetAdapter {
    fn language(&self) -> TargetLanguage {
        TargetLanguage::DotNet
    }
    fn map_type(&self, generic_type: &str) -> String {
        match generic_type {
            "decimal" => "decimal".to_string(),
            "string" => "string".to_string(),
            // ... type mappings
        }
    }
    fn template_dir(&self) -> &str {
        "templates/dotnet"
    }
    fn file_extension(&self) -> &str {
        "cs"
    }
    async fn validate(&self, manifest: &ManifestDocument) -> Result<Vec<ValidationError>> {
        // .NET-specific validations
        Ok(vec![])
    }
}
```

### Orchestrator (DIP) - REFACTORED CODE

```rust
// orchestrator.rs
pub struct ManifestOrchestrator {
    parser: Box<dyn ManifestParser>,
    renderer: Box<dyn TemplateRenderer>,
    writer: Box<dyn FileWriter>,
    adapter: Box<dyn LanguageAdapter>,
}

impl ManifestOrchestrator {
    pub fn new(
        parser: Box<dyn ManifestParser>,
        renderer: Box<dyn TemplateRenderer>,
        writer: Box<dyn FileWriter>,
        adapter: Box<dyn LanguageAdapter>,
    ) -> Self {
        Self { parser, renderer, writer, adapter }
    }

    pub async fn process(&self, config: ApplyConfig) -> Result<ApplySummary> {
        // 1. Parse manifest (uses ManifestParser trait)
        let manifest = self.parser.parse(&config.manifest_path).await?;
        
        // 2. Validate (uses ManifestParser + LanguageAdapter)
        self.parser.validate(&manifest).await?;
        self.adapter.validate(&manifest).await?;
        
        // 3. Collect render tasks
        let tasks = collect_render_tasks(&manifest)?;
        
        // 4. Render templates (uses TemplateRenderer trait)
        let mut changes = Vec::new();
        for task in tasks {
            let content = self.renderer.render(&task.template, &task.payload).await?;
            changes.push(FileChange {
                kind: ChangeKind::Create,
                path: task.destination,
                content,
            });
        }
        
        // 5. Write files (uses FileWriter trait)
        self.writer.write(changes.clone()).await?;
        
        // 6. Return summary
        Ok(ApplySummary {
            created: changes.len(),
            // ...
        })
    }
}
```

---

## 📁 Final Structure Summary

```
crates/commands/manifest/
├── Cargo.toml
├── src/
│   ├── lib.rs                         # Public API (ApplyArgs, run, etc.)
│   ├── orchestrator.rs                # Main workflow (DIP)
│   ├── ports/                         # 🎯 Traits (SOLID/DIP)
│   │   ├── mod.rs
│   │   ├── manifest_parser.rs
│   │   ├── template_renderer.rs
│   │   ├── file_writer.rs
│   │   └── language_adapter.rs
│   ├── adapters/                      # 🎯 Implementations
│   │   ├── mod.rs
│   │   ├── yaml_parser.rs
│   │   ├── handlebars_renderer.rs
│   │   ├── fs_writer.rs
│   │   └── languages/
│   │       ├── mod.rs
│   │       └── dotnet.rs
│   ├── models/                        # Data structures (~400 lines)
│   │   ├── mod.rs
│   │   ├── config.rs
│   │   ├── summary.rs
│   │   ├── document.rs
│   │   ├── meta.rs
│   │   ├── solution.rs
│   │   ├── project.rs
│   │   ├── policy.rs
│   │   ├── context.rs
│   │   ├── domain.rs
│   │   ├── application.rs
│   │   ├── api.rs
│   │   ├── enums.rs
│   │   ├── common.rs
│   │   └── templates.rs
│   ├── tasks/                         # Task building (~700 lines)
│   │   ├── mod.rs
│   │   ├── render_task.rs
│   │   ├── collector.rs
│   │   ├── locators.rs
│   │   ├── serializers.rs
│   │   └── builders/
│   │       ├── mod.rs
│   │       ├── domain.rs
│   │       ├── application.rs
│   │       └── api.rs
│   ├── files/                         # File operations (~200 lines)
│   │   ├── mod.rs
│   │   ├── changes.rs
│   │   ├── executor.rs
│   │   └── utils.rs
│   ├── stubs/                         # Code generation stubs (~80 lines)
│   │   ├── mod.rs
│   │   ├── solution.rs
│   │   └── project.rs
│   └── ui/                            # Interactive UI components (~100 lines)
│       ├── mod.rs
│       ├── manifest_picker.rs
│       ├── output_picker.rs
│       └── summary.rs
└── tests/
    ├── models_tests.rs
    ├── tasks_tests.rs
    ├── adapters_tests.rs
    ├── orchestrator_tests.rs
    ├── dry_run_tests.rs
    ├── apply_tests.rs
    └── integration_tests.rs
```

---

## 📊 Line Count Breakdown

| Module | Estimated Lines | Notes |
|--------|-----------------|-------|
| **lib.rs** | ~100 | Public API, entry points |
| **orchestrator.rs** | ~200 | Main workflow |
| **ports/** | ~150 | 4 traits (NEW CODE) |
| **adapters/** | ~300 | 4 implementations (REFACTORED) |
| **models/** | ~400 | 15 structs (MOVED) |
| **tasks/** | ~700 | Task building system (REFACTORED) |
| **files/** | ~200 | File operations (REFACTORED) |
| **stubs/** | ~80 | Code generation (MOVED) |
| **ui/** | ~100 | Interactive UI (NEW/MOVED) |
| **tests/** | ~250 | Test suite (REFACTORED) |
| **TOTAL** | **~2,480** | +~500 lines (ports/adapters abstractions) |

**Original:** 2,205 lines (monolithic)
**Target:** 2,480 lines (well-organized + SOLID abstractions)
**Growth:** +12% (acceptable for SOLID compliance)

---

## 🎯 Migration Priorities

### Phase 4.1: Models (2 hours)
1. Create `models/` directory structure
2. Move all structs with minimal changes
3. Add `mod.rs` re-exports
4. **Estimated:** 15 files, ~400 lines

### Phase 4.2: Ports (1 hour)
1. Define 4 traits (NEW CODE)
2. Document trait contracts
3. **Estimated:** 4 files, ~150 lines

### Phase 4.3: Tasks (4 hours)
1. Extract task building system
2. Split builders by layer (domain, application, api)
3. **Estimated:** 10 files, ~700 lines

### Phase 4.4: Files & Stubs (2 hours)
1. Extract file operations
2. Extract code stubs
3. **Estimated:** 7 files, ~280 lines

### Phase 4.5: Adapters (3 hours)
1. Implement YamlManifestParser
2. Implement HandlebarsRenderer
3. Implement FileSystemWriter
4. Implement DotNetAdapter
5. **Estimated:** 5 files, ~300 lines

### Phase 4.6: Orchestrator (2 hours)
1. Refactor main workflow
2. Inject dependencies (DIP)
3. **Estimated:** 1 file, ~200 lines

### Phase 4.7: Public API (1 hour)
1. Create lib.rs with clean exports
2. Maintain backward compatibility
3. **Estimated:** 1 file, ~100 lines

### Phase 4.8: Tests (3 hours)
1. Port existing tests
2. Add adapter tests
3. Add orchestrator tests
4. **Estimated:** 7 files, ~250 lines

### Phase 4.9: UI Components (2 hours)
1. Extract interactive UI logic
2. Create manifest/output pickers
3. **Estimated:** 4 files, ~100 lines

**Total Effort:** ~20 hours (2.5 days)

---

## ✅ Next Steps

1. ✅ Phase 0 Task 5: Analysis complete
2. 🔄 Commit Phase 0 inventory
3. ➡️ Begin Phase 1: Create workspace skeleton
4. ➡️ Phase 4: Execute manifest refactoring (20 hours)