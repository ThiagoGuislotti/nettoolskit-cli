# Security Policy

## Supported Versions

Security fixes are applied to the latest supported release line documented in [COMPATIBILITY.md](COMPATIBILITY.md).

## Reporting a Vulnerability

Do not open a public GitHub issue for suspected vulnerabilities.

Use private disclosure instead:

- Email: `security@nettoolskit.local`
- Subject: `NetToolsKit security report`

Include:

- affected version or commit
- reproduction steps
- impact assessment
- logs or proof-of-concept if available

## Response Targets

- Initial acknowledgement: within 5 business days
- Triage and severity classification: within 10 business days
- Fix or mitigation plan: as soon as a validated remediation path exists

## Disclosure Process

1. Report is triaged privately.
2. Maintainer validates impact and affected versions.
3. Fix is prepared with regression tests and release notes.
4. Public disclosure happens after a fix or mitigation is available.

## Operational Notes

- Dependency vulnerability gates run in CI and local release validation.
- Service mode should remain loopback-by-default unless explicit authentication is configured.
- ChatOps ingress should remain protected by signature/replay controls when exposed beyond localhost.