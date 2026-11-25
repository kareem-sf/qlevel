# ADR-005: Dual Excel Layouts & Classic Excel 2016+ Formula Compatibility

- **Status**: Accepted
- **Date**: 2026-11-25

## Context

Construction contracting firms across Saudi Arabia, Egypt, and the UAE frequently run perpetual-license Microsoft Excel 2016 or Excel 2019 installations alongside Microsoft 365. Modern Excel dynamic-array functions (`XLOOKUP`, `MINIFS`, `MAXIFS`, `LET`, `TEXTJOIN`) serialize with `_xlfn.` prefixes in OOXML ([MS-XLSX Functions](https://learn.microsoft.com/en-us/openspecs/office_standards/ms-xlsx/5d1b6d44-6fc1-4ecd-8fef-0b27406cc2bf)) and render `#NAME?` errors when opened on Excel 2016. Additionally, different procurement teams prefer either a single consolidated comparison sheet or separate Commercial (`CBE`) and Technical (`TBE`) tabs.

## Decision

1. Support **both** workbook layouts in `qlevel-workbook` (selectable in `Advanced Options`, defaulting to `SingleMasterSheet`):
   - `SingleMasterSheet`: One unified worksheet containing Executive Summary Totals at top, the line-by-line BOQ vs. Vendors matrix in the middle, and the Technical Exclusions & Risk Matrix directly below.
   - `DualTab`: Separate `01_Commercial_CBE` and `02_Technical_TBE` worksheets plus `03_Audit_Notes`.
2. Emit strictly classic Excel 2007/2016+ formulas via [rust_xlsxwriter](https://docs.rs/rust_xlsxwriter): `SUM`, `MIN`, `MAX`, `AVERAGE`, `IF`, `AND`, `OR`, `COUNTIF`, and `ROUND`. Never emit `_xlfn.` formulas.
3. Enforce the OOXML `31-character` worksheet tab name limit and sanitize forbidden characters (`\ / ? * : [ ]`).

## Consequences

- Every exported `.xlsx` opens cleanly without `#NAME?` or repair warnings in Excel 2016, Excel 2019, Excel 2021, Microsoft 365, and LibreOffice Calc.
