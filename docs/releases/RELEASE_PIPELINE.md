# Automated Release, Code Provenance & User-Space Update Pipeline

This guide documents how `qlevel` ships verifiable cross-platform releases from `0.0.1` onward without requiring manual version edits or hardware EV code-signing dongles.

## 1. Automated SemVer & Release PRs (`release-plz`)

1. Every pull request merged into `main` uses Conventional Commits (`feat:`, `fix:`, `perf:`, `security:`).
2. `.github/workflows/release-pr.yml` runs [release-plz](https://release-plz.dev/docs/config) using `release-plz.toml` (`git_only = true`, `publish = false`, `version_group = "qlevel"`).
3. `release-plz` maintains an automated Release Pull Request that updates `[workspace.package].version` in `Cargo.toml` (`apps/desktop/src-tauri/tauri.conf.json` inherits version from `Cargo.toml` automatically) and updates `CHANGELOG.md` via `cliff.toml`.
4. When a maintainer merges the Release PR, `release-plz` tags `vX.Y.Z` (using `RELEASE_PLZ_TOKEN`), triggering `.github/workflows/release.yml`.

## 2. Cross-OS Build Matrix (`release.yml`)

| Target OS | GitHub Runner | Artifact Outputs | Compatibility Notes |
| --- | --- | --- | --- |
| Windows x64 | `windows-2022` | `qlevel-portable-windows-x64.exe` + `currentUser` NSIS setup | Runs on non-admin corporate Windows 10/11 accounts (`%LOCALAPPDATA%`) |
| macOS Apple Silicon | `macos-15` | `.dmg` + `.app.tar.gz` (`aarch64-apple-darwin`) | Ad-hoc signed (`"-"`) to avoid Gatekeeper "damaged app" errors on unsigned OSS builds |
| macOS Intel | `macos-15` | `.dmg` + `.app.tar.gz` (`x86_64-apple-darwin`) | Ad-hoc signed (`"-"`) |
| Linux x64 | `ubuntu-22.04` | `.AppImage` + `.deb` | Compiled against `glibc 2.35` (`webkit2gtk-4.1`) for broad Ubuntu/Debian/Fedora compatibility ([GitHub Runner Images](https://github.com/actions/runner-images)) |

## 3. Checksums & Cryptographic Provenance (`Sigstore`)

- `release.yml` computes `SHA256SUMS` across all release installers and portable binaries.
- On public GitHub repositories (`!github.event.repository.private`), `actions/attest@v4` generates cryptographic Sigstore provenance attestations ([GitHub Attestations](https://docs.github.com/actions/security-for-github-actions/using-artifact-attestations/using-artifact-attestations-to-establish-provenance-for-builds)) verifiable by enterprise IT teams via:
  ```bash
  gh attestation verify qlevel-portable-windows-x64.exe --owner kareem-sf
  ```
- On Windows portable builds, `qlevel-bridge::updater` verifies `SHA-256` digests and checks folder writability before atomically swapping the running `.exe` via `self-replace`.
