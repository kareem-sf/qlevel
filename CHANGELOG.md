# Changelog

All notable changes to `qlevel` are documented in this file.

## [0.0.1] - 2026-11-25

### Features

- **(domain)** Establish fixed-point `DecimalValue` domain contracts for RICS NRM 2 BOQ items, VAT policies, merged alignment groups, and 7 technical trap categories
- **(parser)** Add pure-Rust Excel parser with XML `<row hidden="1">` detection, PDF digital/scanned page inspector, clipboard TSV/HTML triage, and OS file-lock detection
- **(ai)** Implement direct OpenAI (`Sign in with ChatGPT` OAuth PKCE + API Key), Google Gemini (`v1beta`), and Anthropic Claude (`2026-10-21`) clients with live multimodal `/models` filtering
- **(engine)** Implement SQLite WAL `BLAKE3` checkpointing, `Auto` model/effort router, synthetic BOQ alignment, `InteractiveReview` and `YellowHighlightAuto` merged-item modes, and zero-double-counting commercial equalization
- **(workbook)** Generate live Excel 2016+ `.xlsx` comparison sheets in `SingleMasterSheet` and `DualTab` layouts with `RTL`/`LTR` support and classic `SUM`/`MIN`/`IF` formulas
- **(bridge)** Add envelope-encrypted OS vault (`32-byte` keyring DEK + `AES-256-GCM`), portable self-updater with SHA-256 verification, and `ts-rs` contract generator
- **(desktop)** Build bilingual Arabic/English Progressive Disclosure workspace (`Basic` view + collapsible `Advanced Options` + `Ctrl+V` paste + `pdfjs-dist` scanned page rasterizer)
- **(ci)** Add cross-OS GitHub Actions matrix (`windows-2022`, `macos-15`, `ubuntu-22.04`) with `release-plz` versioning and Sigstore `actions/attest@v4` provenance
