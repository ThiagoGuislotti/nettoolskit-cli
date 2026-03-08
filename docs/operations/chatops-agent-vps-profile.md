# ChatOps Agent VPS Profile (Telegram/Discord)

## Scope

This profile defines the first operational baseline for running a ChatOps-driven NetToolsKit agent on a VPS with **local-first persistence**.

## Current Implementation Slice

- Platform-neutral ChatOps contracts in orchestrator (`chatops` module).
- Remote command parsing to internal task pipeline (`/task submit|list|watch|cancel`).
- Explicit authorization policy with allowlists (user + channel).
- Deterministic notifier and ingress mocks for automated tests.
- Local JSONL audit trail (`chatops/audit.jsonl` under NTK data directory).
- Runtime wiring in `ntk service` (`chatops_runtime` module) with env-driven startup and periodic polling loop.
- Async Telegram/Discord adapters for ingress polling and outbound message notifications.
- Optional Telegram webhook ingress mode (`NTK_CHATOPS_TELEGRAM_WEBHOOK_ENABLED=true`) with local queue ingestion via `POST /chatops/telegram/webhook`.
- Optional Discord interaction ingress mode (`NTK_CHATOPS_DISCORD_INTERACTIONS_ENABLED=true`) with local queue ingestion via `POST /chatops/discord/interactions`.
- Optional ingress hardening for internet exposure: Telegram secret-token validation, Discord Ed25519 request signature validation, and bounded replay protection.
- Optional replay-cache backend selection (`memory`/`file`) with file-backed multi-process sharing for horizontally scaled service replicas.
- Policy-gated repository workflow intent (`repo-workflow`) for `clone -> branch -> execute -> commit -> optional push/PR` automation.
- Scoped command authorization (`NTK_CHATOPS_ALLOWED_COMMANDS`) and per-user/per-channel rate limits.
- Burst-aware throttling strategy options (`fixed_window` / `token_bucket`) with optional burst budgets for high-traffic channels.
- Adaptive rate-limit auto-tuning profile (`NTK_CHATOPS_RATE_LIMIT_AUTOTUNE_PROFILE`) for ingress-driven strategy switching.
- Deterministic VPS-style smoke profile test (`chatops_vps_smoke_profile_*`) wired into CI dual-runtime gate.

## Runtime Security Defaults

- Deny-by-default policy when allowlists are not configured.
- No command execution when user/channel is not explicitly allowed.
- Audit events persisted locally for received/rejected/executed/notified lifecycle.
- Reuse existing task and AI safety controls through internal command routing.
- Repository workflow is deny-by-default and requires explicit host/command allowlists.
- Ingress replay guard keeps a bounded in-memory dedupe window for webhook/interaction payloads.

## Recommended VPS Hardening

- Run service as non-root user.
- Keep host firewall closed except required ingress endpoints.
- Store platform tokens in environment secrets (not in repository).
- Mount persistent local volume for NTK data (`/var/lib/ntk`).
- Keep reverse proxy/TLS termination outside the app process.
- Use reference proxy profiles:
  - `deployments/reverse-proxy/nginx/ntk-chatops.conf.example`
  - `deployments/reverse-proxy/caddy/Caddyfile.example`
  - `docs/operations/chatops-reverse-proxy-profiles.md`

## Suggested Environment Contract

- `NTK_RUNTIME_MODE=service`
- `NTK_SERVICE_AUTH_TOKEN=<secret>`
- `NTK_SERVICE_HTTP_TIMEOUT_MS=30000`
- `NTK_CHATOPS_ENABLED=true`
- `NTK_CHATOPS_ALLOWED_USERS=<id1,id2,...>`
- `NTK_CHATOPS_ALLOWED_CHANNELS=<id1,id2,...>`
- `NTK_CHATOPS_AUDIT_PATH=/var/lib/ntk/chatops/audit.jsonl`
- `NTK_CHATOPS_TELEGRAM_TOKEN=<secret>` (when Telegram adapter is enabled)
- `NTK_CHATOPS_TELEGRAM_WEBHOOK_ENABLED=false` (set `true` to use webhook mode instead of Telegram polling)
- `NTK_CHATOPS_TELEGRAM_WEBHOOK_SECRET_TOKEN=<secret-token>` (recommended with webhook mode)
- `NTK_CHATOPS_DISCORD_TOKEN=<secret>` (when Discord adapter is enabled)
- `NTK_CHATOPS_DISCORD_INTERACTIONS_ENABLED=false` (set `true` to use interaction mode instead of Discord channel polling)
- `NTK_CHATOPS_DISCORD_INTERACTIONS_PUBLIC_KEY=<discord-public-key-hex>` (recommended with interaction mode)
- `NTK_CHATOPS_INGRESS_REPLAY_WINDOW_SECONDS=300`
- `NTK_CHATOPS_INGRESS_REPLAY_MAX_ENTRIES=4096`
- `NTK_CHATOPS_INGRESS_REPLAY_BACKEND=file` (or `memory`)
- `NTK_CHATOPS_INGRESS_REPLAY_FILE_PATH=/var/lib/ntk/chatops/ingress-replay-cache.json` (when backend is `file`)
- `NTK_CHATOPS_DISCORD_CHANNELS=<id1,id2,...>` (required when Discord polling mode is enabled)
- `NTK_CHATOPS_POLL_INTERVAL_MS=3000`
- `NTK_CHATOPS_MAX_BATCH=16`
- `NTK_CHATOPS_ALLOWED_COMMANDS=help,list,watch,cancel,submit:ai-plan,submit:repo-workflow`
- `NTK_CHATOPS_RATE_LIMIT_PER_USER=10`
- `NTK_CHATOPS_RATE_LIMIT_PER_CHANNEL=30`
- `NTK_CHATOPS_RATE_LIMIT_STRATEGY=token_bucket` (or `fixed_window`)
- `NTK_CHATOPS_RATE_LIMIT_AUTOTUNE_PROFILE=balanced` (or `conservative`, `aggressive`, `disabled`)
- `NTK_CHATOPS_RATE_LIMIT_BURST_PER_USER=20` (token bucket only)
- `NTK_CHATOPS_RATE_LIMIT_BURST_PER_CHANNEL=60` (token bucket only)
- `NTK_CHATOPS_RATE_LIMIT_WINDOW_SECONDS=60`
- `NTK_SERVICE_AUTOMATION_PROFILE=balanced`
- `NTK_SERVICE_ALLOWED_INTENTS=ai-plan,ai-explain,ai-apply-dry-run,repo-workflow`
- `NTK_SERVICE_SUBMIT_BUDGET=60`
- `NTK_SERVICE_SUBMIT_WINDOW_SECONDS=60`
- `NTK_SERVICE_MAX_PAYLOAD_BYTES=8192`
- `NTK_SERVICE_MAX_INFLIGHT_TASKS=64`
- `NTK_REPO_WORKFLOW_ENABLED=true`
- `NTK_REPO_WORKFLOW_ALLOWED_HOSTS=github.com`
- `NTK_REPO_WORKFLOW_ALLOWED_COMMANDS=cargo test,cargo fmt,dotnet test`
- `NTK_REPO_WORKFLOW_ALLOW_PUSH=false`
- `NTK_REPO_WORKFLOW_ALLOW_PR=false`
- `NTK_REPO_WORKFLOW_BASE_DIR=/var/lib/ntk/repo-workflow`

Service exposure baseline:

- keep the service listener on `127.0.0.1` whenever possible
- if binding to a non-loopback/private interface, configure `NTK_SERVICE_AUTH_TOKEN`
- keep `GET /health` and `GET /ready` private behind the reverse proxy where practical
- propagate or log `x-request-id` through the reverse proxy for operator troubleshooting

## Command Surface for Remote Operators

- `help`
- `list`
- `watch <task-id>`
- `cancel <task-id>`
- `submit <intent> <payload...>`

Examples:

- `submit ai-plan design CI hardening rollout`
- `submit ai-explain why release gate failed`
- `watch task-1234`
- `submit repo-workflow repo=https://github.com/acme/api.git;branch=feature/chatops;command=cargo test;dry_run=true`

## Automated Smoke Coverage

- CI dual-runtime gate executes ChatOps VPS smoke profile (`cargo test -p nettoolskit-orchestrator --test test_suite chatops_vps_smoke_profile_`).
- Service endpoint tests validate Telegram/Discord ingress paths (`/chatops/telegram/webhook`, `/chatops/discord/interactions`) for valid/invalid payloads and disabled-mode behavior.
- ChatOps `submit` commands now enter the same typed control-plane path as service HTTP task admission, so local audit trails carry normalized request/operator/session/task metadata for remote execution.
- The smoke test uses a local Telegram-compatible mock HTTP server and validates:
  - ingress polling (`getUpdates`)
  - command execution routing (`list` -> `/task list`)
  - outbound notification dispatch (`sendMessage`)
  - local audit trail persistence (`chatops/audit.jsonl`)
- Manual release verification now includes service runtime startup + `/health` smoke on packaged binaries.

## Next Delivery Steps

1. Add hardened sample systemd unit + least-privilege profile for VPS service deployment.
2. Add ingress-volume telemetry dashboard profile for tuning auto-throttle thresholds.
3. Add signed backup/restore runbook for replay cache + audit trail files.