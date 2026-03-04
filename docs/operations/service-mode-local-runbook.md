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

## Submit a Task by HTTP

```bash
curl -sS -X POST http://127.0.0.1:8080/task/submit \
  -H "Content-Type: application/json" \
  -d '{"intent":"ai-plan","payload":"create roadmap for service hardening"}'
```

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

## Planned Next Expansion (ChatOps Agent)

Planned but not delivered in this runbook:

- Telegram adapter for command ingress and notifications.
- Discord adapter for command ingress and notifications.
- Repository job runner workflow (`clone -> branch -> execute -> commit/push/PR`) with policy gates.