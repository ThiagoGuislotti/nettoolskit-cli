# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Decision log centralized in `CHANGELOG.md` as the single source of truth for architecture/engineering decisions.

### Decisions
- **DEC-0001 (Accepted, 2026-02-28): Modular workspace boundaries**
  - Keep a modular Cargo workspace with clear crate responsibilities:
    - `core` (shared models/utilities)
    - `ui` (terminal rendering/interaction)
    - `otel` (telemetry/tracing setup)
    - `orchestrator` (execution flow)
    - `commands/*` (domain command implementations)
    - `cli` (binary entrypoint + interactive loop)
  - Enforce dependency direction from higher-level crates to lower-level crates only.
- **DEC-0002 (Accepted, 2026-02-28): Terminal rendering without alternate screen**
  - Keep rendering in the main terminal buffer (no alternate screen).
  - Preserve output/history on `/quit` and `Ctrl+C`.
  - Use resize debounce and explicit clear/reflow ordering for stability.
  - Keep cursor explicitly visible/blinking in prompt states.
- **DEC-0003 (Accepted, 2026-02-28): Quality gates and lint policy**
  - Hard gates:
    - `cargo fmt --all -- --check`
    - `cargo clippy --workspace --all-targets --all-features -- -D warnings`
    - `cargo test --workspace --all-targets`
    - dependency security audit (`cargo audit` / `cargo-deny` in CI)
  - Lint policy:
    - `clippy::all` as blocking baseline
    - `pedantic`, `nursery`, and `cargo` as advisory by default
- **DEC-0004 (Accepted, 2026-03-01): Hybrid observability model with optional OTLP traces**
  - Keep custom in-process metrics API for fast/local CLI usage.
  - Add optional OpenTelemetry trace export via OTLP, enabled only when endpoint env vars are set.
  - Support OTLP gRPC and HTTP/protobuf protocols with configurable timeout.
- **DEC-0005 (Accepted, 2026-03-01): Correlation ID at session and command boundaries**
  - Add lightweight correlation IDs for interactive session, non-interactive execution, and command dispatch spans.
  - Keep format process-local and dependency-light (`prefix + timestamp + sequence`) for CLI performance.
- **DEC-0006 (Accepted, 2026-03-01): Runtime metrics taxonomy for command operations**
  - Standardize counters, gauges, and timing names for command-level observability.
  - Track command latency, success/error/cancellation rates, and non-command text input volume.
- **DEC-0007 (Accepted, 2026-03-01): Incident response playbook as operational baseline**
  - Establish a single operational runbook for severity classification, triage, mitigation, and post-incident review.
  - Include scenario-specific troubleshooting for terminal resize/layout, command error/cancellation spikes, and OTLP export failures.
- **DEC-0008 (Accepted, 2026-03-01): OTLP metrics export and explicit telemetry shutdown**
  - Mirror in-process runtime metrics to OTLP when metrics endpoint env vars are configured.
  - Keep in-process metrics as the source API while enabling centralized metric pipelines.
  - Trigger explicit telemetry shutdown before process exit to flush traces/metrics in short-lived CLI runs.
- **DEC-0009 (Accepted, 2026-03-01): Pin Rust toolchain to MSRV 1.85.0**
  - Adopt `rust-toolchain.toml` with `1.85.0` to stabilize local and CI behavior.
  - Align MSRV policy with current dependency graph requirements (lockfile v4 and edition2024 dependencies).
- **DEC-0010 (Accepted, 2026-03-01): Release must publish dual-format SBOM assets**
  - Generate SBOM for every tagged release in both CycloneDX and SPDX JSON formats.
  - Publish SBOM files as release assets for supply-chain transparency and auditability.
- Historical ADR files from `docs/adr/` were retired and consolidated into this section.

### Changed
- Workspace lint policy adjusted to keep CI gate strict on `clippy::all` with `-D warnings`.
- `CHANGELOG.md` aligned to Keep a Changelog structure with an explicit `Unreleased` section.
- `crates/otel` migrated to a hybrid model: optional OTLP trace export plus existing in-process metrics.
- OTLP dependencies were added to workspace/crate manifests (`tracing-opentelemetry`, `opentelemetry`, `opentelemetry_sdk`, `opentelemetry-otlp`).
- Correlation IDs were introduced and attached to tracing spans at session/execution/command boundaries.
- Runtime/business metrics were defined in orchestrator with stable names for latency, error rate, and cancellation rate.
- Incident response and troubleshooting playbook was added under `docs/operations/` and linked from project README.
- OpenTelemetry support now includes optional OTLP metrics export (`OTEL_EXPORTER_OTLP_METRICS_*` / `NTK_OTLP_METRICS_*`) in addition to trace export.
- OTLP env resolution now supports signal-specific overrides for traces and metrics with shared fallbacks.
- Rust toolchain is now pinned via `rust-toolchain.toml` and CI MSRV check moved to `1.85.0`.
- Release pipeline now generates and publishes SBOM assets in CycloneDX and SPDX formats.

### Fixed
- Terminal resize stability improvements to avoid duplicated/overlapped UI content on rapid terminal/font-size changes.
- Interactive terminal behavior now preserves visible shell output/history on `/quit` and `Ctrl+C` (no alternate screen wipe).
- Cursor visibility/blinking handling improved in interactive prompt flow.
- Environment-variable race flake fixed in feature-detection tests by synchronizing tests that mutate `NTK_USE_*`.
- OpenTelemetry subscriber layering/type mismatch fixed in `otel` tracing setup (paths with/without OTLP now compile and initialize correctly).
- Non-interactive CLI now calls telemetry shutdown before `process::exit`, preventing loss of buffered OTLP data.

### Security
- Dependency hardening and audit cleanups (`cargo audit` baseline cleaned for current lockfile updates).

### Testing
- Quality validation passes in workspace:
  - `cargo fmt --all -- --check`
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  - `cargo test --workspace --all-targets`
  - `cargo doc --workspace --no-deps`
- Additional validation for OTLP migration:
  - `cargo clippy -p nettoolskit-otel --all-targets --all-features -- -D warnings`
  - `cargo test -p nettoolskit-otel --all-targets`

## [1.0.0] - 2025-01-04

### Changed - Major Architecture Refactoring ✅

#### Created Orchestrator Layer
- **New Crate**: `crates/orchestrator/` for command orchestration
  - Centralized command dispatch and routing
  - Async execution with progress tracking
  - Command models (MainAction, ExitStatus)
  - Clean separation from UI and command implementations

#### CLI Layer Cleanup
- **CLI Now UI-Only**: `crates/cli/` simplified to terminal interface
  - Removed: execution/, models/, handlers/ (moved to orchestrator)
  - Kept: display.rs, events.rs, input.rs (UI concerns only)
  - Dependencies reduced: removed strum, walkdir, inquire, regex, futures, handlebars
  - Clear responsibility: user interaction and display

#### Command Structure Reorganization
- **Removed**: `crates/commands/management/` (deprecated, replaced by orchestrator)
- **Help Command**: Moved from `cli/src/handlers/help.rs` to `crates/commands/help/`
  - Created dedicated `nettoolskit-help` crate
  - Structure matches other commands (manifest, translate)
- **Commands Crate**: Now pure aggregator of command implementations
  - Simplified to re-export help, manifest, translate
  - No orchestration logic

#### Architecture Benefits
- **Clear Separation of Concerns**:
  - CLI: User interface (input, display, events)
  - Orchestrator: Command routing and execution
  - Commands: Business logic implementations
- **Reduced Coupling**: Each layer has minimal dependencies
- **Easier Testing**: Isolated test suites per layer
- **Better Scalability**: Easy to add new commands without touching CLI

#### Testing
- **8 tests passing** across new structure:
  - Orchestrator: 4 tests (execution, progress tracking)
  - Help: 2 tests (discovery handlers)
  - Manifest: 2 tests (apply handlers)
- Clean workspace build with proper dependency graph

## [0.3.0] - 2025-11-10

### Added - Phase 3: Templating Engine Refactoring ✅

#### Architecture
- **Commands Dispatcher**: Implemented thin orchestrator pattern at `crates/commands/`
  - Feature aggregation: Re-exports sub-features as modules
  - Command routing: Unified entry point for CLI commands
  - Error handling: `CommandError` and `Result` types
  - Async support: `AsyncCommandExecutor` with progress tracking

- **Templating Feature**: Relocated to `crates/commands/templating/` (sub-crate)
  - **Strategy Pattern**: Language-specific rendering strategies (6 languages)
  - **Factory Pattern**: `LanguageStrategyFactory` with dynamic strategy creation
  - **Async APIs**: Non-blocking template operations with `tokio`
  - **Caching**: Template caching with 2000x performance improvement
  - **Parallelism**: Batch rendering with 10x speedup via `rayon`
  - **Languages Supported**: DotNet, Java, Go, Python, Rust, Clojure

#### Features
- Template caching with TTL (Time-To-Live) configuration
- Parallel batch rendering for multiple templates
- Path normalization for cross-platform compatibility
- Filename-based template discovery
- TODO marker insertion and detection in generated code

#### Testing
- **33 tests total** (100% passing):
  - 27 integration tests across 6 test files
  - 6 documentation tests
- Test categories:
  - `strategy_tests.rs`: Language strategy validation (6 tests)
  - `factory_tests.rs`: Factory pattern and detection (6 tests)
  - `engine_tests.rs`: Template rendering engine (5 tests)
  - `resolver_tests.rs`: Path resolution and caching (7 tests)
  - `batch_tests.rs`: Parallel batch operations (3 tests)
  - Doc tests: API usage examples (6 tests)

#### Performance
- **Caching**: 2000x speedup for repeated template access
- **Parallelism**: 10x speedup for batch operations
- **Async**: Non-blocking I/O for file operations

#### Documentation
- Created `crates/commands/README.md`: Commands architecture guide
- Updated workspace `README.md`: Architecture overview with 10 crates
- Inline documentation: Comprehensive rustdoc comments
- Usage examples: Doc tests demonstrating API usage

### Changed

#### Structure
- **Relocated**: `crates/templating/` → `crates/commands/templating/`
  - Aligns with architecture plan: features as sub-crates of commands
  - Commands acts as thin dispatcher for features
  - Proper separation of concerns (SOLID principles)

#### Workspace
- Updated `Cargo.toml` workspace members:
  - Removed: `"crates/templating"`, `"crates/commands-old"`
  - Added: `"crates/commands"`, `"crates/commands/templating"`
- Fixed all dependency paths in referencing crates
- Re-exported templating through commands public API

#### API
- **Public API**: `nettoolskit_commands::templating` module
- **Error Types**: Unified `CommandError` across all features
- **Async Executor**: `AsyncCommandExecutor` for long-running operations

### Technical Details

#### Workspace Configuration
```toml
[workspace]
members = [
    "crates/cli",
    "crates/core",
    "crates/commands",
    "crates/commands/templating",  # Feature sub-crate
    "crates/ui",
    "crates/otel",
    "crates/ollama",
    "crates/shared/async-utils",
    "crates/shared/file-search",
    "crates/shared/utils",
]
```

#### Commands Public API
```rust
// Re-export templating feature
pub use nettoolskit_templating as templating;

// Access via commands
use nettoolskit_commands::templating::{
    TemplateEngine,
    LanguageStrategyFactory,
    BatchRenderer,
};
```

#### Test Results
```bash
cargo test -p nettoolskit-templating
# Running 33 tests
# test result: ok. 33 passed; 0 failed
```

### Migration Notes

#### For Contributors
1. **New Location**: Templating code now at `crates/commands/templating/`
2. **Import Path**: Use `nettoolskit_commands::templating` instead of `nettoolskit_templating`
3. **Tests**: Run `cargo test -p nettoolskit-templating` for feature tests
4. **Workspace**: Build with `cargo check --workspace` to verify all references

#### For Maintainers
- Phase 3 objectives: ✅ Complete
- Architecture alignment: ✅ Confirmed
- Test coverage: ✅ 100% (33/33 tests passing)
- Performance: ✅ Validated (2000x cache, 10x parallel)
- Documentation: ✅ Updated

---

## [0.4.0] - 2025-11-12

### Added - Phase 4: Manifest Feature ✅

#### Architecture
- **Manifest Feature**: Implemented at `crates/commands/manifest/` (sub-crate)
  - **Domain Models**: Complete manifest structure in `models.rs` (469 lines)
  - **Parser**: YAML parsing and validation in `parser.rs`
  - **Executor**: Orchestration engine in `executor.rs`
  - **Task Generation**: Layer-based task builders (domain, application, api)
  - **File Operations**: Collision detection and file management
  - **Rendering Integration**: Delegates to templating crate

#### Features
- **YAML-based Configuration**: Define entire code generation workflows declaratively
- **DDD Support**: First-class support for bounded contexts, aggregates, entities, value objects, domain events
- **Multiple Apply Modes**: Feature, Artifact, and Layer-based generation
- **Template Orchestration**: 55 templates mapped to 7 ArtifactKind types
- **Dry-Run Mode**: Preview changes before applying
- **Collision Detection**: Configurable policies (fail/overwrite)
- **Async Execution**: Non-blocking operations with progress tracking

#### Artifact Kinds
- **ValueObject**: DDD Value Objects
- **Entity**: DDD Entities
- **DomainEvent**: Domain Events
- **RepositoryInterface**: Repository Interfaces
- **EnumType**: Enumerations
- **UseCaseCommand**: CQRS Commands/Queries
- **Endpoint**: API Controllers/Endpoints

#### Templates
- **55 Templates** in `templates/dotnet/`:
  - Domain layer: entity.hbs, value-object.hbs, domain-event.hbs, enum.hbs
  - Application layer: command.hbs, command-handler.hbs, query.hbs, query-handler.hbs
  - API layer: controller.hbs, endpoint.hbs
  - Infrastructure layer: repository-interface.hbs, repository-impl.hbs

#### Testing
- **87 tests total** (100% passing):
  - 11 async tests: Async execution, timeouts, concurrency
  - 17 error tests: All error types, propagation, display
  - 8 executor tests: Configuration, execution, dry-run
  - 10 file tests: Create, update, collision handling
  - 7 integration tests: End-to-end workflows with real templates
  - 15 model tests: Domain models, serialization
  - 10 parser tests: YAML parsing, validation
  - 8 task tests: Task generation, filtering

#### Documentation
- Created `crates/commands/manifest/README.md`: Comprehensive feature guide (1000+ lines)
- Created test fixtures in workspace `tests/fixtures/`:
  - `ntk-manifest.yml`: Complete DDD example (285 lines)
  - `ntk-manifest-minimal.yml`: Minimal test manifest (50 lines)
  - `ntk-manifest-domain.yml`: Domain-focused example
  - `templates/`: Copy of workspace templates for integration tests
- Inline documentation: Comprehensive rustdoc comments

### Changed

#### Integration
- **Commands Dispatcher**: Added manifest re-export in `crates/commands/src/lib.rs`
  - Public API: `nettoolskit_commands::manifest`
  - Re-exports: `ManifestExecutor`, `ExecutionConfig`, `ManifestDocument`, `ApplyModeKind`

#### Testing
- **Integration Test**: Enabled `test_integration_full_workflow_with_templates`
  - Removed `#[ignore]` attribute
  - Uses test fixtures from workspace `tests/fixtures/` (with templates nearby)
  - Validates full workflow with dry-run mode
  - Now passing (was 1 ignored, now 7 passing)

### Technical Details

#### Manifest Structure
```yaml
apiVersion: ntk/v1
kind: solution

meta:
  name: my-project

solution:
  root: ./src
  slnFile: MyProject.sln

conventions:
  namespaceRoot: MyProject
  targetFramework: net9.0
  policy:
    collision: fail

templates:
  mapping:
    - artifact: entity
      template: templates/dotnet/src/domain/Entities/entity.hbs
      dst: Domain/Entities/{name}.cs

contexts:
  - name: Orders
    aggregates:
      - name: Order
        entities:
          - name: OrderItem

apply:
  mode: feature
  feature:
    context: Orders
```

#### Commands Public API
```rust
// Re-export manifest feature
pub use nettoolskit_manifest as manifest;

// Access via commands
use nettoolskit_commands::manifest::{
    ManifestExecutor,
    ExecutionConfig,
    ManifestDocument,
};
```

#### Test Results
```bash
cargo test -p nettoolskit-manifest
# Running 87 tests
# test result: ok. 87 passed; 0 failed
```

#### Apply Modes

**Feature Mode** - Generate all artifacts for a bounded context:
```yaml
apply:
  mode: feature
  feature:
    context: Orders
    include: [entity, value-object, usecase-command]
```

**Artifact Mode** - Generate specific artifacts by type:
```yaml
apply:
  mode: artifact
  artifact:
    kind: entity
    context: Orders
    name: OrderItem
```

**Layer Mode** - Generate by architectural layer:
```yaml
apply:
  mode: layer
  layer:
    include: [domain, application]
```

### Migration Notes

#### For Contributors
1. **New Location**: Manifest code now at `crates/commands/manifest/`
2. **Import Path**: Use `nettoolskit_commands::manifest` instead of direct dependency
3. **Tests**: Run `cargo test -p nettoolskit-manifest` for feature tests
4. **Test Fixtures**: Located in workspace `tests/fixtures/` with templates copy

#### For Maintainers
- Phase 4 objectives: ✅ Complete
- Architecture alignment: ✅ Confirmed (Clean Architecture with ports/adapters)
- Test coverage: ✅ 100% (87/87 tests passing)
- Template coverage: ✅ 55 templates for 7 artifact kinds
- Documentation: ✅ Complete (README.md + examples)

### Performance
- **Async Operations**: Non-blocking I/O for file operations
- **Template Caching**: Shared with templating crate (2000x speedup)
- **Task Generation**: Efficient filtering and aggregation

---

## [0.2.0] - 2025-11-08

### Added
- Interactive TUI with command palette
- Async command execution with progress tracking
- File search utilities with glob pattern support
- OpenTelemetry integration for observability

### Changed
- Refactored UI components using ratatui
- Improved error handling with thiserror

---

## [0.1.0] - 2025-11-01

### Added
- Initial CLI implementation
- Basic template rendering
- Command processing infrastructure
- Core types and utilities

---

[Unreleased]: https://github.com/ThiagoGuislotti/NetToolsKit/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/ThiagoGuislotti/NetToolsKit/releases/tag/v1.0.0
[0.4.0]: https://github.com/ThiagoGuislotti/NetToolsKit/releases/tag/v0.4.0
[0.3.0]: https://github.com/ThiagoGuislotti/NetToolsKit/releases/tag/v0.3.0
[0.2.0]: https://github.com/ThiagoGuislotti/NetToolsKit/releases/tag/v0.2.0
[0.1.0]: https://github.com/ThiagoGuislotti/NetToolsKit/releases/tag/v0.1.0
