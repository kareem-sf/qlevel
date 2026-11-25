# AGENTS.md - Coding Agent Architecture & Guardrails for qlevel

This file defines mandatory architectural boundaries and verification gates for AI coding agents working inside `qlevel`.

## 1. Repository Topology (Modular Monolith)

`qlevel` is a standalone product repository structured as a `Cargo Workspace + pnpm Workspace`:

- `crates/qlevel-domain`: Pure domain types, `DecimalValue`, RICS/FIDIC line-item taxonomies, trap models, session configs. Zero I/O, zero network, zero OS calls.
- `crates/qlevel-parser`: Pure-Rust Excel (`calamine` + `quick-xml` hidden-row scanner), PDF text & page-count inspector (`lopdf`), clipboard TSV/HTML parser, bilingual AR/EN header detector, and Excel file-lock probe.
- `crates/qlevel-ai`: Direct cloud clients for OpenAI (`Sign in with ChatGPT` dynamic OAuth PKCE + `/v1/responses` streaming + API Key), Google Gemini (`v1beta` structured JSON + thinking levels), and Anthropic Claude (`2026-10-21` adaptive thinking without deprecated `temperature`). Includes live `/models` multimodal filtering.
- `crates/qlevel-engine`: SQLite WAL checkpoint store (`BLAKE3` batch keys), `Auto` model/effort router, synthetic BOQ builder (`orphan_vendor_rows == 0`), merged-item resolver (`InteractiveReview` & `YellowHighlightAuto`), exact `Decimal` commercial calculator, and 7-category + custom-rule TBE evaluator.
- `crates/qlevel-workbook`: Live `.xlsx` generator (`rust_xlsxwriter`) compatible with Excel 2016+ (zero `_xlfn.` dynamic array functions, `<=31` char sheet names, `SingleMasterSheet` and `DualTab` layouts, `RTL`/`LTR` direction).
- `crates/qlevel-bridge`: Tauri v2 IPC DTOs, `ts-rs` exporter (`export_bindings`), envelope-encrypted OS vault (`32-byte` keyring DEK + `AES-256-GCM`), and portable executable self-updater (`self_update` + `self-replace`).
- `apps/desktop/src-tauri`: Thin binary shell (`qlevel-desktop`) wiring Tauri capabilities, strict CSP, and Linux WebKitGTK DMA-BUF mitigation.
- `packages/bindings`: Synchronization target for Rust-to-TypeScript contracts (`DecimalValue = string`) and typed Tauri IPC client.
- `packages/design-system`: Accessible bilingual React 19 + Tailwind CSS v4 primitives using logical layout properties.
- `apps/desktop`: Progressive Disclosure UI (`Basic` default workflow + collapsible `Advanced Options` panel + in-memory `pdfjs-dist` scanned page rasterizer).

## 2. Hard Prohibitions

- **Never** introduce `f32` or `f64` into commercial rates, quantities, taxes, discounts, or plug formulas.
- **Never** add `.unwrap()`, `.expect()`, `todo!()`, `unimplemented!()`, or `any` types.
- **Never** add OpenRouter or local LLM backends (Ollama, LM Studio, llama.cpp).
- **Never** hardcode static provider model IDs in UI dropdowns; only consume live filtered `/models` output.
- **Never** store full OAuth token JSON directly in Windows Credential Manager (`2560-byte` `CREDENTIALW` cap); always go through `EnvelopeVault`.
- **Never** emit `_xlfn.XLOOKUP`, `_xlfn.MINIFS`, `_xlfn.LET`, or sheet names longer than 31 UTF-16 code units in `qlevel-workbook`.

## 3. Mandatory Verification Gate Before Committing

Run all four gates before opening or updating any pull request:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p qlevel-bridge --bin export_bindings -- --check
pnpm -r typecheck
```
