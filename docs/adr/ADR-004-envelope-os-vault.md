# ADR-004: Envelope-Encrypted Local Vault for OS Credential Stores

- **Status**: Accepted
- **Date**: 2026-11-25

## Context

Windows Credential Manager (`CREDENTIALW`) enforces a hard `2560-byte` ceiling (`CRED_MAX_CREDENTIAL_BLOB_SIZE`) on credential secrets ([Microsoft CREDENTIALW](https://learn.microsoft.com/en-us/windows/win32/api/wincred/ns-wincred-credentialw)). Storing full OAuth PKCE bundles (`access_token`, `id_token`, `refresh_token`, and dynamically registered `oaiapp_...` `client_id`) directly inside `keyring` fails on Windows with `ERROR_BAD_ARGUMENTS` once JWT payloads exceed 2.5 KB ([Codex Issue #10353](https://github.com/openai/codex/issues/10353)).

## Decision

Implement Envelope Encryption (`crates/qlevel-bridge/src/vault.rs`):

1. Generate a cryptographically random `32-byte` Data Encryption Key (`DEK`) and store **only** those 32 bytes in the OS keyring via [keyring](https://docs.rs/keyring/latest/keyring/) (`Entry::set_secret` / `Entry::get_secret`).
2. Encrypt all API keys and OAuth token bundles in a local container (`vault.enc`) using [aes-gcm](https://docs.rs/aes-gcm/latest/aes_gcm/) (`AES-256-GCM`) with a fresh 96-bit CSPRNG nonce on every write, magic header `QLVL_V1\0`, and AAD `"qlevel.auth.v1"`.
3. Scrub all in-memory key and plaintext buffers on drop via [zeroize](https://docs.rs/zeroize/latest/zeroize/), and guard OAuth token refresh rotations with an async `Mutex`.

## Consequences

- Guaranteed compatibility with Windows Credential Manager, macOS Keychain, and Linux Secret Service regardless of OAuth JWT size.
- Zero plaintext secrets written to disk, SQLite checkpoints, logs, or IPC error messages.
