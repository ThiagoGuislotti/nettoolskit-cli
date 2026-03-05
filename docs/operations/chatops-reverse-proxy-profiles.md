# ChatOps Reverse Proxy Profiles (Nginx/Caddy)

## Purpose

Provide reference reverse-proxy profiles for internet-exposed ChatOps ingress endpoints while preserving service-side security checks.

## Reference Files

- `deployments/reverse-proxy/nginx/ntk-chatops.conf.example`
- `deployments/reverse-proxy/caddy/Caddyfile.example`

## Required Service Environment

Configure service ingress hardening:

- `NTK_CHATOPS_TELEGRAM_WEBHOOK_SECRET_TOKEN=<secret-token>`
- `NTK_CHATOPS_DISCORD_INTERACTIONS_PUBLIC_KEY=<discord-public-key-hex>`
- `NTK_CHATOPS_INGRESS_REPLAY_WINDOW_SECONDS=300`
- `NTK_CHATOPS_INGRESS_REPLAY_MAX_ENTRIES=4096`
- `NTK_CHATOPS_INGRESS_REPLAY_BACKEND=file`
- `NTK_CHATOPS_INGRESS_REPLAY_FILE_PATH=/var/lib/ntk/chatops/ingress-replay-cache.json`
- `NTK_CHATOPS_RATE_LIMIT_AUTOTUNE_PROFILE=balanced` (optional adaptive throttling)

## Why Header Preservation Matters

Service-side validation depends on original ingress headers:

- Telegram webhook: `X-Telegram-Bot-Api-Secret-Token`
- Discord interactions: `X-Signature-Ed25519` and `X-Signature-Timestamp`

If the proxy drops or rewrites these values, ingress requests are rejected (`401`).

## Deployment Notes

- Keep `ntk service` bound to loopback/private network (`127.0.0.1:8080`).
- Expose only required endpoints:
  - `POST /chatops/telegram/webhook`
  - `POST /chatops/discord/interactions`
- Keep `/health` and `/ready` private when possible.
- Deny all other public paths by default.

## Validation Checklist

1. Start service mode and proxy.
2. Send a Telegram webhook with correct secret token header; expect success (`202`).
3. Send the same webhook payload again within replay window; expect conflict (`409`).
4. Send Discord interaction with valid signature/timestamp headers; expect success.
5. Send Discord interaction without signature headers; expect unauthorized (`401`).

## Security Baseline

- TLS termination at proxy.
- Strict endpoint allowlist.
- Service-level signature/origin validation.
- Bounded replay window.
- Local audit trail remains enabled in service runtime.