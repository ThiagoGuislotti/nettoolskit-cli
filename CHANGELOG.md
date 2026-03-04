# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Decision log centralized in `CHANGELOG.md` as the single source of truth for architecture/engineering decisions.
- Added `COMPATIBILITY.md` as the official compatibility matrix and support policy for release artifacts.
- Added release verification runbook (`docs/operations/release-artifact-verification.md`) for checksum and keyless cosign validation of published artifacts.
- Added manual release verification workflow (`.github/workflows/release-verify.yml`) for tag-based validation of published assets.
- Added formal support lifecycle and EOL table in `COMPATIBILITY.md` with dated maintenance windows.
- Added SBOM verification coverage (signature + metadata sanity) to the manual release verification workflow.
- Added deterministic PowerShell-based compatibility lifecycle validation in the release workflow to enforce EOL policy semantics.
- Added shared-script pattern to release validation: workflow clones `copilot-instructions` and uses shared lifecycle validator when available (inline fallback retained).
- Added `rustyline`-based CLI input path with persisted history and command auto-complete (with fallback to the legacy input loop if initialization fails).
- Added multiline input support in CLI (`rustyline` validator + explicit trailing `\` continuation marker).
- Added interactive `FilePicker` component with fuzzy filtering, regex mode (`re:`), literal mode (`lit:`), and keyboard navigation for manifest file selection.
- Added interactive `StatusBar` component with mode indicator, bounded notifications queue, command outcome counters, and runtime usage summary.
- Added interactive `HistoryViewer` component with pagination, indexed entry rendering, and case-insensitive filtering.
- Added interactive input syntax highlighting in `rustyline` for commands/flags plus lexical styles for Rust, C#, JavaScript, and TypeScript lines.
- Added `tree-sitter` parser integration for Rust, C#, JavaScript, and TypeScript token-aware interactive highlighting.
- Added cross-platform desktop attention notifications in interactive mode with configurable runtime toggle (`attention_desktop_notification`).
- Added async manifest aliases (`/new-async`, `/render-async`, `/apply-async`) with progress streaming in command execution.
- Added orchestrator runtime command cache module with LRU ordering, per-command TTL, and memory-budget eviction controls.
- Added dedicated Criterion benchmark target (`command_cache`) covering runtime cache insert/hit/miss/eviction paths.
- Added predictive slash-command hints in interactive `rustyline` input for faster command completion guidance.
- Added runtime configuration support for predictive input hints (`predictive_input`) with file/env and `/config` command integration.
- Added orchestrator plugin foundation with in-process registry and safe before/after command hook pipeline.
- Added bounded interactive error-recovery flow for input backends with retry budget and backoff before failing session startup/loop.
- Added panic-safe async task wrapper in CLI interactive runtime to recover from command/text task panics without crashing the full session.
- Added rich CLI state module (`cli::state`) with serializable `CliState`, typed history entries, and shared `Arc<RwLock<_>>` handle for session-scoped state coordination.
- Added local-only interactive session persistence with JSON snapshots (save/load/list/prune) under the OS app data directory and latest-session auto-resume support.
- Added startup local session resume picker (when multiple local snapshots exist), built on `CommandPalette`.
- Added terminal frame scheduler runtime with coalesced frame requests, 60 FPS rate limiting, and async poll-timeout adaptation helpers.
- Added language-aware fenced code block highlighting in Markdown renderer (Rust, C#, JavaScript, TypeScript, JSON, TOML, Bash, PowerShell).
- Added dedicated AI E2E integration tests in orchestrator for `/ai plan`, `/ai apply --dry-run`, safety blocking of mutating apply without approval, and free-text alias routing to AI flows.
- Added explicit CI `AI Gate` job to enforce AI-specific E2E/safety/resilience test slices.
- Added shared runtime contracts in `nettoolskit-core` for dual-mode execution planning (`RuntimeMode`, `TaskIntentKind`, `TaskIntent`, `TaskExecutionStatus`, `TaskAuditEvent`).
- Added embedded background worker runtime in orchestrator for service-mode task execution with bounded queue, concurrency limits, retry backoff, cancellation, and task audit trail.
- Added `ntk service` subcommand with HTTP endpoints (`GET /health`, `GET /ready`, `POST /task/submit`) for local background-service operation.
- Added local Docker service baseline assets: `deployments/Dockerfile.service`, `deployments/docker-compose.local.yml`, and `deployments/service.local.env.example`.
- Added local service-mode operations runbook: `docs/operations/service-mode-local-runbook.md`.
- Added CI `Dual Runtime Gate` job validating runtime-mode contracts, service orchestration tests, service endpoint tests, and Docker compose smoke checks.

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
- Release pipeline now validates compatibility/support documentation and ships `COMPATIBILITY.md` inside packaged artifacts.
- Release pipeline now enforces presence of support lifecycle/EOL section and EOL table header in `COMPATIBILITY.md`.
- CI coverage job now exports `lcov`, JSON summary, and HTML report artifacts, and enforces minimum line/function coverage thresholds.
- Manifest interactive commands (`check`, `render`, `apply`) now try picker-based manifest selection first, with manual path input fallback on cancel.
- Interactive CLI loops (`rustyline` and legacy raw-mode fallback) now render a live status bar above the prompt and update status by command outcome.
- Interactive CLI now handles local `/history` command to open session history viewer without delegating to orchestrator command routing.
- Interactive `rustyline` helper now applies lightweight ANSI-based highlighting with language detection and keyword/string/comment styling.
- Interactive syntax highlighting now uses parser reuse + thread-local cache and a bounded large-line fast-path for lower input latency.
- Interactive command outcome signaling now supports optional desktop notifications (Windows toast / macOS `osascript` / Linux `notify-send`) and respects focus-based gating when enabled.
- Orchestrator command routing now recognizes async manifest aliases (top-level and `/manifest *-async` forms) and emits standardized progress messages with percent/step context.
- Interactive CLI loops now route interruption state into orchestrator command execution, enabling runtime-aware command cancellation checks.
- `/help` and `/manifest list` command paths now use bounded runtime cache lookups with cache hit/miss metrics and stale-entry pruning.
- Interactive input startup now wires `predictive_input` from resolved config into `RustylineInput`, allowing runtime enable/disable without code changes.
- Command processor now executes plugin before/after hooks with non-blocking error isolation and plugin observability gauges.
- Interactive loops now apply deterministic recovery policy for `rustyline` and legacy read failures (`3` consecutive failures max) with warning notifications and footer diagnostics.
- Interactive loops now mirror command/text history into shared typed state (`CliState`) while preserving existing history viewer behavior.
- Interactive runtime now seeds in-memory history from resumed local state and persists snapshots on shutdown paths (including interrupted/error exits), with bounded local snapshot retention.
- Interactive startup flow now prompts for local snapshot selection only when multiple session snapshots are available, with fallback to latest snapshot on cancel/error.
- Interactive status bar rendering now goes through frame scheduling (coalesced/rate-limited), and legacy async input polling now uses scheduler-aware timeouts for smoother frame cadence.
- Markdown rendering now applies token-level ANSI styling for fenced code blocks (keywords/strings/numbers/comments) while preserving non-color fallback output.
- Enterprise roadmap Phase 8 (AI Assistant Integration) is now fully delivered, including operational controls and AI-specific release gating.
- Configuration now supports deterministic runtime mode selection (`general.runtime_mode`) with environment override (`NTK_RUNTIME_MODE`), and `/config` supports showing/updating runtime mode.
- `/task submit` now uses runtime-aware execution: immediate local execution in `cli` mode, and asynchronous queued background-worker dispatch in `service` mode.
- `/task list` and `/task watch` now include retry-attempt metadata and recent audit event history for task lifecycle transparency.
- Non-interactive CLI now supports a long-running service runtime profile via `ntk service --host <host> --port <port>`.

### Fixed
- Terminal resize stability improvements to avoid duplicated/overlapped UI content on rapid terminal/font-size changes.
- Interactive terminal behavior now preserves visible shell output/history on `/quit` and `Ctrl+C` (no alternate screen wipe).
- Cursor visibility/blinking handling improved in interactive prompt flow.
- Environment-variable race flake fixed in feature-detection tests by synchronizing tests that mutate `NTK_USE_*`.
- OpenTelemetry subscriber layering/type mismatch fixed in `otel` tracing setup (paths with/without OTLP now compile and initialize correctly).
- Non-interactive CLI now calls telemetry shutdown before `process::exit`, preventing loss of buffered OTLP data.
- Async manifest aliases now honor `Ctrl+C` cancellation by aborting in-flight async executor tasks and returning `Interrupted` status.
- Interactive runtime now avoids immediate session termination on transient input backend failures and recovers command/text panics as controlled `Error` outcomes.

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
- Added release gate validation for compatibility lifecycle semantics in GitHub Actions.
- Added coverage sweep validation with `cargo llvm-cov` and report exports (line coverage baseline around `68.2%`, functions around `72.5%`).
- Added AI gate validation slices in CI: `e2e_ai_*`, `process_ai_command_*`, and retry/rate-limit resilience tests.
- Added dual-runtime service-mode task tests validating queue submission behavior and worker retry-delay policy semantics.
- Added service CLI tests covering `service --help` command surface and HTTP helper parsing/response routines.
- Added service endpoint handler tests for `GET /health`, invalid JSON rejection on `POST /task/submit`, and accepted task submission responses.

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