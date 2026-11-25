# qlevel System Architecture & Data Flow

`qlevel` is a standalone open-source desktop micro-application (`com.kareemsafwat.qlevel`) built as a `Cargo Workspace + pnpm Workspace` Modular Monolith on [Tauri v2](https://v2.tauri.app/).

## 1. Crate & Package Dependency Graph

```text
apps/desktop (React 19 + Tailwind v4 + pdfjs-dist)
   |
   +--> packages/design-system (Logical RTL/LTR UI Primitives)
   +--> packages/bindings (Strict TypeScript IPC Contracts + Client)
           |
           v  (Tauri v2 IPC Boundary - DecimalValue serialized as exact string)
apps/desktop/src-tauri (Thin Binary Shell, CSP, Capabilities)
   |
   +--> crates/qlevel-bridge (Commands, Session State, Envelope Vault, Self-Updater)
           |
           +--> crates/qlevel-engine (SQLite WAL Checkpoints, Auto Router, CBE/TBE Math)
           |       |
           |       +--> crates/qlevel-parser (calamine + quick-xml, lopdf, Clipboard)
           |       +--> crates/qlevel-ai (OpenAI SIWC/Responses, Gemini v1beta, Claude 2026-10-21)
           |       +--> crates/qlevel-domain (Fixed-Point DecimalValue & Pure Invariants)
           |
           +--> crates/qlevel-workbook (rust_xlsxwriter Excel 2016+ Live Formulas)
                   |
                   +--> crates/qlevel-domain
```

## 2. End-to-End Leveling Pipeline

1. **Triage & Ingestion (`qlevel-parser`)**:
   - Excel workbooks (`.xlsx`, `.xlsm`, `.xls`, `.xlsb`) are parsed via [calamine](https://docs.rs/calamine/latest/calamine/) paired with a pre-flight `quick-xml` scan of `xl/worksheets/sheet*.xml` to detect `<row hidden="1">` rows that `calamine` does not mark natively ([calamine Issue #237](https://github.com/tafia/calamine/issues/237)).
   - PDF files are inspected via `lopdf` to classify pages into digital text pages vs. scanned/low-text pages (`< 40` non-whitespace characters). Scanned pages emit a `qlevel://pdf-raster-request` event to the WebView `pdfjs-dist` `<canvas>` worker and return high-DPI PNG/WebP byte buffers without requiring external C++ DLLs.
   - Clipboard tables (`Ctrl+V` from Excel, Outlook, or WhatsApp screenshots) are normalized directly into `StagedInputDocument` payloads.
2. **Dynamic Model Routing (`qlevel-ai` + `auto_router`)**:
   - Available models are queried live from each connected provider's `/models` endpoint and filtered in Rust (`ModelFilter`) to expose flagship Multimodal (text + vision/PDF) models only.
   - In `Auto` mode, clean tabular batches route to fast Multimodal models at `Low` effort, while scanned PDFs, ambiguous merged rows, and exclusion fine print escalate automatically to flagship reasoning models at `High` effort.
3. **Resumable Batch Extraction (`checkpoint_db`)**:
   - Every extraction batch is fingerprinted with a canonical [BLAKE3](https://docs.rs/blake3/latest/blake3/) digest over `(schema_version, provider, model_id, effort, document_blake3, batch_index, custom_rules_hash)` and persisted in a local [rusqlite](https://docs.rs/rusqlite/latest/rusqlite/) `WAL` checkpoint store so interrupted runs resume immediately.
4. **Baseline Synthesis & Merged-Item Resolution (`qlevel-engine`)**:
   - If the engineer provides a Master BOQ, vendor quotes align against its `BoqItem` rows. If only vendor quotes are provided, `SyntheticBoqBuilder` elects the cleanest quote and performs constrained bipartite matching so `orphan_vendor_rows == 0`.
   - Many-to-one or one-to-many merged vendor rows form explicit `AlignmentGroup` records handled according to the engineer's selected mode (`InteractiveReview` quick card or `YellowHighlightAuto` anchor-and-zero-followers rule).
5. **Deterministic Commercial & Technical Equalization (`commercial_calc` + `technical_eval`)**:
   - All rates, quantities, VAT adjustments, Provisional Sum locks ([RICS NRM 2](https://www.rics.org/content/dam/ricsglobal/documents/standards/NRM-2_Oct2022_Update.pdf)), lump-sum discount allocations, and missing-scope plugs execute strictly in 128-bit fixed-point `rust_decimal::Decimal`.
6. **Live-Formula Workbook Generation (`qlevel-workbook`)**:
   - Exports either `SingleMasterSheet` or `DualTab` `.xlsx` files via [rust_xlsxwriter](https://docs.rs/rust_xlsxwriter) using classic Excel 2016+ formulas (`SUM`, `MIN`, `IF`, `COUNTIF`) and preflight file-lock detection.
