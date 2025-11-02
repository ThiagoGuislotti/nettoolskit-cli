# NetToolsKit.CLI — Implementation Plan (v0.1.0)

> Multi‑stack code generator based on **static templates**. CLI written in **Rust**. No Roslyn in this phase. Placeholders with `{{Tokens}}`, collision policy, `--dry-run` with unified diff, and optional insertion of `// TODO` + `NotImplementedException` when optional sections are empty.

---

## 1. Purpose
Deliver a single binary `ntk` that scaffolds and expands projects and files for **.NET**, **Vue/Quasar**, **Clojure**, and **Rust** from versioned **manifests** and **templates**, with safety (idempotency), predictability (show diffs before write), and maintainability.

## 2. In-scope (v0.1.0)

**CLI Core & Workflow**
- [x] Rust CLI with subcommands: `list`, `check`, `new`, `render`, **`apply`**.
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
- [ ] Configurable logging levels (enable/disable footer output, verbosity presets).
- [ ] Clear command to reset terminal back to initial header/logo layout.
- [ ] Persistent input prompt after command completion (cursor always below latest output).

**Quality & Insights**
- [ ] Test coverage sweep with coverage graph generation.

## 3. Out‑of‑scope (v0.1.0)
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

**Quality & Delivery**
- **WBS-8 Tests & CI**: [ ] Snapshot + collision suites, coverage gating, GitHub Actions.
- **WBS-14 Coverage Insights**: [ ] Test sweep automation, coverage graph export/presentation.
- **WBS-9 Docs & Release**: [ ] README, plan updates, templates guide, release packaging.

> **Execution order hint:** complete *Foundation* tasks before tackling *Generation Features*, so apply workflows have a stable engine. Finalize *Terminal Experience* improvements once the apply pipeline exists, then close with *Quality & Delivery* to validate and ship.

---

## 10. Milestones & Acceptance
- **M0 Skeleton**: [ ] `ntk --help`, `ntk list`.
- **M1 Rendering Engine**: [ ] `ntk render` with `--var/--vars-file/--output`; Handlebars strict mode.
- **M2 Validation & Manifests**: [ ] `ntk check` schema validation; manifest parsing basics.
- **M3 Writing & Collisions**: [ ] `ntk new` honoring collision policy; `--dry-run` diffs.
- **M4 Idempotent Apply**: [ ] region markers, TODO insertion, guards enforced.
- **M5 Template Library**: [ ] project/solution/class templates for four stacks compile.
- **M6 Terminal Polish**: [ ] logging config, `/clear`, stable input prompt.
- **M7 Quality Gates**: [ ] coverage sweep with graph, CI green on 3 OSes.
- **M8 Docs & Release**: [ ] docs complete; binaries signed/shipped; tag `v0.1.0`.

---

## 11. Performance & UX Improvements (Based on Codex-RS Analysis)

### 11.1 Context
Analysis of `codex-rs/cli` identified critical gaps in NetToolsKit CLI's performance and UX. See detailed comparison in `comparison-codex-vs-ntk.md`.

### 11.2 Critical Improvements (Phase 1 - Foundation)

#### **IMP-1: Raw Mode Guard** ⭐⭐⭐
**Problem:** Current implementation toggles raw mode on/off for every command, causing overhead and potential flickering.

**Current State:**
```rust
// cli/src/lib.rs - runs every command cycle
enable_raw_mode()?;
let input = read_line().await?;
disable_raw_mode()?;
```

**Target State:**
```rust
pub struct RawModeGuard;
impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
    }
}
```

**Tasks:**
- [ ] Create `RawModeGuard` in `cli/src/lib.rs`
- [ ] Replace manual enable/disable with guard pattern
- [ ] Add RAII cleanup on panic/exit

**Priority:** High | **Status:** ✅ Completed

---

#### **IMP-2: Event-Driven Architecture** ⭐⭐⭐
**Problem:** Blocking event loop prevents async operations during user input, no progress feedback for long operations.

**Current State:**
```rust
// cli/src/lib.rs - sequential blocking
async fn run_interactive_loop() -> io::Result<ExitStatus> {
    loop {
        let input = read_line().await?;  // Blocks here
        process(input).await;
    }
}
```

**Target State (Implemented with Preservation Strategy):**
```rust
use events::{CliEvent, EventSender};
use tokio::sync::mpsc;

async fn run_interactive_loop() -> io::Result<ExitStatus> {
    let mut input_buffer = String::new();
    let mut palette = CommandPalette::new();
    let _raw_mode_guard = RawModeGuard::new()?;

    // Event-driven Ctrl+C handling
    let (interrupt_tx, mut interrupt_rx) = mpsc::unbounded_channel();
    let interrupt_sender = EventSender::new(interrupt_tx);
---

#### **IMP-3: Enhanced Input Handling** ⭐⭐⭐
**Problem:** Basic readline without history persistence, auto-complete, or multi-line editing.

**Current State:**
```rust
// cli/src/input.rs - minimal functionality
pub async fn read_line_with_palette(...) -> InputResult {
    // Basic character-by-character reading
}
```

**Target State:**
```rust
use rustyline::{Editor, Config, CompletionType};

pub struct InteractiveShell {
    editor: Editor<CommandCompleter>,
    history_path: PathBuf,
}

impl InteractiveShell {
    pub fn new() -> Result<Self> {
        let config = Config::builder()
            .completion_type(CompletionType::List)
            .auto_add_history(true)
            .build();

        let mut editor = Editor::with_config(config)?;
        editor.set_helper(Some(CommandCompleter::new()));

        let history_path = dirs::config_dir()
            .unwrap_or_default()
            .join("nettoolskit")
            .join("history.txt");

        if history_path.exists() {
            let _ = editor.load_history(&history_path);
        }

        Ok(Self { editor, history_path })
    }
}
```

**Tasks:**
- [ ] Add `rustyline` dependency (14.0)
- [ ] Implement `InteractiveShell` wrapper
- [ ] Create `CommandCompleter` for palette
- [ ] Add persistent history to `~/.config/nettoolskit/history.txt`
- [ ] Integrate with event system (IMP-2)
- [ ] **Preserve** CommandPalette (`/` triggers, navigation, Tab completion)

**Priority:** High | **Status:** Dependencies Ready, Implementation Pending
- [ ] Integrate with event loop

**Priority:** High | **Status:** Planned

---

#### **IMP-4: Progress Indicators** ⭐⭐
**Problem:** No feedback during long operations (template rendering, file I/O).

**Target State:**
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
- [ ] Add `indicatif` dependency (0.17)
- [ ] Create progress bar helpers in `ui/src/progress.rs`
- [ ] Integrate with `apply` command
- [ ] Add spinner for indeterminate operations
- [ ] Test with event system (IMP-2) for non-blocking progress

**Priority:** Medium | **Status:** Dependencies Ready, Implementation Pending

---

### 11.3 High Priority (Phase 2 - Concurrency)

#### **IMP-5: Task Spawning & Parallelization** ⭐⭐⭐
**Problem:** Sequential processing blocks CLI during heavy operations.

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
- [ ] Identify parallelizable operations
- [ ] Implement `JoinSet` pattern for validation
- [ ] Add concurrent template loading
- [ ] Create progress aggregation for parallel tasks
- [ ] Add timeout handling

**Priority:** High | **Status:** Planned

---

### 11.4 Dependencies Updates

**Add:**
```toml
[dependencies]
# Event streaming
tokio-stream = "0.1"

# Enhanced input
rustyline = "14.0"

# Progress indicators
indicatif = "0.17"

# Async utilities (if not present)
futures = "0.3"
```

**Remove (if not using TUI):**
```toml
# Currently included but underutilized
ratatui = "0.29"  # Only remove if not implementing full TUI
```

---

### 11.5 Implementation Phases

#### **Phase 1: Foundation**
**Goal:** Resolve critical performance gaps

**Tasks:**
- [x] IMP-1: RawModeGuard implementation
- [ ] IMP-2: Event-driven architecture setup
- [ ] IMP-2: Event loop migration
- [ ] IMP-3: Rustyline integration
- [ ] IMP-4: Basic progress indicators

**Deliverable:** Responsive CLI with professional input handling

---

#### **Phase 2: Concurrency**
**Goal:** Parallelize heavy operations

**Tasks:**
- [ ] Identify parallelizable operations
- [ ] IMP-5: JoinSet patterns implementation
- [ ] Parallel validation and template loading
- [ ] Progress aggregation for concurrent tasks

**Deliverable:** Non-blocking template operations

---

#### **Phase 3: Polish**
**Goal:** UX refinements and testing

**Tasks:**
- [ ] Enhanced progress indicators
- [ ] Keyboard shortcuts documentation
- [ ] Integration testing
- [ ] Performance benchmarks

**Deliverable:** Production-ready CLI

---

### 11.6 Success Metrics

**Performance:**
- [ ] Startup time < 100ms (current: ~50ms ✅)
- [ ] Input latency < 16ms (60 FPS responsiveness)
- [ ] Template rendering: 100+ files without blocking UI

**UX:**
- [ ] Auto-complete functional
- [ ] Persistent history
- [ ] Graceful Ctrl+C handling
- [ ] Progress feedback for ops > 1s

**Code Quality:**
- [ ] Test coverage > 70%
- [ ] Benchmarks for critical operations
- [ ] Zero panics in interactive mode

### 11.7 References
- **Analysis Document:** `comparison-codex-vs-ntk.md`
- **Codex Source:** `codex/codex-rs/tui/src/`
- **Tokio Select:** https://docs.rs/tokio/latest/tokio/macro.select.html
- **Rustyline:** https://github.com/kkawakam/rustyline