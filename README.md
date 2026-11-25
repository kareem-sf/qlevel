# qlevel - Commercial & Technical Bid Leveling Workbench

**Open-source desktop workbench for Quantity Surveyors, Cost Managers, and Procurement Engineers to equalize subcontractor quotations (`CBE`) and catch hidden technical exclusions (`TBE`) into a live-formula Excel comparison sheet.**

- **License**: Apache-2.0 (`LICENSE`, `NOTICE`, `TRADEMARKS.md`)
- **Stack**: Tauri v2 + Pure Rust (`rust_decimal` fixed-point math) + React 19 / TypeScript / Tailwind CSS v4
- **Platforms**: Windows 10/11 (`.exe` portable + NSIS user-space installer), macOS 13+ (`Universal` / Apple Silicon & Intel), Linux (`Ubuntu 22.04+` AppImage / `.deb`)
- **Zero Server Custody**: Runs 100% locally on your machine using your own direct OpenAI (`Sign in with ChatGPT` or API Key), Google Gemini, or Anthropic Claude account.

## Why qlevel?

Subcontractors rarely price your Master BOQ cleanly:

1. **Arithmetic & Quantity Tampering**: Vendors quietly alter BOQ quantities, miscalculate `Qty * Rate`, or hide excluded rows inside `<row hidden="1">` Excel sheets.
2. **Merged Lump-Sum Rows**: Vendors combine 3 BOQ items into one lump sum (`Included above`), breaking row-by-row Excel formulas.
3. **Mixed VAT & Provisional Sums**: One bidder includes 15% KSA VAT (or 14% Egypt VAT) inside unit rates while another quotes net, or discounts fixed Provisional Sums contrary to RICS NRM 2 equalization rules.
4. **Last-Page Technical Exclusions**: A bidder looks 12% cheaper on page 1, then excludes scaffolding, civil builders work, third-party testing, or Approved Vendor List (`AVL`) brands in fine print on the last PDF page.

`qlevel` ingests your Master BOQ (or synthesizes a unified baseline straight from raw vendor quotes when no Master BOQ exists), aligns every vendor item without dropping rows, flags all 7 commercial/technical trap categories plus your own custom rules, and exports an auditable `.xlsx` comparison workbook with live `SUM` / `MIN` / `IF` formulas compatible with Excel 2016+.

## Key Capabilities

- **Zero-Trust Fixed-Point Math**: AI models extract text, unit mappings, and exclusion clauses only. Every multiplication, VAT strip, discount proration, and missing-scope plug runs in deterministic Rust `Decimal` arithmetic (`#![deny(clippy::float_arithmetic)]`).
- **Progressive Disclosure UX**:
  - **Basic Mode (Default)**: Built for everyday procurement engineers. Drag-and-drop Excel/PDF files or press `Ctrl+V` to paste tables/WhatsApp quote screenshots, review detected roles (`Master BOQ` / `Vendor Quote` / `Specs & AVL`), and click **Compare & Export Excel** using automatic model/effort routing.
  - **Advanced Options (Collapsible)**: Full control for power users over live Multimodal models, reasoning effort (`Auto` / `Low` / `Medium` / `High`), Excel layout (`Single Master Sheet` vs `Dual-Tab CBE + TBE`), merged-item handling (`Interactive Quick Card` vs `Yellow Highlight Auto-Export`), missing-scope plug basis, currency (`SAR`, `EGP`, `AED`, `QAR`, `KWD`, `BHD`, `OMR`, `USD`, `EUR`), VAT %, `RTL`/`LTR` sheet direction, and custom engineering audit rules.
- **Direct Cloud Multimodal Providers (Zero Hardcoded Model Lists)**:
  - **OpenAI**: One-click `Sign in with ChatGPT` (dynamic local OAuth PKCE) or `OPENAI_API_KEY`.
  - **Google Gemini**: `GEMINI_API_KEY` (with built-in unpaid free-tier confidentiality check).
  - **Anthropic Claude**: `ANTHROPIC_API_KEY`.
  - Live `/models` discovery strictly filters out `TTS`, `STT`, `whisper`, `audio`, `embedding`, `imagen`, and retired snapshots so only current Multimodal vision/PDF models appear.
- **Enterprise-Friendly Local Security**:
  - Envelope encryption (`32-byte` OS Keyring DEK + `AES-256-GCM` local vault) bypasses the Windows Credential Manager 2560-byte blob limit.
  - Native OS TLS root certificates (`rustls-tls-native-roots`) work seamlessly behind corporate SSL inspection proxies (`Zscaler`, `FortiGate`).
  - SQLite WAL batch checkpoints keyed by `BLAKE3` resume interrupted 150-page tenders instantaneously without re-billing completed batches.

## Repository Architecture

```text
qlevel/
  apps/
    desktop/              # React 19 + TypeScript + Tailwind v4 UI + pdf.js rasterizer
      src-tauri/          # Thin Tauri v2 native shell, strict CSP, capabilities
  packages/
    bindings/             # Auto-synchronized Rust <-> TypeScript IPC contracts & client
    design-system/        # Logical RTL/LTR accessible engineering UI components
  crates/
    qlevel-domain/        # Pure fixed-point Decimal domain models & invariants
    qlevel-parser/        # Excel (calamine + XML hidden-row guard), PDF, Clipboard, Lock probe
    qlevel-ai/            # OpenAI SIWC/Responses, Gemini v1beta, Claude 2026-10-21, Model filter
    qlevel-engine/        # SQLite WAL checkpoints, Auto router, Synthetic BOQ, CBE/TBE engine
    qlevel-workbook/      # Excel 2016+ live-formula .xlsx exporter (SingleSheet & DualTab)
    qlevel-bridge/        # Envelope OS vault, portable self-updater, Tauri commands, ts-rs CLI
  docs/                   # Architecture diagrams, ADRs, mathematical & Excel specs, user guides
  tests/fixtures/         # Golden end-to-end KSA MEP (15% VAT) & Egypt Fit-Out (14% VAT) tenders
  .github/workflows/      # CI gates, release-plz SemVer PRs, cross-OS builds + Sigstore attest
```

## Quick Start (User-Space Development, No Admin Rights Needed)

Prerequisites: [Rust 1.85+](https://rustup.rs/), [Node.js 20+](https://nodejs.org/), and [pnpm 9+](https://pnpm.io/).

```bash
git clone https://github.com/sfkareem/qlevel.git
cd qlevel

# Install frontend workspace packages
pnpm install

# Run full Rust unit & golden integration tests
cargo test --workspace

# Launch the desktop application in development mode
pnpm dev
```

To build a portable user-space executable on Windows, macOS, or Linux:

```bash
pnpm tauri:build
```

## Documentation Index

- [System Architecture & Data Flow](docs/architecture/OVERVIEW.md)
- [Commercial Leveling Math Specification (RICS NRM 2 & GCC/Egypt VAT)](docs/specs/COMMERCIAL_MATH_SPEC.md)
- [Technical Trap & Scope-Exclusion Taxonomy](docs/specs/TECHNICAL_TRAPS_SPEC.md)
- [Excel 2016+ Workbook Formula & Layout Specification](docs/specs/EXCEL_WORKBOOK_SPEC.md)
- [Arabic & English Engineer Quickstart Guide](docs/guides/QUICKSTART_AR_EN.md)
- [Architecture Decision Records (`ADR-001` to `ADR-006`)](docs/adr/)

## License & Attribution

Licensed under the [Apache License, Version 2.0](LICENSE). Redistribution in source or binary form must preserve [NOTICE](NOTICE) and comply with [TRADEMARKS.md](TRADEMARKS.md).
