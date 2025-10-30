# NetToolsKit.CLI — Implementation Plan (v0.1.0)

> Multi‑stack code generator based on **static templates**. CLI written in **Rust**. No Roslyn in this phase. Placeholders with `{{Tokens}}`, collision policy, `--dry-run` with unified diff, and optional insertion of `// TODO` + `NotImplementedException` when optional sections are empty.

---

## 1. Purpose
Deliver a single binary `ntk` that scaffolds and expands projects and files for **.NET**, **Vue/Quasar**, **Clojure**, and **Rust** from versioned **manifests** and **templates**, with safety (idempotency), predictability (show diffs before write), and maintainability.

## 2. In‑scope (v0.1.0)
- Rust CLI with subcommands: `list`, `check`, `new`, `render`, **`apply`**.
- Template engine: **Handlebars** in strict mode.
- Manifest per template and YAML **solution manifests**.
- Initial stacks: `.NET (background-service, api)`, `Vue/Quasar (app)`, `Clojure (app)`, `Rust (lib)`.
- Write collision policy: `fail` | `safe` | `force`.
- `--dry-run` prints unified diffs.
- Idempotency markers: `// <ntk:region ...>`.
- Optional post‑steps triggered with `--with-post`.

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
- **FR01** List templates.
- **FR02** Check template/manifest.
- **FR03** Render from variables.
- **FR04** Dry‑run diff.
- **FR05** Write with collision policy.
- **FR06** Idempotent regeneration.
- **FR07** Insert TODOs for optional gaps.
- **FR08** Run post‑steps.
- **FR09** Project‑level defaults via `.ntkrc.json`.
- **FR10** Apply manifest as **feature slice** (context + selected layers).
- **FR11** Apply manifest as **layer‑only**.
- **FR12** Apply manifest as **artifact‑only** (e.g., entity or endpoint).
- **FR13** Support **existing solution guards** (`requireExistingProjects`, `onMissingProject`).

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
- **WBS‑1 CLI Core**: clap, flags, output formatters.
- **WBS‑2 Template Engine**: Handlebars helpers and escaping.
- **WBS‑3 Manifest & Validation**: YAML loader + schema validation.
- **WBS‑4 File Writer**: collision strategies, diff generator, backups.
- **WBS‑5 Idempotency/TODO**: region markers and injections.
- **WBS‑6 Apply Engine**: selectors for full/feature/layer/artifact; guards.
- **WBS‑7 Initial Templates**: .NET, Vue/Quasar, Clojure, Rust.
- **WBS‑8 Tests & CI**: snapshots, collisions, dry‑run; GitHub Actions.
- **WBS‑9 Docs & Release**: README, plan, templates guide; v0.1.0 tag.

---

## 10. Milestones & Acceptance
- **M0 Skeleton**: `ntk --help`, `ntk list`.
- **M1 Engine**: `ntk render` with `--var` and `--input`.
- **M2 Validation**: `ntk check` validates schema.
- **M3 Write/Collision/Diff**: `ntk new` + `--dry-run` diffs.
- **M4 Idempotency/TODO**: re‑runs do not duplicate; TODOs inserted.
- **M5 Templates**: 4 stacks compile.
- **M6 Apply Engine**: full/feature/layer/artifact work with guards; diffs correct.
- **M7 Tests/CI**: green on 3 OSes.
- **M8 Docs/Release**: docs complete; binaries shipped; tag `v0.1.0`.