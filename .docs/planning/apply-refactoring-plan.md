# Apply Command Refactoring Plan

**Version**: 1.0.0
**Date**: 2025-01-08
**Status**: Planning

## 🎯 Objective

Refactor the monolithic `apply.rs` (1,979 lines, 89 types/functions) into a well-organized module structure following the Codex pattern and Rust best practices.

---

## 📊 Current State Analysis

### File Size
- **Total lines**: 1,979 (65 KB)
- **Structs**: 35+
- **Enums**: 8+
- **Functions**: 46+
- **Complexity**: High - multiple concerns mixed

### Identified Concerns

1. **Manifest Parsing** (~400 lines)
   - `ManifestDocument`, `ManifestMeta`, `ManifestKind`
   - `ManifestContext`, `ManifestProject`, `ManifestPolicy`
   - Domain models: `ManifestAggregate`, `ManifestEntity`, `ManifestValueObject`
   - Application models: `ManifestUseCase`, `ManifestRepository`

2. **Template Rendering** (~300 lines)
   - `render_template()`, `RenderTask`, `RenderRule`
   - `locate_template_file()`, `resolve_destination()`
   - Handlebars integration

3. **Task Building** (~500 lines)
   - `build_enum_task()`, `build_value_object_task()`
   - `build_entity_task()`, `build_domain_event_task()`
   - `build_repository_task()`, `build_use_case_task()`
   - `build_endpoint_task()`

4. **Task Collection** (~300 lines)
   - `collect_render_tasks()`
   - `append_domain_tasks()`, `append_application_tasks()`, `append_api_tasks()`
   - Locator structs: `ValueObjectLocator`, `EntityLocator`, etc.

5. **File Operations** (~200 lines)
   - `execute_plan()`, `FileChange`, `FileChangeKind`
   - `ensure_directory()`, collision detection

6. **Utilities** (~150 lines)
   - `normalize_line_endings()`, `to_lower_camel()`
   - `lookup_placeholder()`, placeholder resolution

7. **Entry Point** (~100 lines)
   - `run()`, `execute_apply()`, `apply_sync()`
   - `ApplyArgs`, `ApplyConfig`, `ApplySummary`

---

## 🎨 Target Architecture

Following the Codex pattern, create a dedicated **`apply/`** module with clear separation of concerns:

```
commands/
├── Cargo.toml
├── src/
│   ├── lib.rs                    # Re-export apply
│   ├── apply/                    # Apply command module
│   │   ├── mod.rs                # Public API + orchestration
│   │   ├── args.rs               # CLI arguments
│   │   ├── summary.rs            # ApplySummary
│   │   ├── config.rs             # ApplyConfig
│   │   ├── manifest/             # Manifest parsing
│   │   │   ├── mod.rs
│   │   │   ├── document.rs       # ManifestDocument
│   │   │   ├── meta.rs           # ManifestMeta, ManifestKind
│   │   │   ├── solution.rs       # ManifestSolution
│   │   │   ├── project.rs        # ManifestProject, ManifestProjectKind
│   │   │   ├── context.rs        # ManifestContext
│   │   │   ├── domain.rs         # Aggregate, Entity, ValueObject, DomainEvent
│   │   │   ├── application.rs    # UseCase, Repository
│   │   │   ├── templates.rs      # ManifestTemplates, TemplateMapping
│   │   │   ├── policy.rs         # ManifestPolicy, CollisionPolicy
│   │   │   └── enums.rs          # ManifestEnum, EnumValue
│   │   ├── tasks/                # Task building & collection
│   │   │   ├── mod.rs
│   │   │   ├── render_task.rs    # RenderTask struct
│   │   │   ├── collector.rs      # collect_render_tasks()
│   │   │   ├── locators.rs       # Locator structs
│   │   │   ├── builders/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── domain.rs     # build_*_task for domain
│   │   │   │   ├── application.rs # build_*_task for application
│   │   │   │   └── api.rs        # build_endpoint_task
│   │   │   └── serializers.rs    # serialize_field, serialize_repository_method
│   │   ├── render/               # Template rendering
│   │   │   ├── mod.rs
│   │   │   ├── engine.rs         # render_template(), Handlebars setup
│   │   │   ├── resolver.rs       # locate_template_file(), resolve_destination()
│   │   │   └── helpers.rs        # normalize_line_endings, to_lower_camel
│   │   ├── files/                # File operations
│   │   │   ├── mod.rs
│   │   │   ├── changes.rs        # FileChange, FileChangeKind
│   │   │   ├── executor.rs       # execute_plan()
│   │   │   └── utils.rs          # ensure_directory()
│   │   └── stubs/                # Code generation stubs
│   │       ├── mod.rs
│   │       ├── solution.rs       # build_solution_stub()
│   │       └── project.rs        # build_project_stub()
│   ├── check.rs
│   ├── list.rs
│   ├── new.rs
│   └── ...
└── tests/
    ├── apply/
    │   ├── mod.rs
    │   ├── manifest_tests.rs
    │   ├── tasks_tests.rs
    │   ├── render_tests.rs
    │   └── integration_tests.rs
    └── ...
```

---

## 📋 Module Responsibilities

### 1. `apply/mod.rs` (Orchestration)
- Public API: `pub async fn run(args: ApplyArgs) -> ExitStatus`
- Orchestrates: parse → collect → render → execute
- ~100 lines

### 2. `apply/args.rs`
- `ApplyArgs` struct with clap derives
- ~30 lines

### 3. `apply/summary.rs`
- `ApplySummary` struct and `print()` method
- ~60 lines

### 4. `apply/config.rs`
- `ApplyConfig` struct
- `resolve_manifest_path()`, `resolve_output_root()`
- ~80 lines

### 5. `apply/manifest/` (8 files, ~600 lines total)
- Each file: one primary concern
- Clean separation: meta, domain, application, templates
- All serde derives and validation

### 6. `apply/tasks/` (6 files, ~700 lines total)
- `collector.rs`: Main task collection logic
- `builders/`: Domain/application/API task builders
- `locators.rs`: Find artifacts across contexts
- `serializers.rs`: Convert structs to JSON

### 7. `apply/render/` (4 files, ~300 lines total)
- `engine.rs`: Handlebars setup and rendering
- `resolver.rs`: Template/destination path resolution
- `helpers.rs`: String utilities

### 8. `apply/files/` (3 files, ~200 lines total)
- `changes.rs`: Change tracking types
- `executor.rs`: Write files, collision detection
- `utils.rs`: Directory creation

### 9. `apply/stubs/` (3 files, ~80 lines total)
- Generate .sln and .csproj stubs

---

## 🔄 Migration Strategy

### Phase 1: Setup Module Structure (Day 1)
**Goal**: Create empty module structure with proper exports

**Tasks**:
1. Create `commands/src/apply/` directory
2. Create all subdirectories: `manifest/`, `tasks/`, `render/`, `files/`, `stubs/`
3. Create all `mod.rs` files with TODO comments
4. Update `commands/src/lib.rs` to export `apply` module
5. Verify project compiles

**Validation**:
```bash
cargo build -p commands
```

### Phase 2: Extract Args, Config, Summary (Day 1)
**Goal**: Move simple, standalone types

**Tasks**:
1. Create `apply/args.rs` with `ApplyArgs` + Default impl
2. Create `apply/config.rs` with `ApplyConfig` + resolver functions
3. Create `apply/summary.rs` with `ApplySummary` + print method
4. Update `apply/mod.rs` to re-export these types
5. Update original `apply.rs` imports

**Validation**:
```bash
cargo test -p commands -- apply
```

### Phase 3: Extract Manifest Types (Day 2)
**Goal**: Break manifest parsing into logical modules

**Tasks**:
1. `manifest/meta.rs`: `ManifestMeta`, `ManifestKind`
2. `manifest/solution.rs`: `ManifestSolution`, `ManifestGuards`
3. `manifest/project.rs`: `ManifestProject`, `ManifestProjectKind`
4. `manifest/context.rs`: `ManifestContext`
5. `manifest/domain.rs`: `ManifestAggregate`, `ManifestEntity`, `ManifestValueObject`, `ManifestDomainEvent`
6. `manifest/application.rs`: `ManifestUseCase`, `ManifestRepository`, `ManifestRepositoryMethod`
7. `manifest/templates.rs`: `ManifestTemplates`, `TemplateMapping`, `ManifestRender`, `RenderRule`
8. `manifest/policy.rs`: `ManifestPolicy`, `ManifestCollisionPolicy`, `MissingProjectAction`
9. `manifest/enums.rs`: `ManifestEnum`, `ManifestEnumValue`
10. `manifest/document.rs`: `ManifestDocument` (depends on all above)
11. `manifest/mod.rs`: Re-export all types

**Validation**:
```bash
cargo test -p commands -- manifest
```

### Phase 4: Extract File Operations (Day 2)
**Goal**: Isolate file system concerns

**Tasks**:
1. `files/changes.rs`: `FileChange`, `FileChangeKind`
2. `files/utils.rs`: `ensure_directory()`
3. `files/executor.rs`: `execute_plan()`
4. `files/mod.rs`: Re-export

**Validation**:
```bash
cargo test -p commands -- files
```

### Phase 5: Extract Rendering (Day 3)
**Goal**: Separate template rendering logic

**Tasks**:
1. `render/helpers.rs`: `normalize_line_endings()`, `to_lower_camel()`, `lookup_placeholder()`
2. `render/resolver.rs`: `locate_template_file()`, `resolve_destination()`
3. `render/engine.rs`: `render_template()`, Handlebars setup
4. `render/mod.rs`: Re-export

**Validation**:
```bash
cargo test -p commands -- render
```

### Phase 6: Extract Stubs (Day 3)
**Goal**: Move code generation helpers

**Tasks**:
1. `stubs/solution.rs`: `build_solution_stub()`
2. `stubs/project.rs`: `build_project_stub()`, `build_project_payload()`
3. `stubs/mod.rs`: Re-export

**Validation**:
```bash
cargo test -p commands -- stubs
```

### Phase 7: Extract Task System (Day 4-5)
**Goal**: Break down task building/collection (largest refactor)

**Tasks**:
1. `tasks/render_task.rs`: `RenderTask` struct
2. `tasks/locators.rs`: All `*Locator` structs + `find_*()` functions
3. `tasks/serializers.rs`: `serialize_field()`, `serialize_repository_method()`
4. `tasks/builders/domain.rs`: `build_enum_task()`, `build_value_object_task()`, `build_entity_task()`, `build_domain_event_task()`
5. `tasks/builders/application.rs`: `build_repository_task()`, `build_use_case_task()`
6. `tasks/builders/api.rs`: `build_endpoint_task()`
7. `tasks/builders/mod.rs`: Re-export
8. `tasks/collector.rs`: `collect_render_tasks()`, `append_domain_tasks()`, `append_application_tasks()`, `append_api_tasks()`, `select_contexts()`
9. `tasks/mod.rs`: Re-export all

**Validation**:
```bash
cargo test -p commands -- tasks
```

### Phase 8: Orchestration (Day 6)
**Goal**: Update main entry point to use new modules

**Tasks**:
1. Update `apply/mod.rs` with:
   - `pub async fn run(args: ApplyArgs) -> ExitStatus`
   - `async fn execute_apply()` orchestration
   - `fn apply_sync()` core logic using new modules
2. Remove old `apply.rs` (now replaced by `apply/mod.rs`)
3. Update `lib.rs` exports

**Validation**:
```bash
cargo test -p commands
cargo build -p commands
```

### Phase 9: Testing (Day 7)
**Goal**: Ensure all tests pass and add missing coverage

**Tasks**:
1. Move existing tests from `tests/commands_tests.rs` to `tests/apply/`
2. Add unit tests for each new module
3. Add integration tests in `tests/apply/integration_tests.rs`
4. Test dry-run mode
5. Test all manifest types (artifact, feature, layer)

**Validation**:
```bash
cargo test -p commands --all-features
cargo test -p commands -- --nocapture
```

### Phase 10: Documentation (Day 7)
**Goal**: Document all public APIs

**Tasks**:
1. Add module-level documentation to all `mod.rs` files
2. Add doc comments to all public structs/enums
3. Add doc examples to key functions
4. Update main README.md with new structure
5. Generate docs: `cargo doc --open -p commands`

**Validation**:
```bash
cargo doc -p commands --no-deps
```

---

## 🎯 Success Criteria

### Code Quality
- ✅ No file > 400 lines
- ✅ Each module has single responsibility
- ✅ Clear dependency hierarchy (no circular dependencies)
- ✅ All public types documented

### Testing
- ✅ All existing tests pass
- ✅ >80% code coverage
- ✅ Unit tests for all new modules
- ✅ Integration tests for full workflow

### Performance
- ✅ No performance regression
- ✅ Memory usage unchanged
- ✅ Compilation time similar or better

### Maintenance
- ✅ Easy to locate code for specific concerns
- ✅ Clear module boundaries
- ✅ Simple to add new artifact types
- ✅ Follows Codex pattern

---

## 📐 Design Principles

### 1. Separation of Concerns
- **Manifest**: Data structures (serde models)
- **Tasks**: Business logic (task building)
- **Render**: Template processing
- **Files**: I/O operations

### 2. Dependency Direction
```
mod.rs (orchestration)
  ├─> config
  ├─> manifest (data)
  ├─> tasks (depends on manifest)
  ├─> render (depends on tasks)
  └─> files (depends on render)
```

### 3. Error Handling
- Use `crate::Result` and `crate::CommandError`
- Add context with `.map_err()` at module boundaries
- Propagate errors with `?`

### 4. Testing Strategy
- **Unit tests**: Per-module in `tests/apply/*_tests.rs`
- **Integration tests**: Full workflow in `tests/apply/integration_tests.rs`
- Use fixtures in `tests/fixtures/manifests/`

---

## 🔧 Codex Pattern Compliance

### Each Module Must Have
1. `mod.rs` with public API
2. Clear re-exports
3. Internal types private by default
4. Documentation comments

### Naming Conventions
- Files: `snake_case.rs`
- Modules: `snake_case`
- Structs: `PascalCase`
- Functions: `snake_case`
- Constants: `SCREAMING_SNAKE_CASE`

### Module Structure
```rust
// mod.rs
//! Module documentation

mod internal_module; // Private
pub mod submodule;   // Public sub-module

pub use internal_module::PublicType;

pub fn public_api() -> Result<()> {
    // Implementation
}
```

---

## 📅 Timeline

**Total**: 7 days (conservative estimate with testing)

| Phase | Duration | Status |
|-------|----------|--------|
| Phase 1: Setup | 2 hours | ⏳ Not Started |
| Phase 2: Args/Config/Summary | 2 hours | ⏳ Not Started |
| Phase 3: Manifest Types | 1 day | ⏳ Not Started |
| Phase 4: File Operations | 4 hours | ⏳ Not Started |
| Phase 5: Rendering | 4 hours | ⏳ Not Started |
| Phase 6: Stubs | 2 hours | ⏳ Not Started |
| Phase 7: Task System | 2 days | ⏳ Not Started |
| Phase 8: Orchestration | 4 hours | ⏳ Not Started |
| Phase 9: Testing | 1 day | ⏳ Not Started |
| Phase 10: Documentation | 4 hours | ⏳ Not Started |

---

## 🚀 Next Steps

1. **Review this plan** with stakeholders
2. **Create feature branch**: `feature/refactor-apply-command`
3. **Start Phase 1**: Module structure setup
4. **Commit after each phase** with clear commit messages

---

## 📚 References

- **Codex Pattern**: `tools/codex/codex-rs/` (reference implementation)
- **Rust API Guidelines**: https://rust-lang.github.io/api-guidelines/
- **Cargo Workspaces**: https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html
- **Module System**: https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html

---

## 🔑 Key Principles

1. **Incremental**: Refactor in small, testable steps
2. **Safe**: All tests must pass after each phase
3. **Documented**: Update docs as you go
4. **Reversible**: Git commits allow rollback if needed
5. **Maintainable**: Follow Codex pattern strictly
6. **Tested**: Add tests before removing old code
7. **Reviewed**: Get code review after major phases
8. **Pragmatic**: Don't over-engineer, keep it simple