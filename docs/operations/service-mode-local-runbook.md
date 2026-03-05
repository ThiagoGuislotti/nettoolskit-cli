# Service Mode Local Runbook

## Purpose

Run NetToolsKit in background service mode using Docker for local or VPS-style operation, with health checks and local-first persistence.

## Prerequisites

- Docker Engine 24+
- Docker Compose v2
- Available port `8080`

## Files

- `deployments/Dockerfile.service`
- `deployments/docker-compose.local.yml`
- `deployments/service.local.env.example`
- `deployments/reverse-proxy/nginx/ntk-chatops.conf.example`
- `deployments/reverse-proxy/caddy/Caddyfile.example`
- `docs/operations/chatops-reverse-proxy-profiles.md`

## Start Service Mode

From repository root:

```bash
docker compose -f deployments/docker-compose.local.yml up --build -d
```

## Validate Health

```bash
curl -fsS http://127.0.0.1:8080/health
```

Expected JSON fields:

- `status`: `ok`
- `runtime_mode`: `service`
- `uptime_seconds`
- `version`

Core endpoints:

- `GET /health`
- `GET /ready`
- `POST /task/submit`
- `POST /chatops/telegram/webhook` (only when Telegram webhook mode is enabled)
- `POST /chatops/discord/interactions` (only when Discord interaction mode is enabled)

## Submit a Task by HTTP

```bash
curl -sS -X POST http://127.0.0.1:8080/task/submit \
  -H "Content-Type: application/json" \
  -d '{"intent":"ai-plan","payload":"create roadmap for service hardening"}'
```

## Telegram Webhook Mode (Optional)

Enable webhook ingress mode for Telegram as an alternative to polling:

- `NTK_CHATOPS_ENABLED=true`
- `NTK_CHATOPS_TELEGRAM_TOKEN=<secret>`
- `NTK_CHATOPS_TELEGRAM_WEBHOOK_ENABLED=true`
- `NTK_CHATOPS_TELEGRAM_WEBHOOK_SECRET_TOKEN=<secret-token>` (recommended for internet-exposed endpoints)
- `NTK_CHATOPS_ALLOWED_USERS=<id1,id2,...>`
- `NTK_CHATOPS_ALLOWED_CHANNELS=<id1,id2,...>`

Submit a Telegram update payload directly to the service:

```bash
curl -sS -X POST http://127.0.0.1:8080/chatops/telegram/webhook \
  -H "Content-Type: application/json" \
  -H "X-Telegram-Bot-Api-Secret-Token: <secret-token>" \
  -d '{"update_id":10,"message":{"date":1737200000,"text":"list","chat":{"id":555},"from":{"id":777}}}'
```

## Discord Interaction Mode (Optional)

Enable interaction ingress mode for Discord as an alternative to channel polling:

- `NTK_CHATOPS_ENABLED=true`
- `NTK_CHATOPS_DISCORD_TOKEN=<secret>`
- `NTK_CHATOPS_DISCORD_INTERACTIONS_ENABLED=true`
- `NTK_CHATOPS_DISCORD_INTERACTIONS_PUBLIC_KEY=<discord-public-key-hex>` (recommended for internet-exposed endpoints)
- `NTK_CHATOPS_ALLOWED_USERS=<id1,id2,...>`
- `NTK_CHATOPS_ALLOWED_CHANNELS=<id1,id2,...>`

Submit an interaction payload directly to the service:

```bash
curl -sS -X POST http://127.0.0.1:8080/chatops/discord/interactions \
  -H "Content-Type: application/json" \
  -d '{"type":2,"channel_id":"555","member":{"user":{"id":"777"}},"data":{"name":"list"}}'
```

Note: when `NTK_CHATOPS_DISCORD_INTERACTIONS_PUBLIC_KEY` is set, requests must include valid Discord signature headers (`X-Signature-Ed25519`, `X-Signature-Timestamp`) or the endpoint returns `401`.

## Persistence

Local data is persisted to:

- Host path: `./.temp/service-data`
- Container path: `/var/lib/ntk`

This keeps runtime data local-first and portable for VPS backups.

## Logs and Status

```bash
docker compose -f deployments/docker-compose.local.yml ps
docker compose -f deployments/docker-compose.local.yml logs -f ntk-service
```

## Stop Service

```bash
docker compose -f deployments/docker-compose.local.yml down
```

## Troubleshooting

- Port already in use:
  - Change host port in `deployments/docker-compose.local.yml` (for example `8081:8080`).
- Healthcheck failing:
  - Verify container is running and check logs.
  - Run `curl http://127.0.0.1:8080/health` from host.
- Task submit returns 400:
  - Ensure JSON payload contains both `intent` and `payload`.

## Security Notes

- Keep service behind firewall/VPN in VPS environments.
- Do not expose service directly to the public internet without authentication/reverse proxy.
- For mutating AI flows, keep explicit approval and dry-run safeguards enabled.
- For internet-exposed ChatOps ingress, use one of the reverse-proxy reference profiles and keep ingress security envs enabled.

## Repository Workflow Policy (Service Mode)

Repository automation via `/task submit repo-workflow ...` is available behind explicit policy gates:

- `NTK_REPO_WORKFLOW_ENABLED=true`
- `NTK_REPO_WORKFLOW_ALLOWED_HOSTS=github.com,gitlab.com`
- `NTK_REPO_WORKFLOW_ALLOWED_COMMANDS=cargo test,cargo fmt,dotnet test`
- `NTK_REPO_WORKFLOW_ALLOW_PUSH=false` (default)
- `NTK_REPO_WORKFLOW_ALLOW_PR=false` (default)
- `NTK_REPO_WORKFLOW_BASE_DIR=/var/lib/ntk/repo-workflow`

By default (`false`/empty), workflow execution is denied.

## Service Automation Policy Profile

Service mode now supports explicit automation policy profiles (allowed actions + budgets) before task queue admission:

- `NTK_SERVICE_AUTOMATION_PROFILE=balanced`
  - Supported values: `strict`, `balanced` (default), `open`
- `NTK_SERVICE_ALLOWED_INTENTS=ai-ask,ai-plan,ai-explain,ai-apply-dry-run,repo-workflow`
  - Use `*` (or `all`) to allow every intent
- `NTK_SERVICE_SUBMIT_BUDGET=60`
- `NTK_SERVICE_SUBMIT_WINDOW_SECONDS=60`
- `NTK_SERVICE_MAX_PAYLOAD_BYTES=8192`
- `NTK_SERVICE_MAX_INFLIGHT_TASKS=64`

Profile defaults:

- `strict`: denies `command` and `repo-workflow`, tighter budgets.
- `balanced`: allows AI intents + `repo-workflow`, denies raw `command`.
- `open`: allows all intents with higher budgets (use only in trusted/private environments).

For ChatOps remote control hardening, also configure:

- `NTK_CHATOPS_TELEGRAM_WEBHOOK_ENABLED=true` (optional alternative to Telegram polling)
- `NTK_CHATOPS_DISCORD_INTERACTIONS_ENABLED=true` (optional alternative to Discord polling)
- `NTK_CHATOPS_TELEGRAM_WEBHOOK_SECRET_TOKEN=<secret-token>`
- `NTK_CHATOPS_DISCORD_INTERACTIONS_PUBLIC_KEY=<discord-public-key-hex>`
- `NTK_CHATOPS_INGRESS_REPLAY_WINDOW_SECONDS=300`
- `NTK_CHATOPS_INGRESS_REPLAY_MAX_ENTRIES=4096`
- `NTK_CHATOPS_INGRESS_REPLAY_BACKEND=file` (or `memory`)
- `NTK_CHATOPS_INGRESS_REPLAY_FILE_PATH=/var/lib/ntk/chatops/ingress-replay-cache.json` (when backend is `file`)
- `NTK_CHATOPS_ALLOWED_COMMANDS=help,list,watch,cancel,submit:ai-plan,submit:repo-workflow`
- `NTK_CHATOPS_RATE_LIMIT_PER_USER=10`
- `NTK_CHATOPS_RATE_LIMIT_PER_CHANNEL=30`
- `NTK_CHATOPS_RATE_LIMIT_STRATEGY=token_bucket` (or `fixed_window`)
- `NTK_CHATOPS_RATE_LIMIT_AUTOTUNE_PROFILE=balanced` (or `conservative`, `aggressive`, `disabled`)
- `NTK_CHATOPS_RATE_LIMIT_BURST_PER_USER=20` (token bucket only)
- `NTK_CHATOPS_RATE_LIMIT_BURST_PER_CHANNEL=60` (token bucket only)
- `NTK_CHATOPS_RATE_LIMIT_WINDOW_SECONDS=60`

Ingress security behavior:

- Telegram webhook rejects missing or mismatched secret tokens with `401`.
- Discord interactions reject missing/invalid signatures or stale timestamps with `401`.
- Replay-detected ingress payloads return `409` within configured replay window.
- Replay backend failures return `503` (`Service Unavailable`).

Replay backend notes:

- `memory` backend is process-local and suitable for single-instance service mode.
- `file` backend enables replay sharing across multiple service processes/replicas when they share the same persistent volume path.

Rate-limit auto-tuning behavior:

- When auto-tuning profile is enabled, strategy may switch between `fixed_window` and `token_bucket` based on sustained observed ingress traffic.
- Conservative profile requires longer sustained changes; aggressive profile reacts faster.

## CI/Release Smoke Coverage

- CI dual-runtime gate validates service mode plus ChatOps VPS smoke flow:
  - `cargo test -p nettoolskit-orchestrator --test test_suite chatops_vps_smoke_profile_`
- Manual release verification workflow now performs packaged-binary service startup and `/health` smoke checks on Linux, Windows, and macOS.