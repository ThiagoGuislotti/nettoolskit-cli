# NetToolsKit.CLI — Implementation Plan (v0.2.0)

> **Planning Document**: This document serves as the comprehensive implementation plan and roadmap for NetToolsKit.CLI. It tracks requirements, phases, milestones, and technical decisions throughout the project lifecycle. Use this as the single source of truth for project scope, progress, and architecture.

> Multi‑stack code generator based on **static templates**. CLI written in **Rust**. No Roslyn in this phase. Placeholders with `{{Tokens}}`, collision policy, `--dry-run` with unified diff, and optional insertion of `// TODO` + `NotImplementedException` when optional sections are empty.

---

## 🖥️ Terminal Layout Architecture

### Layout Structure
The CLI implements a **scrollable header + dynamic content area** with a **fixed footer** design.

**Requirements:**
1. **Header**: Always visible at top, shows context and branding
2. **Dynamic Area**: Scrolls vertically as commands are executed; prompt always repositions below latest output; supports multi-line output and progress bars
3. **Footer**: Always visible at bottom; real-time log streaming (non-blocking); configurable verbosity levels
4. **Layout Preservation**: Header/footer remain fixed during commands; `/clear` resets to initial state; no flickering or layout shifts

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


💡 Tip: Type /help to see all commands, or / to open command palette
   Use ↑↓ to navigate, Enter to select, /quit to exit
-> header

-> dynamic area
>

> /

› /help      Display help information and available commands
  /manifest  Manage and apply manifests (submenu)
  /translate Translate code between languages (deferred)
  /quit      Exit NetToolsKit CLI

> /manifest

› /manifest list   Discover available manifests in the workspace
  /manifest check  Validate manifest structure and dependencies
  /manifest render Preview generated files without creating them
  /manifest apply  Apply manifest to generate/update project files

> /manifest lis

› /manifest list   Discover available manifests in the workspace

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

**Commands:**
/help      - Display help information and available commands
/manifest  - Manage and apply manifests (submenu)
  ├─ list   - Discover available manifests in workspace
  ├─ check  - Validate manifest structure and dependencies
  ├─ render - Preview generated files without creating them
  └─ apply  - Apply manifest to generate/update project files
/translate - Translate code between languages (deferred)
/quit      - Exit NetToolsKit CLI

---

## 📐 Code Architecture (Layered Architecture)

> **Full Reference**: [ARCHITECTURE.txt](../../ARCHITECTURE.txt) (complete diagram at the repository root)

The NetToolsKit CLI architecture follows a **four-layer hierarchical model** with a **bottom-up dependency flow** (base → top). Each layer can only depend on lower layers, guaranteeing isolation and zero dependency cycles.

### 1.1 Hierarchical Levels

```
┌──────────────────────────────────────────────────────────────┐
│ LEVEL 4: Entry Point (Orchestration)                        │
│   └─ cli: application entry point                           │
└──────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌──────────────────────────────────────────────────────────────┐
│ LEVEL 3: Application (Business Logic)                       │
│   └─ commands: enum orchestrator for commands               │
│       ├─ src/                                               │
│       │   ├─ translate: language transcription pipeline     │
│       │   ├─ manifest: orchestration (Apply, Check, Test)   │
│       │   └─ templating: Handlebars (core, string-utils)    │
│       └─ tests/                                             │
└──────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌──────────────────────────────────────────────────────────────┐
│ LEVEL 2: Presentation & Infrastructure                      │
│   ├─ otel: logging/telemetry                                │
│   └─ ui: terminal interface (crossterm)                     │
└──────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌──────────────────────────────────────────────────────────────┐
│ LEVEL 1: Foundation (ZERO internal deps)                    │
│   ├─ core: fundamental types (Result, Config, Features)     │
│   ├─ string-utils: string manipulation                      │
│   ├─ async-utils: async helpers                             │
│   └─ file-search: file discovery and filtering              │
└──────────────────────────────────────────────────────────────┘
```

### 1.2 Dependency Flow (Bottom-Up)

**LEVEL 1 (Foundation)** → ZERO internal dependencies
- `core`: fundamental types (Result, Config, Features)
- `string-utils`: string manipulation (ZERO total deps)
- `async-utils`: async helpers (tokio, futures)
- `file-search`: file discovery and filtering

**LEVEL 2 (Infrastructure)** → depends only on Level 1
- `otel`: logging/telemetry → depends on `core`
- `ui`: terminal interface → depends on `core`, `string-utils`

**LEVEL 3 (Application)** → depends on Levels 1 and 2
- `commands`: command orchestrator enum → depends on `core`, `otel`, `ui`, `async-utils`
  - `src/translate`: language transcription
  - `src/manifest`: orchestration (Apply, Check, Test...)
  - `src/templating`: Handlebars → depends on `core`, `string-utils`

**LEVEL 4 (Entry Point)** → depends on everything
- `cli`: application entry point → depends on `commands`, `ui`, `core`, `async-utils`, `otel`, `file-search`

### 1.3 Golden Rules

**1. LEVEL 1 (Foundation)**
- ✓ ZERO internal dependencies
- ✓ Only essential external dependencies
- ✗ Never depends on higher levels

**2. LEVEL 2 (Infrastructure)**
- ✓ May depend on Level 1
- ✗ Cannot depend on Levels 3 or 4

**3. LEVEL 3 (Application)**
- ✓ May depend on Levels 1 and 2
- ✓ Commands contains manifest and translate inside `src/`
- ✗ Cannot depend on Level 4

**4. LEVEL 4 (Entry Point)**
- ✓ May depend on ALL levels
- ✗ Nothing is allowed to depend on it

### 1.4 Circular Dependency Resolution

**Identified Problem:**
- `commands → ui → otel → commands` (cycle detected)

**Implemented Solution:**
- Isolated `command-definitions` crate (ZERO internal deps)
- Contains only the `Command` enum (7 variants: List, Check, Render, New, Apply, Translate, Quit)
- `ui` depends on `command-definitions` (not on `commands`)
- Breaks the cycle: `commands → ui → command-definitions` ✅

**Enum architecture as the single source of truth:**
- Command enum uses `strum 0.26` (Display, EnumIter, EnumString, IntoStaticStr)
- Centralizes command definitions
- Guarantees consistency between UI and application logic

### 1.5 Validation Status

- ✅ Compilation: 11.85s (release)
- ✅ Tests: 186/188 passing (98.9%)
- ✅ Detected cycles: **ZERO**
- ✅ Hierarchy: **VALIDATED**
- ✅ Isolation: **CORRECT**

### 1.6 Decision Rationale

**Commands at Level 3:**
- Orchestrates every component (otel, ui)
- Implements the business logic for the commands
- Contains internal submodules inside `src/`:
  * `translate`: language transcription
  * `manifest`: template orchestration
  * `templating`: Handlebars engine (depends on core, string-utils)
- Modular structure without splitting into separate crates

**Otel at Level 2:**
- Depends on `ui` for `append_footer_log`
- Required for visual log feedback in the terminal
- Acceptable because it does not create cycles with Level 3

---

## 2. Technology Stack
- **Language:** Rust 2021 edition
- **UI Library:** Ratatui 0.28.1 (optional, feature-gated)
- **Terminal:** Crossterm 0.28.1 (with event-stream)
- **Async Runtime:** Tokio 1.34 (multi-thread, macros, time, net, io-util, sync)
- **Colors:** owo-colors 3.5
- **Utilities:** futures 0.3, clap 4.5, tracing 0.1

---

## 3. Development Guidelines

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

## 4. Purpose
Deliver a single binary `ntk` that scaffolds and expands projects and files for **.NET**, **Vue/Quasar**, **Clojure**, and **Rust** from versioned **manifests** and **templates**, with safety (idempotency), predictability (show diffs before write), and maintainability.

---

## 5. Scope

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

---

## 6. Out‑of‑scope (v0.2.0)
- Semantic refactoring of C# code (Roslyn).
- PATH‑discovered external plugins (`ntk-*`).
- Multi‑repo presets or orchestration.
- Telemetry/analytics.

---

## 7. Stakeholders
- Platform/Tooling, Backend, Frontend, DevOps, QA.

---

## 8. Constraints
- Cross‑platform (Windows, Linux, macOS).
- Single executable per platform.
- Human and JSON outputs (`--json`).
- Terminal layout with scrollable header and fixed footer.

---

## 9. Assumptions
- Toolchains installed per stack (`dotnet`, `node`/`pnpm`, `cargo`, `clj/lein`).
- Git available for diffs and CI.
- No network access by default; post‑steps may use it when enabled.

---

## 10. Requirements Analysis

### 10.1 Method
Lightweight elicitation and classification into **FR/NFR/BR**, explicit CLI contracts, and acceptance criteria.

### 10.2 Functional Requirements (FR)
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

### 10.3 Non‑Functional Requirements (NFR)
Portability, packaging, observability, safety, testability, security.

### 10.4 Business Rules (BR)
- **BR01** Templates declare required variables.
- **BR02** Post‑steps are never implicit.
- **BR03** Diffs always available in dry‑run.
- **BR04** Default collision policy is `fail`.

### 10.5 CLI Contracts
```
ntk apply --manifest <file.yml> [--set key=val[,key=val]...] [--dry-run] [--with-post]
```
Exit codes: `0` ok, `1` args, `2` manifest error, `3` collision, `4` post‑step failure, `5` internal.

### 10.6 Deliverables
Binaries, `templates/`, `docs/README.md`, `docs/nettoolskit-cli.md`, `docs/TEMPLATES.md`, tests, CI.

---

## 11. Work Breakdown Structure (WBS)
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

## 12. Milestones & Acceptance
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

## 13. Implementation Phases Progress

### 📋 Phases Overview

| Phase | Status | Focus | Document |
|-------|--------|-------|----------|
| **Phase 0** | ✅ Complete | Infrastructure & Testing | [task-phase-0.0-infrastructure.md](task-phase-0.0-infrastructure.md) |
| **Phase 1.1** | ✅ Complete | UI Reorganization | [task-phase-1.1-ui-reorganization.md](task-phase-1.1-ui-reorganization.md) |
| **Phase 1.2** | ✅ Complete | Hybrid Architecture | [task-phase-1.2-hybrid-architecture.md](task-phase-1.2-hybrid-architecture.md) |
| **Phase 1.3** | ✅ Complete | Event Stream | [task-phase-1.3-event-stream.md](task-phase-1.3-event-stream.md) |
| **Phase 2.0** | ✅ Complete | Async Planning | [task-phase-2.0-async-architecture-plan.md](task-phase-2.0-async-architecture-plan.md) |
| **Phase 2.1** | ✅ Complete | Async Executor | [task-phase-2.1-async-executor.md](task-phase-2.1-async-executor.md) |
| **Phase 2.2** | ✅ Complete | CLI Integration | [task-phase-2.2-cli-integration.md](task-phase-2.2-cli-integration.md) |
| **Phase 2.3** | ✅ Complete | Command Conversion | [task-phase-2.3-command-conversion.md](task-phase-2.3-command-conversion.md) |
| **Phase 2.4** | 🔄 In Progress | Additional Commands | TBD |
| **Phase 2.5** | 📋 Planned | Caching System | TBD |
| **Phase 2.6** | 📋 Planned | Advanced Features | TBD |
| **Phase 3.1** | 📋 Planned | Rich State Management | TBD |
| **Phase 3.2** | 📋 Planned | Session Persistence | TBD |
| **Phase 3.3** | 📋 Planned | Frame Scheduler | TBD |
| **Phase 4.1** | 📋 Planned | Enhanced Input | TBD |
| **Phase 4.2** | 📋 Planned | File Picker | TBD |
| **Phase 4.3** | 📋 Planned | Status Bar | TBD |
| **Phase 4.4** | 📋 Planned | Visual History | TBD |
| **Phase 5.1** | 📋 Planned | Syntax Highlighting | TBD |
| **Phase 5.2** | 📋 Planned | Markdown Rendering | TBD |
| **Phase 5.3** | 📋 Planned | Clipboard Integration | TBD |
| **Phase 5.4** | 📋 Planned | Desktop Notifications | TBD |

**Overall Progress:** 40% complete (8/20 phases) | **Current Focus:** Phase 2.4 (Additional Commands)
**Total Tasks:** 113 | **Completed:** 5/113 (4%)

---

### 12.1 Phase 0: Infrastructure
**Document**: [task-phase-0.0-infrastructure.md](task-phase-0.0-infrastructure.md)
- [x] Feature flags: `modern-tui`, `event-driven`, `frame-scheduler`, `full-tui`.
- [x] Environment variables: `NTK_USE_MODERN_TUI`, `NTK_USE_EVENT_STREAM`, `NTK_USE_ASYNC_EXECUTOR`.
- [x] Testing framework established.
- [x] Documentation structure.

### 12.2 Phase 1: UI Modernization

##### 12.2.1 Phase 1.1: Refactoring Inicial
**Document**: [task-phase-1.1-ui-reorganization.md](task-phase-1.1-ui-reorganization.md)
- [x] Split `ui/src` into `legacy/` and `modern/` modules.
- [x] Zero visual changes (100% compatibility).
- [x] Clean module boundaries.
- [x] All tests passing.

#### 12.2.2 Phase 1.2: TUI Context Architecture
**Document**: [task-phase-1.2-hybrid-architecture.md](task-phase-1.2-hybrid-architecture.md)
- [x] Ratatui 0.28 integration.
- [x] 16ms event polling (3.1x faster than 50ms legacy).
- [x] Hybrid approach: Modern events + legacy visuals.
- [x] Zero visual changes maintained.
- [x] Terminal layout with scrollable header and fixed footer.
- [x] Fixed header with logo and context.
- [x] Fixed footer with telemetry stream.
- [x] Dynamic scrollable area.

##### 12.2.3 Phase 1.3: Feature Flag Integration
**Document**: [task-phase-1.3-event-stream.md](task-phase-1.3-event-stream.md)
- [x] EventStream implementation.
- [x] Zero CPU idle state.
- [x] Async event handling.
- [x] Build system working for both modes.
- [x] Event-driven updates.

### 12.3 Phase 2: Async Architecture

#### 12.3.1 Phase 2.0: Planning
**Document**: [task-phase-2.0-async-architecture-plan.md](task-phase-2.0-async-architecture-plan.md)
- [x] Architecture design.
- [x] Production roadmap.
- [x] 2-week timeline established.

#### 12.3.2 Phase 2.1: Async Executor
**Document**: [task-phase-2.1-async-executor.md](task-phase-2.1-async-executor.md)
- [x] `AsyncCommandExecutor` (~335 lines).
- [x] `CommandHandle`, `CommandProgress`, `ProgressSender`.
- [x] Concurrency limits (default: 10).
- [x] Test suite (4/4 passing).
- [x] Zero warnings.

#### 12.3.3 Phase 2.2: CLI Integration
**Document**: [task-phase-2.2-cli-integration.md](task-phase-2.2-cli-integration.md)
- [x] `cli/src/async_executor.rs` (~177 lines).
- [x] `commands/src/processor_async.rs` (~112 lines).
- [x] Progress display (message, percentage, tasks).
- [x] Environment variable control.
- [x] Test suite (7/7 passing total).
- [x] Non-blocking progress display.

#### 12.3.4 Phase 2.3: Command Conversion
**Document**: [task-phase-2.3-command-conversion.md](task-phase-2.3-command-conversion.md)
- [x] `/list-async` with 4-stage progress.
- [x] Helper function `is_async_command()`.
- [x] Test suite (13/13 passing total).
- [x] Zero warnings.

#### 12.3.5 Phase 2.4: Additional Commands
- [ ] `/new-async` - Project scaffolding.
- [ ] `/render-async` - Template rendering.
- [ ] `/apply-async` - Configuration application.
- [ ] Ctrl+C cancellation.
- [ ] Enhanced progress patterns.
- [ ] `/clear` command implementation.
- [ ] Cursor safety guarantees.

#### 12.3.6 Phase 2.5: Caching System
- [ ] LRU cache implementation.
- [ ] TTL per command type.
- [ ] Memory-bounded eviction.
- [ ] Performance benchmarks.
- [ ] Configurable footer verbosity.

#### 12.3.7 Phase 2.6: Advanced Features
- [ ] Predictive input.
- [ ] Configuration system.
- [ ] Plugin foundation.
- [ ] Error recovery.

---

### 12.4 Phase 3: State & Persistence

#### 12.4.1 Phase 3.1: Rich State Management
- [ ] `CliState` structure with history, session, config
- [ ] `HistoryEntry` trait for command/text entries
- [ ] Arc-based state sharing
- [ ] State serialization/deserialization

#### 12.4.2 Phase 3.2: Session Persistence
- [ ] Save sessions to disk (JSON format)
- [ ] Load previous sessions
- [ ] Session selection UI (resume picker)
- [ ] Session metadata (id, timestamp, history)

#### 12.4.3 Phase 3.3: Frame Scheduler
- [ ] Frame coalescing implementation
- [ ] Rate limiting (60 FPS target)
- [ ] Async-friendly scheduler
- [ ] Integration with event loop

---

### 12.5 Phase 4: Interactive Features

#### 12.5.1 Phase 4.1: Tab Completion (IMP-3)
- [ ] Rustyline integration
- [ ] Command history persistence
- [ ] Auto-complete for commands
- [ ] Multi-line editing support
- [ ] Integration with CommandPalette

#### 12.5.2 Phase 4.2: File Picker
- [ ] Fuzzy finder implementation
- [ ] Regex support
- [ ] Real-time filtering
- [ ] Keyboard navigation

#### 12.5.3 Phase 4.3: Status Bar
- [ ] Status bar widget
- [ ] Mode indicators
- [ ] Notifications queue
- [ ] Resource usage display

#### 12.5.4 Phase 4.4: Interactive Prompts
- [ ] History viewer widget
- [ ] Scroll support
- [ ] Entry rendering
- [ ] Search/filter capabilities

---

### 12.6 Phase 5: Advanced Features

#### 12.6.1 Phase 5.1: Syntax Highlighting
- [ ] Tree-sitter integration
- [ ] Language support (Rust, C#, JS/TS)
- [ ] Theme support
- [ ] Performance optimization

#### 12.6.2 Phase 5.2: Markdown Rendering
- [ ] Pulldown-cmark integration
- [ ] Styled rendering
- [ ] Code block highlighting
- [ ] Link handling

#### 12.6.3 Phase 5.3: Keyboard Shortcuts
- [ ] Arboard dependency
- [ ] Copy command output
- [ ] Paste support
- [ ] Cross-platform compatibility

#### 12.6.4 Phase 5.4: Advanced Layouts
- [ ] Notification API integration
- [ ] Focus detection
- [ ] Configurable triggers
- [ ] Cross-platform support

---

### 12.7 Phase 6: Performance & UX Improvements (Codex-RS Analysis)

#### 10.7.1 Context
Analysis of `codex-rs/cli` identified critical gaps in NetToolsKit CLI's performance and UX.

#### 10.7.2 Critical Improvements

| Improvement | Priority | Status | Phase | Description |
|-------------|----------|--------|-------|-------------|
| **IMP-1** | ⭐⭐⭐ | ✅ Complete | Phase 1.2 | Raw Mode Guard - RAII pattern for terminal control |
| **IMP-2** | ⭐⭐⭐⭐⭐ | ✅ Complete | Phase 1.2-2.3 | Event-Driven Architecture - Async operations with progress |
| **IMP-3** | ⭐⭐⭐ | 📋 Planned | Phase 4.1+ | Enhanced Input - Rustyline integration with history |
| **IMP-4** | ⭐⭐ | 🔄 Partial | Phase 2.2-2.3 | Progress Indicators - Indicatif integration planned |
| **IMP-5** | ⭐⭐⭐ | 📋 Planned | Phase 2.5+ | Task Parallelization - JoinSet pattern for concurrency |

#### 10.7.3 Dependencies

**Current:**
- tokio 1.34 (rt-multi-thread, macros, time, net, io-util, sync)
- ratatui 0.28.1 (optional, feature-gated)
- crossterm 0.28.1 (event-stream)
- owo-colors 3.5
- futures 0.3
- clap 4.5
- tracing 0.1

**Planned:**
- rustyline 14.0 (IMP-3)
- indicatif 0.17 (IMP-4)
- tokio-stream 0.1

#### 10.7.4 Success Metrics

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
- Zero panics in interactive mode.

#### 10.7.5 References
- **Analysis:** [comparative-analysis-codex-vs-ntk.md](comparative-analysis-codex-vs-ntk.md)
- **Codex Source:** `codex/codex-rs/tui/src/`
- **Tokio Select:** https://docs.rs/tokio/latest/tokio/macro.select.html
- **Rustyline:** https://github.com/kkawakam/rustyline
- **Indicatif:** https://github.com/console-rs/indicatif

---

## 14. Known Issues

### Deferred Issues
1. **Cursor positioning bug** (modern mode)
   - **Issue:** Prompt returns to top after command.
   - **Status:** Deferred to later phase.
   - **Workaround:** Use legacy mode.
   - **Documented:** `.docs/bugfixes/cursor-position-pending.md`.

### Active Issues
None

---

## 15. References

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

- [task-phase-2.3-command-conversion.md](task-phase-2.3-command-conversion.md)

---

## 16. Workspace Architecture Migration (Rust Workspace Refactoring)

> **⚠️ IMPORTANT**: Phase 6 refers to the **Workspace Architecture Migration** project (separate from CLI feature development Phases 0-5). This is a **parallel track** to refactor the Rust workspace structure from flat to `crates/`-based modular architecture.

> **Parallel Track**: This section tracks the **workspace architecture migration** to refactor the current flat structure into a `crates/`-based modular workspace with 13 crates. This is a **separate initiative** from the CLI feature development tracked above.

**Migration Project:** Workspace-based Modular Monolith
**Sequential Phase:** Phase 6.x (follows CLI Phases 0-5)
**Branch:** `feature/workspace-architecture`
**Version:** 1.0.0
**Started:** 2025-11-09
**Detailed Documentation:** [architecture-migration-plan.md](architecture-migration-plan.md)

> **Note**: For complete details of the layered code architecture, see **Section 1. Code Architecture** at the beginning of this document.

---
