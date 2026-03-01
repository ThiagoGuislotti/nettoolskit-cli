# Incident Response and Troubleshooting Playbook

This playbook defines how NetToolsKit CLI incidents are detected, triaged, mitigated, and reviewed.

## Scope

Applies to:
- Interactive terminal runtime (`crates/cli`, `crates/ui`)
- Command orchestration (`crates/orchestrator`)
- Observability stack (`crates/otel`)
- Release/runtime quality gates (CI and local validation)

Out of scope:
- Third-party SaaS outages outside project control (except dependency impact assessment)

## Incident Severity

Use the highest applicable severity.

| Severity | Definition | Typical Examples | Target Ack | Target Mitigation |
|---|---|---|---|---|
| SEV-1 | Core workflow unavailable or severe data-loss risk | CLI unusable on startup; persistent terminal corruption; security-critical exploit | 15 min | 60 min |
| SEV-2 | Major feature degraded for many users | Command execution failures in common flows; widespread regression in resize behavior | 30 min | 4 h |
| SEV-3 | Limited degradation or workaround available | OTLP export broken while local metrics/logging still work | 4 h | 1 business day |
| SEV-4 | Minor issue, low user impact | Cosmetic rendering defect with no workflow block | 1 business day | Next planned release |

## Roles

| Role | Responsibilities |
|---|---|
| Incident Commander (IC) | Owns timeline, severity, decisions, communications |
| Operations Engineer | Executes mitigations, validates runtime behavior |
| Feature Engineer | Root-cause analysis and code fix |
| Communications Owner | Change log/status updates, user-facing notes |

For small incidents, one person may play multiple roles.

## Detection Signals

Primary signals:
- Failing quality gates: `fmt`, `clippy -D warnings`, `test`, vulnerability audit
- Runtime error spikes: `runtime_commands_error_total`
- Cancellation spikes: `runtime_commands_interrupted_total`
- Latency degradation: `runtime_command_avg_latency_ms` and per-command latency gauges
- User reports with terminal artifacts (duplicated lines, broken cursor state, lost prompt)

## Triage Checklist

1. Confirm incident and assign `SEV`.
2. Capture environment:
- OS and terminal emulator
- CLI version/commit
- command input and reproduction steps
- relevant env vars (`RUST_LOG`, `OTEL_EXPORTER_OTLP_*`, `NTK_OTLP_*`)
3. Start timeline log:
- first detection timestamp
- first mitigation action timestamp
- status updates every 30 minutes for SEV-1/2
4. Determine blast radius:
- all users or specific platform/terminal
- interactive only or non-interactive as well

## Standard Mitigation Workflow

1. Stabilize
- Prefer safe fallback over partial broken state
- Disable optional integrations if needed (for example OTLP export)

2. Contain
- Stop bad release propagation
- Revert or feature-flag risky change if rollback is lower risk

3. Recover
- Apply minimal fix
- Run required validation before release:
```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets
```
- Run vulnerability gate:
```powershell
$SecurityScriptsRoot = Join-Path $env:USERPROFILE '.codex\shared-scripts\security'
pwsh -File (Join-Path $SecurityScriptsRoot 'Invoke-RustPackageVulnerabilityAudit.ps1') -RepoRoot $PWD -ProjectPath . -FailOnSeverities Critical,High
```

4. Verify
- Re-test explicit reproduction scenario
- Validate no regression in terminal cleanup and command flow

## Scenario Runbooks

### A. Terminal Layout Corruption on Resize

Symptoms:
- Duplicate/overlapped lines after terminal/font resize
- Prompt/cursor appears in inconsistent position

Actions:
1. Reproduce with repeated shrink/grow cycles.
2. Validate debounce and reconfigure paths in `crates/ui/src/interaction/terminal.rs`.
3. Confirm no alternate-screen side effects and terminal state restoration.
4. Run focused tests:
```bash
cargo test -p nettoolskit-ui --all-targets
cargo test -p nettoolskit-cli --all-targets
```

Exit criteria:
- No duplicated content after repeated resize
- Cursor visible and prompt stable

### B. Command Error Rate Spike

Symptoms:
- Increase in `runtime_command_error_rate_pct`
- user-facing failures on common commands

Actions:
1. Identify top failing command keys (`runtime_command_<key>_error_total`).
2. Reproduce with command-specific input.
3. Add or adjust regression tests in `crates/orchestrator/tests/execution/processor_tests.rs`.
4. Ship minimal safe patch and revalidate workspace gates.

Exit criteria:
- Error rate returns to baseline
- Regression tests added and passing

### C. Cancellation/Interrupt Spike

Symptoms:
- Increase in `runtime_command_cancellation_rate_pct`
- Frequent interrupted sessions/commands

Actions:
1. Confirm source: user-initiated interrupt vs runtime instability.
2. Inspect `Ctrl+C` handling and terminal cleanup path in `crates/cli/src/lib.rs`.
3. Validate final exit status and terminal persistence behavior.

Exit criteria:
- Cancellation rate normalized or explained by external user behavior
- No terminal corruption on interrupt path

### D. OTLP Export Failure

Symptoms:
- Trace export unavailable while command execution still works

Actions:
1. Check OTLP env configuration and endpoint reachability.
2. Switch to local-only observability mode by unsetting OTLP endpoint env vars.
3. Validate CLI remains healthy with tracing/metrics local.

Exit criteria:
- Core CLI remains fully usable
- OTLP incident isolated and tracked separately

## Communications Template

Use this short format in issue/incident thread:

- Incident: `<title>`
- Severity: `SEV-x`
- Status: `Investigating | Mitigating | Monitoring | Resolved`
- Impact: `<who/what is affected>`
- Started at: `<UTC timestamp>`
- Latest update: `<UTC timestamp>`
- Next update ETA: `<time>`

## Post-Incident Review (Required for SEV-1/2, Recommended for SEV-3)

Within 2 business days:
1. Build timeline (detection, ack, mitigation, recovery).
2. Identify root cause and contributing factors.
3. Define corrective actions with owner and due date.
4. Add changelog decision/update entry when architecture/process changed.
5. Update this playbook if response gaps were found.

## Ownership and Maintenance

- Owner: NetToolsKit maintainers
- Review cadence: monthly or after each SEV-1/2 incident
- Last reviewed: 2026-03-01
