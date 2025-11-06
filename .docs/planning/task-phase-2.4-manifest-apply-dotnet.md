# Phase 2.4: Manifest Apply (.NET) — PLANNED

**Date**: 2025-11-06
**Status**:  Planned
**Version**: 3.0.0

---

## Objective
- Deliver an end-to-end implementation for the `/apply` command that materialises NetToolsKit manifests into .NET solutions using the existing template library.
- Provide deterministic previews via `--dry-run`, enforce collision policies, and surface `// TODO` placeholders for rules that still need manual completion.
- Keep the Rust command runtime responsive, instrumented, and aligned with the Phase 2 async roadmap described in `nettoolskit-cli.md`.

---

## Scope
### In Scope
- Parse and validate NTK solution manifests (`artifact`, `feature`, `layer`) defined in `ntk-manifest-*.yml`.
- Honour manifest directives that require creating brand-new .NET solutions/projects (generate `.sln`, `.csproj`, and initial scaffolding when they do not exist).
- Resolve template mappings under `templates/dotnet/**` and build the payload needed to render each target file.
- Generate change plans (create/update .cs, .csproj, config, and solution files) and honour `policy`/`guards` directives.
- Emit rich CLI feedback (progress lines, diffs on dry-run, success/error summaries) and hook into telemetry counters.
- Add automated coverage (unit + integration tests) driven by the sample manifests and a disposable copy of `samples/src/Rent.Service.*`.

### Out of Scope
- Non-.NET stacks or manifests referencing other template families.
- Full solution surgery (complex .sln merge heuristics, refactors beyond inserting structured `// TODO` markers).
- Plugin system, remote template registries, or manifest authoring tooling.
- End-to-end async executor adoption (kept synchronous for now; async integration tracked separately in Phase 2.5).

---

## Reference Material
- Manifests: [`../ntk-manifest-artifact.yml`](../ntk-manifest-artifact.yml), [`../ntk-manifest-feature.yml`](../ntk-manifest-feature.yml), [`../ntk-manifest-layer.yml`](../ntk-manifest-layer.yml)
- Templates: [`../../templates/dotnet`](../../templates/dotnet)
- Roadmap context: [`nettoolskit-cli.md`](nettoolskit-cli.md)
- Existing stub: `commands/src/apply.rs`
- Supporting utilities: `file-search/src/search.rs`, `commands/src/async_executor.rs`

---

## Success Criteria
- `/apply` loads manifests from disk (YAML), validates metadata/guards, and resolves the targeted solution directory.
- `--dry-run` prints a stable diff/summary without touching the filesystem; execution mode creates or amends files according to `policy`.
- Template rendering uses real data expansion, generating compilable .NET code for the provided samples with `// TODO` markers where manual logic is still required.
- Collision policies (`fail`, later `overwrite`) and `insertTodoWhenMissing` flags are honoured and logged.
- Automated tests cover manifest parsing, rule expansion, template selection, dry-run summaries, and a happy-path application against the Rent.Service sample.

---

## Workstreams and Tasks

### Workstream A — Manifest Ingestion and Validation
1. Add `commands/src/apply/mod.rs` (or expand `apply.rs`) with submodules for `manifest`, `guards`, and `plan`.
2. Introduce `serde_yaml` to the workspace and define strongly typed structures:
   - `ManifestDocument`, `ManifestMeta`, `ManifestProjects`, `ManifestContext`, `ApplyMode`.
   - Optional helper enums for `policy.collision`, `kind`, and `apply.mode`.
3. Implement `load_manifest(path: &Path) -> Result<ManifestDocument>` that:
   - Normalises relative paths using `args.target` or current working directory.
   - Validates `apiVersion` and `kind`.
   - Emits actionable errors via `CommandError`.
4. Implement guard checks:
   - Confirm solution root and `.sln` existence (`solution.guard`).
   - Validate project folders referenced in `projects`.
   - Enforce `requireExistingProjects` semantics (fail fast when missing).
5. Derive an internal `ManifestSummary` object (context count, template count, target projects) for telemetry/logging.

### Workstream B - Template Resolution and Data Expansion
1. Build a `TemplateRegistry` that scans the `templates/dotnet` tree once and caches file metadata (reuse `file-search` helpers).
2. Parse `templates.mapping` entries and verify that each `artifact` reference has a matching template file.
3. Evaluate `render.rules` expressions:
   - Use `serde_json::Value` plus `jsonpath_lib` (or a lightweight visitor) to expand selectors such as `contexts[*].aggregates[*].entities[*]`.
   - Map each expansion to a `RenderTask { artifact, template, destination, data }`.
   - Record unsupported selectors and mark them with a structured TODO entry.
4. Enrich render data with derived fields (namespace, folder, tokens) required by templates:
   - Leverage manifest conventions (`namespaceRoot`, `targetFramework`) for repeated values.
   - Provide casing helpers and keep them centralised for reuse.
5. Honour manifest-level overrides (`apply.mode`, `artifact.context`, etc.) when building the final list of tasks.

### Workstream C - Change Plan Generation and Execution
1. Define a `ChangePlan` that separates `CreateFile`, `UpdateFile`, `AppendToSolution`, and `NoOp` actions.
2. Implement `plan_changes(tasks, target_root, policy)`:
   - Compute target paths (support placeholders `{Name}`, `{UseCase}`, etc.).
   - Detect collisions; on `fail`, abort with error; on unsupported policies, insert `// TODO: handle policy <name>` and continue if safe.
   - When `insertTodoWhenMissing` is true and template sections would be empty, inject canonical TODO comments in rendered output.
3. Introduce explicit handlers for solution/project scaffolding:
   - Generate `.sln` skeletons and register projects when manifests reference new solutions.
   - Create `.csproj` files from templates when projects are missing, wiring default SDK/TFM metadata.
   - Ensure project-to-solution linkage is captured in the plan before writing to disk.
4. Implement dry-run summariser:
   - Produce ascii table or bullet list of intended operations.
   - Optionally render unified diff using `similar` crate (add dependency if needed).
5. Implement executor:
   - Create directories lazily.
   - Write files atomically (temp file + rename) and preserve UTF-8 without BOM.
   - Update `.sln` using minimal string insertion; defer complex manipulations via explicit TODO markers and console guidance.
6. Emit telemetry and tracing events per operation for observability (`command_apply_*` counters).

### Workstream D — CLI UX and Integration
1. Replace the stubbed `apply::run` implementation with the real pipeline:
   - Input resolution → manifest load → validation → plan → dry-run or execute.
   - Print phase headers and progress markers (keep ASCII-friendly).
2. Integrate with metrics:
   - Increment success/failure counters.
   - Record duration using `Timer`.
3. Respect `--dry-run` flag and return `ExitStatus::Success` even when work is skipped intentionally.
4. Provide actionable error messages and guidance when encountering TODO placeholders (e.g., link back to manifest rule).
5. Prepare the command API for future async adaptation (structure steps so they can move into `AsyncCommandExecutor` later without major refactor).

### Workstream E — Testing, Samples, and Documentation
1. Unit tests:
   - Manifest parsing/validation (cover happy path and failure cases).
   - Selector expansion and template lookups.
   - Change plan collision handling.
2. Integration test:
   - Use `tempfile` to clone `samples/src/Rent.Service.*`, run `/apply` with each sample manifest, assert file outputs exist and contain TODO markers where expected.
3. CLI snapshot test (feature gate) to validate dry-run output remains stable.
4. Update `.docs/ntk-manifest-*.yml` with inline notes on any new required fields or TODO behaviours.
5. Record progress in `nettoolskit-cli.md` once implementation starts, keeping this plan as the authoritative checklist.

---

## Data Contracts to Introduce
- `ManifestDocument`: wraps metadata, conventions, solution details, projects, contexts, templates, render rules, and apply mode.
- `ApplyMode`: enum with variants `Artifact { kind, context, name }`, `Feature { context, include }`, `Layer { include }`.
- `TemplateMapping`: holds manifest `artifact`, template path, destination, and optional condition.
- `RenderTask`: resolved combination of mapping + expanded data + resolved destination.
- `ChangePlan` variants (`CreateFile`, `UpdateFile`, `ModifySolution`, `Skip`) with reason codes.
- `SolutionScaffold` and `ProjectDescriptor`: capture creation parameters for new `.sln`/`.csproj` artifacts driven by manifests.
- `PlanOutcome`: summarises counts, TODO injections, and collisions for reporting.

All structs derive `serde::Deserialize` where applicable and live under `commands::apply`.

---

## CLI Experience and Telemetry
- Step-by-step console output:
  1. Locate manifest and target solution.
  2. Validate projects/guards/policies.
  3. Preview change plan (table listing file path, action, template, status).
  4. Execute or skip (for dry-run) with per-file lines (`[create] path`, `[update] path`).
- On completion, print summary counts and reminder when TODO markers were inserted.
- Emit tracing spans (`apply.load_manifest`, `apply.plan`, `apply.execute`) so async integration can piggyback later.

---

## Testing Strategy
- Use `cargo test -p nettoolskit_commands` as gate; add category markers when extending existing suites.
- Mock filesystem interactions with `tempfile` and `assert_fs`.
- Snapshot expected file contents (Golden files stored under `tools/nettoolskit-cli/commands/tests/data/manifests/*`).
- Validate dry-run output using insta or textual comparison while keeping ASCII output deterministic.
- Ensure tests clean up and run on Windows + Linux (path separator handling).

---

## TODO Boundaries
- Complex `.sln` merge rules, cross-project reference wiring, or EF Core scaffolding must emit `// TODO` placeholders in generated files and list them in the CLI summary.
- Unsupported `policy.collision` modes should log a TODO note and exit with error until implemented.
- Feature-specific business logic (e.g., repository implementations) remains manual and should be annotated accordingly.

---

## Risks and Mitigations
- **YAML selector complexity**: Start with the documented patterns; guard with clear error messages and TODOs when encountering unsupported expressions.
- **File collisions**: Default to fail-fast per manifest policy; provide explicit instructions on how to override.
- **Template drift**: Add unit tests that verify template paths exist to catch renames early.
- **Cross-platform paths**: Normalise separators and ensure generated code uses `\n` line endings regardless of host OS.
- **Performance**: Cache template registry and reuse `handlebars` instance to avoid recompilation on each file.

---

## Deliverables
- Updated `commands/src/apply.rs` plus supporting modules for manifest ingestion and plan execution.
- New tests under `commands/tests/**` and golden assets for sample manifests.
- Documentation updates (plan status notes, CLI docs if needed).
- Console output examples captured for future README inclusion.

---

## Follow-up Items
- Track async executor integration for `/apply` (Phase 2.5).
- Evaluate reuse of manifest engine for `/check` command enhancements.
- Catalogue additional template families (e.g., workers) once .NET path stabilises.

---

**Document Version**: 0.1
**Last Updated**: 2025-11-03
**Next Review**: Start of implementation (Phase 2.4 kickoff)
