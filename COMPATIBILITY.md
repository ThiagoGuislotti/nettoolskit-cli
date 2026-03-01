# Compatibility Matrix and Support Policy

This document defines platform compatibility and support commitments for `ntk`.

## Compatibility Matrix

### Tier 1 (fully supported)

| Target | Build in CI | Smoke test in release | Status |
|---|---|---|---|
| `x86_64-unknown-linux-gnu` | Yes | Yes | Supported |
| `x86_64-pc-windows-msvc` | Yes | Yes | Supported |
| `aarch64-apple-darwin` | Yes | Yes | Supported |

### Tier 2 (best effort)

| Target | Build in CI | Smoke test in release | Status |
|---|---|---|---|
| `x86_64-unknown-linux-musl` | Yes | No | Best effort |
| `aarch64-unknown-linux-gnu` | Yes | No | Best effort |
| `x86_64-apple-darwin` | Yes | No | Best effort |

## Runtime and Tooling Baselines

| Component | Baseline |
|---|---|
| Rust toolchain (MSRV) | `1.85.0` |
| Package manager | `cargo` (workspace) |
| Supported shells for completions | `bash`, `zsh`, `fish`, `powershell` |
| Terminal requirement | ANSI-compatible terminal; ASCII fallback supported |

## Support Policy

### Release channels

- Stable releases: tags matching `vMAJOR.MINOR.PATCH`.
- Prereleases: tags matching `vMAJOR.MINOR.PATCH-<label>`.

### Maintenance window

- Active support: latest stable minor release.
- Maintenance support: previous stable minor release (security and critical fixes only).
- Older releases: unsupported unless explicitly announced.

### Security response

- Critical/High dependency vulnerabilities block release promotion.
- Security fixes are prioritized and patched in actively supported releases.

### Breaking changes and deprecation

- Breaking changes follow semantic versioning (major release only).
- Deprecated behavior should be announced in `CHANGELOG.md` before removal.

### Scope

- Support commitments apply to official GitHub release artifacts only.
- Locally compiled binaries and custom environments are best effort.
