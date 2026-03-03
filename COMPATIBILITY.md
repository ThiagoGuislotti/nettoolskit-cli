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

### Support window policy

- Active support window: `12` months from GA (`General Availability`) date.
- Maintenance support window: `6` months after active support ends.
- EOL (`End of Life`) starts on the day after maintenance support ends.
- When a new stable minor is released, the previous stable minor transitions to maintenance support.

### Security response

- Critical/High dependency vulnerabilities block release promotion.
- Security fixes are prioritized and patched in actively supported releases.

### Breaking changes and deprecation

- Breaking changes follow semantic versioning (major release only).
- Deprecated behavior should be announced in `CHANGELOG.md` before removal.

### Scope

- Support commitments apply to official GitHub release artifacts only.
- Locally compiled binaries and custom environments are best effort.

## Support Lifecycle and EOL

Reference date for status labels in this table: **March 1, 2026**.

| Minor | GA date | Active support until | Maintenance support until | EOL date | Status |
|---|---|---|---|---|---|
| `1.0` | January 4, 2025 | January 4, 2026 | July 4, 2026 | July 5, 2026 | Maintenance |
| `<1.0` legacy line | N/A | N/A | N/A | March 1, 2026 | Unsupported |

### Lifecycle update cadence

- The EOL table is updated on every stable minor release.
- Dates are maintained in ISO-aware human format (`Month Day, Year`) for operational clarity.
- Release pipeline validates EOL table semantics (date ordering, `EOL = maintenance + 1 day`, and status coherence).