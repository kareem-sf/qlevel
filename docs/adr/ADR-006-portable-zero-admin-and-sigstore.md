# ADR-006: Zero-Admin Portable Builds, Automated Release-Plz & Sigstore Provenance

- **Status**: Accepted
- **Date**: 2026-11-25

## Context

Corporate procurement engineers often run Windows 11 laptops without local Administrator privileges (`UAC` blocks MSI/system-wide installers), while maintainers of open-source desktop tools want reproducible cross-platform releases without manual tag editing or expensive EV hardware tokens.

## Decision

1. Package Windows builds both as a standalone portable `.exe` (updated safely in user-space via [self_update](https://docs.rs/self_update) + [self-replace](https://docs.rs/self-replace) after verifying directory writability and SHA-256 checksums) and a `currentUser` NSIS installer (`%LOCALAPPDATA%`).
2. Build Linux binaries on `ubuntu-22.04` (`glibc 2.35`) with `libwebkit2gtk-4.1-dev` and set `WEBKIT_DISABLE_DMABUF_RENDERER=1` at startup when unset to prevent blank GPU windows on Wayland/NVIDIA systems.
3. Automate SemVer bumps and `CHANGELOG.md` generation with [release-plz](https://release-plz.dev/docs/config) (`git_only = true`, `publish = false`) and generate cryptographic build provenance on public releases via `actions/attest@v4` ([GitHub Attestations](https://docs.github.com/actions/security-for-github-actions/using-artifact-attestations/using-artifact-attestations-to-establish-provenance-for-builds)) alongside `SHA256SUMS`.

## Consequences

- Works out-of-the-box on non-admin corporate Windows 11 accounts (`K.Safwat-EG` profile class).
- Every public binary release carries verifiable cryptographic provenance from GitHub Actions runners with zero manual certificate maintenance.
