# Security Policy

## Supported Versions

| Version | Supported |
| --- | --- |
| `0.1.x` | Yes |
| `0.0.x` (Pre-release) | Best effort |

## Security Architecture Guarantees

1. **Zero Server Custody**: `qlevel` runs strictly on your local desktop workstation. Tender BOQs, vendor quotations, and commercial sheets are never uploaded to QSMind servers.
2. **Envelope Credential Encryption**: Provider API keys and `OpenAI Sign in with ChatGPT` OAuth tokens are encrypted locally with `AES-256-GCM`. Only a random 32-byte Data Encryption Key (`DEK`) is stored in your operating system credential store (`Windows Credential Manager`, `macOS Keychain`, or `Linux Secret Service`), with automatic memory scrubbing via `zeroize`.
3. **WebView Network Isolation**: All external HTTPS requests execute inside the native Rust TLS client (`reqwest` + OS native root store). The embedded WebView enforces a strict Content Security Policy (`connect-src 'self' ipc: http://ipc.localhost`) that blocks outbound script connections.
4. **Gemini Free-Tier Privacy Guard**: Google's unpaid Gemini API tier uses submitted prompts and documents for model improvement. `qlevel` detects unpaid tier keys or warns users explicitly before transmitting confidential tender sheets.
5. **Verifiable Release Provenance**: Official release binaries include `SHA256SUMS` and cryptographic GitHub Artifact Attestations (`Sigstore` via `actions/attest@v4`).

## Reporting a Vulnerability

Please do **not** open public GitHub issues for suspected security vulnerabilities.

Report vulnerabilities privately via GitHub Security Advisories on `https://github.com/kareem-sf/qlevel/security/advisories/new` or email `kareem@qsmind.app` with:

- Affected `qlevel` version and operating system
- Minimal reproduction steps (using non-confidential synthetic BOQ data)
- Impact assessment

We acknowledge vulnerability reports within 48 hours and aim to issue patched releases within 7 days for high-severity findings.
