# NetToolsKit.CLI — Implementation Plan (v0.2.0)

> **Planning Document**: This document serves as the comprehensive implementation plan and roadmap for NetToolsKit.CLI. It tracks requirements, phases, milestones, and technical decisions throughout the project lifecycle. Use this as the single source of truth for project scope, progress, and architecture.

> Multi‑stack code generator based on **static templates**. CLI written in **Rust**. No Roslyn in this phase. Placeholders with `{{Tokens}}`, collision policy, `--dry-run` with unified diff, and optional insertion of `// TODO` + `NotImplementedException` when optional sections are empty.

**Version**: 0.2.0
**Status**: Phase 2 (Async Architecture)
**Tests**: 13/13 ✅

---

## 📚 Documentation

### Reference Documents
- [comparative-analysis-codex-vs-ntk.md](comparative-analysis-codex-vs-ntk.md) - Detailed Codex vs NTK comparison

### Phase Implementation Documents
- [task-phase-0.0-infrastructure.md](task-phase-0.0-infrastructure.md) - Feature flags & testing setup
- [task-phase-1.1-ui-reorganization.md](task-phase-1.1-ui-reorganization.md) - Legacy/modern UI split
- [task-phase-1.2-hybrid-architecture.md](task-phase-1.2-hybrid-architecture.md) - Ratatui 16ms polling
- [task-phase-1.3-event-stream.md](task-phase-1.3-event-stream.md) - Zero CPU idle implementation
- [task-phase-2.0-async-architecture-plan.md](task-phase-2.0-async-architecture-plan.md) - Async roadmap
- [task-phase-2.1-async-executor.md](task-phase-2.1-async-executor.md) - Command executor
- [task-phase-2.2-cli-integration.md](task-phase-2.2-cli-integration.md) - Loop integration
- [task-phase-2.3-command-conversion.md](task-phase-2.3-command-conversion.md) - `/list-async` conversion

---

## 🖥️ Terminal Layout Architecture

### Layout Structure
The CLI implements a **scrollable header + dynamic content area** with a **fixed footer** design:

**Scrollable Area:**
- Header with branding and context information
- Logo (ASCII art)
- Tips and command hints
- Command execution output (commands + results)

**Fixed Footer:**
- Telemetry stream (always visible at bottom)
- Real-time log updates (non-blocking)

As commands are executed, the header scrolls up naturally with the content, while the footer remains fixed at the bottom of the terminal:

```
-> header
╭─────────────────────────────────────────────────────────────────────────────────────────╮
│ >\_ NetToolsKit CLI (1.0.0)                                                             │
│    A comprehensive toolkit for backend development                                      │
│                                                                                         │
│    directory: ~\\Documents\\Trabalho\\...\\NetToolsKit\\tools\\nettoolskit-cli          │
╰─────────────────────────────────────────────────────────────────────────────────────────╯


 ███╗   ██╗███████╗████████╗████████╗ ██████╗  ██████╗ ██╗     ███████╗██╗  ██╗██╗████████╗
 ████╗  ██║██╔════╝╚══██╔══╝╚══██╔══╝██╔═══██╗██╔═══██╗██║     ██╔════╝██║ ██╔╝██║╚══██╔══╝
 ██╔██╗ ██║█████╗     ██║      ██║   ██║   ██║██║   ██║██║     ███████╗█████╔╝ ██║   ██║
 ██║╚██╗██║██╔══╝     ██║      ██║   ██║   ██║██║   ██║██║     ╚════██║██╔═██╗ ██║   ██║
 ██║ ╚████║███████╗   ██║      ██║   ╚██████╔╝╚██████╔╝███████╗███████║██║  ██╗██║   ██║
 ╚═╝  ╚═══╝╚══════╝   ╚═╝      ╚═╝    ╚═════╝  ╚═════╝ ╚══════╝╚══════╝╚═╝  ╚═╝╚═╝   ╚═╝


💡 Tip: Type / to see available commands
   Use ↑↓ to navigate, Enter to select, /quit to exit
-> header

-> dynamic area
>
-> dynamic area


-> footer
---
2025-10-30T19:08:38.309653Z  INFO 76: Starting NetToolsKit CLI interactive mode
2025-10-30T19:08:38.309707Z  INFO 28: Initializing metrics collector
2025-10-30T19:08:38.373509Z  INFO 96: Displaying application logo and UI
2025-10-30T19:08:48.444836Z  INFO 28: Initializing metrics collector
2025-10-30T19:08:48.444916Z  INFO 33: Processing CLI command command=/check command\_type=check
2025-10-30T19:08:48.445130Z  INFO 153: Operation completed operation=command\_execution duration\_ms=0
2025-10-30T19:08:48.445218Z  WARN 167: Timer dropped without explicit stop - auto-recording operation=command\_execution duration\_ms=0
2025-10-30T19:08:48.445272Z  INFO 90: Command execution completed command=/check duration\_ms=0 status="error"
2025-10-30T19:08:48.445333Z  INFO 113: Metrics summary logged counter\_count=2 gauge\_count=0
---
-> footer
```

### Key Requirements
1. **Header**: Always visible at top, shows context and branding
2. **Dynamic Area**:
   - Scrolls vertically as commands are executed
   - Prompt always repositions below latest output
   - Supports multi-line output and progress bars
3. **Footer**:
   - Always visible at bottom
   - Real-time log streaming (non-blocking)
   - Configurable verbosity levels
4. **Layout Preservation**:
   - Header/footer remain fixed during commands
   - `/clear` resets to initial state
   - No flickering or layout shifts

### Implementation Status
- [x] Fixed header with logo and context (Phase 1.2)
- [x] Fixed footer with telemetry stream (Phase 1.2)
- [x] Dynamic scrollable area (Phase 1.2)
- [x] Event-driven updates (Phase 1.3)
- [x] Non-blocking progress display (Phase 2.2-2.3)
- [ ] `/clear` command implementation
- [ ] Configurable footer verbosity
- [ ] Cursor safety guarantees

---

## 1. Purpose
Deliver a single binary `ntk` that scaffolds and expands projects and files for **.NET**, **Vue/Quasar**, **Clojure**, and **Rust** from versioned **manifests** and **templates**, with safety (idempotency), predictability (show diffs before write), and maintainability.

## 2. In-scope (v0.2.0)

**CLI Core & Workflow**
- [x] Rust CLI with subcommands: `list`, `check`, `new`, `render`, **`apply`**.
- [x] Interactive terminal UI with command palette and footer telemetry stream.
- [x] Event-driven architecture (16ms polling, zero CPU idle).
- [x] Async command execution with progress tracking.
- [ ] Template engine: **Handlebars** in strict mode with helper library.
- [ ] Write collision policy: `fail` | `safe` | `force`.
- [ ] `--dry-run` prints unified diffs.
- [ ] Idempotency markers: `// <ntk:region ...>`.
- [ ] Optional post-steps triggered with `--with-post`.

**Manifests & Templates**
- [ ] Manifest per template and YAML **solution manifests**.
- [ ] Initial stacks: `.NET (background-service, api)`, `Vue/Quasar (app)`, `Clojure (app)`, `Rust (lib)`.
- [ ] Full template implementations (project, solution, classes scaffolding).

**Terminal UX**
- [x] Interactive terminal UI with command palette and footer telemetry stream.
- [x] Scrollable header with fixed footer design.
- [x] Event-driven input handling (Phase 1.2-1.3).
- [x] Async progress display (Phase 2.2-2.3).
- [ ] Configurable logging levels (enable/disable footer output, verbosity presets).
- [ ] Clear command to reset terminal back to initial header/logo layout.
- [ ] Persistent input prompt after command completion (cursor always below latest output).
- [ ] Enhanced input with rustyline (history, auto-complete).

**Quality & Insights**
- [x] Test suite: 13/13 passing.
- [ ] Test coverage sweep with coverage graph generation.

## 3. Out‑of‑scope (v0.2.0)
- Semantic refactoring of C# code (Roslyn).
- PATH‑discovered external plugins (`ntk-*`).
- Multi‑repo presets or orchestration.
- Telemetry/analytics.

## 4. Stakeholders
- Platform/Tooling, Backend, Frontend, DevOps, QA.

## 5. Assumptions
- Toolchains installed per stack (`dotnet`, `node`/`pnpm`, `cargo`, `clj/lein`).
- Git available for diffs and CI.
- No network access by default; post‑steps may use it when enabled.

## 6. Constraints
- Cross‑platform (Windows, Linux, macOS).
- Single executable per platform.
- Human and JSON outputs (`--json`).
- Terminal layout with scrollable header and fixed footer.

## 7. Risks and Mitigations
| Risk | Impact | Mitigation |
|------|--------|------------|
| Template drift | Inconsistent scaffolds | Versioned templates + PR reviews; snapshot tests |
| Overwrites | Data loss | Default `fail`; `--dry-run`; optional `.patch` backups |
| Idempotency bugs | Duplicate blocks | Region markers + regeneration tests |
| Post‑step failures | Broken env | `--with-post`; `--strict-post`; clear logs |

---

## 8. Requirements Analysis

### 8.1 Method
Lightweight elicitation and classification into **FR/NFR/BR**, explicit CLI contracts, and acceptance criteria.

### 8.2 Functional Requirements (FR)
**Core CLI**
- **FR01** [x] List templates (table output + JSON).
- **FR02** [ ] Check template/manifest (schema + semantic validation).
- **FR03** [ ] Render from variables (accept inline `--var`, `--vars-file`, `--output`).
- **FR04** [ ] Dry-run diff (unified diff preview with exit code on pending writes).
- **FR05** [ ] Write with collision policy (`fail`, `safe`, `force` toggle per file).
- **FR06** [ ] Idempotent regeneration (respect markers, skip unchanged regions).
- **FR07** [ ] Insert TODOs for optional gaps (template-level hints).
- **FR08** [ ] Run post-steps (`--with-post`, `--strict-post` fail-fast).
- **FR09** [ ] Project-level defaults via `.ntkrc.json` (discovery + precedence rules).

**Apply Engine**
- **FR10** [ ] Apply manifest as **feature slice** (context + selected layers).
- **FR11** [ ] Apply manifest as **layer-only**.
- **FR12** [ ] Apply manifest as **artifact-only** (e.g., entity or endpoint).
- **FR13** [ ] Support **existing solution guards** (`requireExistingProjects`, `onMissingProject`).
- **FR18** [ ] Deliver complete project/solution/class templates (base requirement for apply).

**Terminal Experience**
- **FR14** [x] Interactive terminal session with persistent footer logs, scrolling output, and command palette.
- **FR15** [ ] Configure logging (footer on/off, verbosity profiles).
- **FR16** [ ] Provide `/clear` (or equivalent) to reset the terminal to the initial layout.
- **FR17** [ ] Ensure input prompt always repositions below the latest output (cursor safety).
- **FR20** [x] Guard layout invariants (scrollable header + fixed footer after each command).

**Quality Insights**
- **FR19** [ ] Perform test coverage scan and display coverage graph (CLI report + export).

### 8.3 Non‑Functional Requirements (NFR)
Portability, packaging, observability, safety, testability, security.

### 8.4 Business Rules (BR)
- **BR01** Templates declare required variables.
- **BR02** Post‑steps are never implicit.
- **BR03** Diffs always available in dry‑run.
- **BR04** Default collision policy is `fail`.

### 8.5 CLI Contracts
```
ntk apply --manifest <file.yml> [--set key=val[,key=val]...] [--dry-run] [--with-post]
```
Exit codes: `0` ok, `1` args, `2` manifest error, `3` collision, `4` post‑step failure, `5` internal.

### 8.6 Deliverables
Binaries, `templates/`, `docs/README.md`, `docs/nettoolskit-cli.md`, `docs/TEMPLATES.md`, tests, CI.

---

## 9. Work Breakdown Structure (WBS)
**Foundation**
- **WBS-1 CLI Core**: [x] Clap setup, config parsing, output formatters.
- **WBS-2 Template Engine**: [ ] Handlebars helpers, strict mode enforcement, error surfacing.
- **WBS-3 Manifest & Validation**: [ ] YAML loader, schema validation, guard evaluation.
- **WBS-4 File Writer**: [ ] Collision policy piping, diff generator, backup strategy.

**Generation Features**
- **WBS-5 Idempotency/TODO**: [ ] Region marker writer, optional section handling.
- **WBS-6 Apply Engine**: [ ] Execution pipeline for feature/layer/artifact manifests.
- **WBS-7 Initial Templates**: [ ] Seed stacks for .NET, Vue/Quasar, Clojure, Rust.
- **WBS-13 Template Library Completion**: [ ] Full solution/project/class scaffolds, manifest metadata.

**Terminal Experience**
- **WBS-10 Terminal UX Enhancements**: [x] Header/logo rendering, fixed footer log view, palette scrolling & logging integration.
- **WBS-11 Logging Configuration**: [ ] Runtime toggles for telemetry verbosity/footer visibility.
- **WBS-12 Terminal Reset & Prompt**: [ ] `/clear` command, cursor positioning, prompt lifecycle guarantees.
- **WBS-15 Layout Validation Guard**: [ ] Snapshot tests for terminal layout compliance.

**Quality & Delivery**
- **WBS-8 Tests & CI**: [x] Basic test suite (13/13), [ ] Snapshot + collision suites, coverage gating, GitHub Actions.
- **WBS-14 Coverage Insights**: [ ] Test sweep automation, coverage graph export/presentation.
- **WBS-9 Docs & Release**: [ ] README, plan updates, templates guide, release packaging.

> **Execution order hint:** complete *Foundation* tasks before tackling *Generation Features*, so apply workflows have a stable engine. Finalize *Terminal Experience* improvements once the apply pipeline exists, then close with *Quality & Delivery* to validate and ship.

---

## 10. Milestones & Acceptance
- **M0 Skeleton**: [x] `ntk --help`, `ntk list`.
- **M1 Rendering Engine**: [ ] `ntk render` with `--var/--vars-file/--output`; Handlebars strict mode.
- **M2 Validation & Manifests**: [ ] `ntk check` schema validation; manifest parsing basics.
- **M3 Writing & Collisions**: [ ] `ntk new` honoring collision policy; `--dry-run` diffs.
- **M4 Idempotent Apply**: [ ] region markers, TODO insertion, guards enforced.
- **M5 Template Library**: [ ] project/solution/class templates for four stacks compile.
- **M6 Terminal Polish**: [ ] logging config, `/clear`, stable input prompt, terminal layout compliance.
- **M7 Quality Gates**: [ ] coverage sweep with graph, CI green on 3 OSes.
- **M8 Docs & Release**: [ ] docs complete; binaries signed/shipped; tag `v0.2.0`.

---

## 11. Implementation Phases Progress

### Phase 0: Infrastructure ✅ COMPLETE
**Document**: [task-phase-0.0-infrastructure.md](task-phase-0.0-infrastructure.md)
- [x] Feature flags: `modern-tui`, `event-driven`, `frame-scheduler`, `full-tui`.
- [x] Environment variables: `NTK_USE_MODERN_TUI`, `NTK_USE_EVENT_STREAM`, `NTK_USE_ASYNC_EXECUTOR`.
- [x] Testing framework established.
- [x] Documentation structure.

### Phase 1: UI Modernization ✅ COMPLETE

#### Phase 1.1: UI Reorganization ✅
**Document**: [task-phase-1.1-ui-reorganization.md](task-phase-1.1-ui-reorganization.md)
- [x] Split `ui/src` into `legacy/` and `modern/` modules.
- [x] Zero visual changes (100% compatibility).
- [x] Clean module boundaries.
- [x] All tests passing.

#### Phase 1.2: Hybrid Architecture ✅
**Document**: [task-phase-1.2-hybrid-architecture.md](task-phase-1.2-hybrid-architecture.md)
- [x] Ratatui 0.28 integration.
- [x] 16ms event polling (3.1x faster than 50ms legacy).
- [x] Hybrid approach: Modern events + legacy visuals.
- [x] Zero visual changes maintained.
- [x] Terminal layout with scrollable header and fixed footer.

#### Phase 1.3: Event Stream ✅
**Document**: [task-phase-1.3-event-stream.md](task-phase-1.3-event-stream.md)
- [x] EventStream implementation.
- [x] Zero CPU idle state.
- [x] Async event handling.
- [x] Build system working for both modes.

### Phase 2: Async Architecture 🔄 IN PROGRESS

#### Phase 2.0: Planning ✅
**Document**: [task-phase-2.0-async-architecture-plan.md](task-phase-2.0-async-architecture-plan.md)
- [x] Architecture design.
- [x] Production roadmap.
- [x] 2-week timeline established.

#### Phase 2.1: Async Executor ✅
**Document**: [task-phase-2.1-async-executor.md](task-phase-2.1-async-executor.md)
- [x] `AsyncCommandExecutor` (~335 lines).
- [x] `CommandHandle`, `CommandProgress`, `ProgressSender`.
- [x] Concurrency limits (default: 10).
- [x] Test suite (4/4 passing).
- [x] Zero warnings.

#### Phase 2.2: CLI Integration ✅
**Document**: [task-phase-2.2-cli-integration.md](task-phase-2.2-cli-integration.md)
- [x] `cli/src/async_executor.rs` (~177 lines).
- [x] `commands/src/processor_async.rs` (~112 lines).
- [x] Progress display (message, percentage, tasks).
- [x] Environment variable control.
- [x] Test suite (7/7 passing total).

#### Phase 2.3: Command Conversion ✅
**Document**: [task-phase-2.3-command-conversion.md](task-phase-2.3-command-conversion.md)
- [x] `/list-async` with 4-stage progress.
- [x] Helper function `is_async_command()`.
- [x] Test suite (13/13 passing total).
- [x] Zero warnings.

#### Phase 2.4: Additional Commands 📋 PLANNED
- [ ] `/new-async` - Project scaffolding.
- [ ] `/render-async` - Template rendering.
- [ ] `/apply-async` - Configuration application.
- [ ] Ctrl+C cancellation.
- [ ] Enhanced progress patterns.

#### Phase 2.5: Caching System 📋 PLANNED
- [ ] LRU cache implementation.
- [ ] TTL per command type.
- [ ] Memory-bounded eviction.
- [ ] Performance benchmarks.

#### Phase 2.6: Advanced Features 📋 PLANNED
- [ ] Predictive input.
- [ ] Configuration system.
- [ ] Plugin foundation.
- [ ] Error recovery.

---

## 12. Performance & UX Improvements (Codex-RS Analysis)

### 12.1 Context
Analysis of `codex-rs/cli` identified critical gaps in NetToolsKit CLI's performance and UX.
See: [comparative-analysis-codex-vs-ntk.md](comparative-analysis-codex-vs-ntk.md)

### 12.2 Critical Improvements

#### **IMP-1: Raw Mode Guard** ⭐⭐⭐
**Problem:** Toggling raw mode every command cycle causes overhead and flickering.
**Status:** ✅ **COMPLETED** (Phase 1.2)

**Implementation:**
```rust
pub struct RawModeGuard;
impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
    }
}
```

**Tasks:**
- [x] Create `RawModeGuard` in `cli/src/lib.rs`.
- [x] Replace manual enable/disable with guard pattern.
- [x] Add RAII cleanup on panic/exit.

---

#### **IMP-2: Event-Driven Architecture** ⭐⭐⭐⭐⭐
**Problem:** Blocking event loop prevents async operations, no progress feedback.
**Status:** ✅ **COMPLETED** (Phase 1.2-1.3, Phase 2.1-2.3)

**Implementation:**
```rust
use tokio::sync::mpsc;
use crossterm::event::{EventStream, Event};

async fn run_interactive_loop() -> io::Result<ExitStatus> {
    let mut events = EventStream::new();
    let (interrupt_tx, mut interrupt_rx) = mpsc::unbounded_channel();

    loop {
        tokio::select! {
            Some(Ok(event)) = events.next() => {
                // Handle input events
            }
            Some(progress) = progress_rx.recv() => {
                // Display async progress
            }
            Some(_) = interrupt_rx.recv() => {
                // Handle Ctrl+C
            }
        }
    }
}
```

**Tasks:**
- [x] Event stream integration (Phase 1.3).
- [x] Async executor (Phase 2.1).
- [x] CLI loop integration (Phase 2.2).
- [x] Progress display (Phase 2.3).
- [ ] Ctrl+C handling (Phase 2.4).

---

#### **IMP-3: Enhanced Input Handling** ⭐⭐⭐
**Problem:** Basic readline without history, auto-complete, or multi-line editing.
**Status:** 📋 **PLANNED** (Phase 2.7+)

**Target State:**
```rust
use rustyline::{Editor, Config, CompletionType};

pub struct InteractiveShell {
    editor: Editor<CommandCompleter>,
    history_path: PathBuf,
}
```

**Tasks:**
- [ ] Add `rustyline` dependency (14.0).
- [ ] Implement `InteractiveShell` wrapper.
- [ ] Create `CommandCompleter` for palette.
- [ ] Add persistent history to `~/.config/nettoolskit/history.txt`.
- [ ] Integrate with event system (preserve CommandPalette).

---

#### **IMP-4: Progress Indicators** ⭐⭐
**Problem:** No feedback during long operations.
**Status:** ✅ **PARTIALLY COMPLETE** (Phase 2.2-2.3)

**Current Implementation:**
- ✅ Custom progress display with message/percentage/task counts.
- ✅ Real-time updates via async channels.
- ✅ Emoji indicators (🔍📦✅).

**Target Enhancement:**
```rust
use indicatif::{ProgressBar, ProgressStyle};

pub async fn apply_manifest_with_progress(manifest: Manifest) -> Result<()> {
    let pb = ProgressBar::new(manifest.steps.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")?
            .progress_chars("█▓▒░ ")
    );

    for step in manifest.steps.iter() {
        pb.set_message(format!("Applying {}", step.name));
        apply_step(step).await?;
        pb.inc(1);
    }

    pb.finish_with_message("✅ Applied successfully");
    Ok(())
}
```

**Tasks:**
- [x] Basic progress display (Phase 2.2).
- [x] Multi-stage progress (Phase 2.3).
- [ ] Add `indicatif` dependency (0.17).
- [ ] Progress bar helpers in `ui/src/progress.rs`.
- [ ] Spinner for indeterminate operations.

---

#### **IMP-5: Task Spawning & Parallelization** ⭐⭐⭐
**Problem:** Sequential processing blocks CLI during heavy operations.
**Status:** 📋 **PLANNED** (Phase 2.5+)

**Target State:**
```rust
use tokio::task::JoinSet;

pub async fn execute_apply(manifest: Manifest) -> Result<()> {
    let mut tasks = JoinSet::new();

    // Parallel validation
    tasks.spawn(validate_manifest(manifest.clone()));
    tasks.spawn(load_templates());
    tasks.spawn(check_filesystem());

    while let Some(result) = tasks.join_next().await {
        result??;
    }

    // Apply with progress
    apply_with_progress(manifest).await
}
```

**Tasks:**
- [ ] Identify parallelizable operations.
- [ ] Implement `JoinSet` pattern for validation.
- [ ] Add concurrent template loading.
- [ ] Progress aggregation for parallel tasks.
- [ ] Timeout handling.

---

### 12.3 Dependencies

**Current:**
```toml
[dependencies]
tokio = { version = "1.34", features = ["rt-multi-thread", "macros", "time", "net", "io-util", "sync"] }
ratatui = { version = "0.28.1", optional = true }
crossterm = { version = "0.28.1", features = ["event-stream"] }
owo-colors = "3.5"
futures = "0.3"
clap = "4.5"
tracing = "0.1"
```

**Planned Additions:**
```toml
# Enhanced input (IMP-3)
rustyline = "14.0"

# Progress indicators (IMP-4)
indicatif = "0.17"

# Async utilities
tokio-stream = "0.1"
```

---

### 12.4 Implementation Roadmap

#### **Phase 1: Foundation** ✅ COMPLETE
- [x] IMP-1: RawModeGuard.
- [x] IMP-2: Event-driven architecture.
- [x] Basic async executor.
- [x] Progress display.

**Deliverable:** Responsive CLI with async command execution.

---

#### **Phase 2: Core Features** 🔄 IN PROGRESS
- [x] Async command executor (Phase 2.1).
- [x] CLI integration (Phase 2.2).
- [x] First command conversion (Phase 2.3).
- [ ] Additional commands (Phase 2.4).
- [ ] Ctrl+C handling (Phase 2.4).

**Deliverable:** Production-ready async commands.

---

#### **Phase 3: Optimization** 📋 PLANNED
- [ ] IMP-5: Parallel validation.
- [ ] Caching system (Phase 2.5).
- [ ] Enhanced progress bars.
- [ ] Performance benchmarks.

**Deliverable:** Optimized template operations.

---

#### **Phase 4: Polish** 📋 PLANNED
- [ ] IMP-3: Rustyline integration.
- [ ] Persistent history.
- [ ] Enhanced auto-complete.
- [ ] Keyboard shortcuts.

**Deliverable:** Professional UX.

---

### 12.5 Success Metrics

**Performance:**
- [x] Startup time < 100ms (current: ~50ms ✅).
- [x] Input latency < 16ms (Phase 1.2: 16ms polling ✅).
- [x] Zero CPU when idle (Phase 1.3: EventStream ✅).
- [ ] Template rendering: 100+ files without blocking UI.

**UX:**
- [x] Async command execution ✅.
- [x] Real-time progress feedback ✅.
- [x] Non-blocking operations ✅.
- [x] Terminal layout compliance (scrollable header + fixed footer) ✅.
- [ ] Auto-complete functional.
- [ ] Persistent history.
- [ ] Graceful Ctrl+C handling.

**Code Quality:**
- [x] Test coverage: 13/13 passing ✅.
- [x] Zero warnings ✅.
- [ ] Test coverage > 70%.
- [ ] Benchmarks for critical operations.
- [ ] Zero panics in interactive mode.

---

### 12.6 References
- **Analysis:** [comparative-analysis-codex-vs-ntk.md](comparative-analysis-codex-vs-ntk.md)
- **Codex Source:** `codex/codex-rs/tui/src/`
- **Tokio Select:** https://docs.rs/tokio/latest/tokio/macro.select.html
- **Rustyline:** https://github.com/kkawakam/rustyline
- **Indicatif:** https://github.com/console-rs/indicatif

---

## 📊 Current Status Summary

### Build & Tests
```
✅ cargo build                         - OK
✅ cargo build --features modern-tui  - OK
✅ cargo test --lib                   - 13/13 passing

   cli           3/3
   commands      4/4
   core          4/4
   ui            2/2
```

### Performance Metrics
- ✅ Event polling: 16ms (target: <20ms).
- ✅ Idle CPU: 0% with event stream.
- ✅ Build warnings: 0.
- ✅ Backward compatibility: 100%.
- ✅ Terminal layout: Scrollable header + fixed footer.

### Environment Variables
| Variable | Purpose |
|----------|---------|
| `NTK_USE_MODERN_TUI` | Enable modern TUI mode (Phase 1.2+) |
| `NTK_USE_EVENT_STREAM` | Use event stream (Phase 1.3) |
| `NTK_USE_ASYNC_EXECUTOR` | Enable async commands (Phase 2.1+) |

### Feature Flags
| Flag | Purpose | Default |
|------|---------|---------|
| `modern-tui` | Modern UI with Ratatui | `off` |
| `event-driven` | Event-driven architecture | `off` |
| `frame-scheduler` | Frame scheduling | `off` |
| `full-tui` | Complete modern UI | `off` |

---

## 🔧 Technology Stack
- **Language:** Rust 2021 edition
- **UI Library:** Ratatui 0.28.1 (optional, feature-gated)
- **Terminal:** Crossterm 0.28.1 (with event-stream)
- **Async Runtime:** Tokio 1.34 (multi-thread, macros, time, net, io-util, sync)
- **Colors:** owo-colors 3.5
- **Utilities:** futures 0.3, clap 4.5, tracing 0.1

---

## 🎯 Project Goals
1. ✅ **Performance:** 3.1x faster event handling (16ms vs 50ms).
2. ✅ **Responsiveness:** Non-blocking command execution.
3. ✅ **Compatibility:** Zero visual changes in legacy mode.
4. ✅ **Quality:** 13/13 tests passing, zero warnings.
5. ✅ **Terminal Layout:** Scrollable header with fixed footer.
6. 🔄 **Production-Ready:** Async architecture in progress (Phase 2.4+).

---

## 📝 Development Guidelines

### Code Style
- Follow Rust 2021 edition conventions.
- Use `cargo fmt` for formatting.
- Use `cargo clippy` for linting.
- Zero warnings policy.

### Testing
- Write tests for all new features.
- Maintain 100% test pass rate.
- Use `#[tokio::test]` for async tests.
- Feature-gate modern-tui tests.

### Documentation
- Document all public APIs.
- Include usage examples.
- Update phase documents.
- Keep this index current.

### Git Workflow
- Feature branches for new work.
- Descriptive commit messages.
- PR reviews required.
- Squash merge to main.

---

## 🐛 Known Issues

### Deferred Issues
1. **Cursor positioning bug** (modern mode)
   - **Issue:** Prompt returns to top after command.
   - **Status:** Deferred to later phase.
   - **Workaround:** Use legacy mode.
   - **Documented:** `.docs/bugfixes/cursor-position-pending.md`.

### Active Issues
None

---

## 📖 References

### External Documentation
- [Ratatui Documentation](https://docs.rs/ratatui/)
- [Tokio Documentation](https://docs.rs/tokio/)
- [Crossterm Documentation](https://docs.rs/crossterm/)
- [Rustyline Documentation](https://docs.rs/rustyline/)
- [Indicatif Documentation](https://docs.rs/indicatif/)

### Internal Links
- [AGENTS.md](../../.github/AGENTS.md) - AI agent configuration
- [CHANGELOG.md](../../CHANGELOG.md) - Project changelog
- [README.md](../../README.md) - Project overview

---

**Document Version:** 2.0
**Last Updated:** 2025-11-02
**Next Review:** After Phase 2.4 completion