# Release Artifact Verification Guide

This runbook explains how to verify official `ntk` release artifacts after a tag is published.

## Purpose

The release pipeline publishes:

- packaged binaries (`.tar.gz` / `.zip`)
- checksums (`.sha256`)
- SBOMs (`.cyclonedx.json` / `.spdx.json`)
- signatures (`.sig`)
- signing certificates (`.pem`)

Verification should check both integrity (checksum) and provenance (cosign keyless certificate).

## Prerequisites

- GitHub CLI (`gh`)
- `cosign` CLI
- Hash utility:
  - Linux/macOS: `sha256sum` or `shasum`
  - Windows: `Get-FileHash` (PowerShell)

## Automated verification workflow (recommended)

This repository provides a manual GitHub Actions workflow for published release validation:

- Workflow: `.github/workflows/release-verify.yml`
- Trigger: `workflow_dispatch`
- Input: `tag` (example: `v1.0.0`)

It validates:

- release tag format
- artifact signature/certificate (`cosign verify-blob`) for binaries, checksums, and SBOMs
- artifact checksum (`.sha256`)
- SBOM metadata sanity (`CycloneDX` and `SPDX` format headers)
- packaged binary smoke commands (`--version`, `--help`, `manifest --help`)

## 1. Download assets for a release tag

Example for tag `v1.0.0`:

```bash
TAG="v1.0.0"
gh release download "$TAG" --repo ThiagoGuislotti/nettoolskit-cli --pattern "ntk-*"
```

## 2. Verify checksum integrity

Linux/macOS:

```bash
sha256sum -c ntk-1.0.0-x86_64-unknown-linux-gnu.tar.gz.sha256
```

If `sha256sum` is unavailable on macOS:

```bash
shasum -a 256 -c ntk-1.0.0-x86_64-unknown-linux-gnu.tar.gz.sha256
```

Windows PowerShell:

```powershell
$asset = "ntk-1.0.0-x86_64-pc-windows-msvc.zip"
$expected = (Get-Content "$asset.sha256" | Select-Object -First 1).Split()[0].ToLowerInvariant()
$actual = (Get-FileHash $asset -Algorithm SHA256).Hash.ToLowerInvariant()
if ($expected -ne $actual) { throw "Checksum mismatch" }
```

## 3. Verify cosign keyless signature and certificate

Use the `.sig` and `.pem` files generated for each artifact.

Example:

```bash
ASSET="ntk-1.0.0-x86_64-unknown-linux-gnu.tar.gz"

cosign verify-blob \
  --certificate "${ASSET}.pem" \
  --signature "${ASSET}.sig" \
  --certificate-identity-regexp "^https://github.com/ThiagoGuislotti/nettoolskit-cli/.github/workflows/release.yml@refs/tags/v[0-9]+\\.[0-9]+\\.[0-9]+.*$" \
  --certificate-oidc-issuer "https://token.actions.githubusercontent.com" \
  "${ASSET}"
```

The verification must succeed and the certificate identity must match the project release workflow.

## 4. Minimum acceptance checklist

- Checksum verification succeeded.
- Signature verification succeeded for binary, checksum, and SBOM files.
- Certificate identity matches release workflow path.
- OIDC issuer is `https://token.actions.githubusercontent.com`.
- SBOM metadata contains valid format headers (`bomFormat=CycloneDX`, `spdxVersion=SPDX-*`).

## Troubleshooting

- Missing `.sig` / `.pem`: release signing step may have failed.
- Checksum mismatch: artifact corruption or tampering.
- Cosign identity mismatch: artifact was not signed by expected workflow/tag context.