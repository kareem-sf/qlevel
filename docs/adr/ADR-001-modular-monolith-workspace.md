# ADR-001: Standalone Modular Monolith Repository Topology

- **Status**: Accepted
- **Date**: 2026-11-25

## Context

`qlevel` is the first open-source desktop micro-application in the QSMind quantity surveying ecosystem. Mixing unrelated products inside a single giant repository slows CI matrices and couples SemVer release tags, while splitting internal Rust layers across multiple repositories breaks atomic cross-crate refactoring.

## Decision

Host `qlevel` in its own dedicated GitHub repository (`https://github.com/sfkareem/qlevel`) organized internally as a `Cargo Workspace + pnpm Workspace` Modular Monolith inspired by mature Tauri v2 workspaces ([Spacedrive Cargo.toml](https://github.com/spacedriveapp/spacedrive/blob/main/Cargo.toml)):

- `crates/qlevel-domain`, `qlevel-parser`, `qlevel-ai`, `qlevel-engine`, `qlevel-workbook`, and `qlevel-bridge` compile and test cleanly with `cargo test` without initializing a GUI window.
- `apps/desktop/src-tauri` acts strictly as a thin native binary shell (`default-members` excludes GUI window startup during headless Rust checks).
- `packages/bindings` and `packages/design-system` isolate IPC contracts and RTL/LTR components from application state in `apps/desktop`.

## Consequences

- Clean crate dependency DAG with zero circular dependencies.
- Fast headless `cargo test --workspace` execution in CI and local terminals.
- Unified Semantic Versioning across all workspace crates via `release-plz` (`version_group = "qlevel"`).
