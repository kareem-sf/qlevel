# Excel 2016+ Workbook Layout & Formula Specification

`crates/qlevel-workbook` generates native `.xlsx` workbooks via [rust_xlsxwriter](https://docs.rs/rust_xlsxwriter) designed for immediate sign-off by Commercial Directors and Tender Committees.

## 1. Layout Modes (Selectable in `Advanced Options`)

### Mode A: `SingleMasterSheet` (Default)
Places the entire commercial and technical decision story on one scrollable, print-ready worksheet (`Bid_Leveling_Sheet`):

1. **Project Header & Metadata Block**: Tender title, currency, statutory VAT %, plug basis, date, and True Low Bidder callout.
2. **Executive Commercial Summary Block**: Side-by-side vendor totals (`As-Submitted Total`, `Arithmetic & Qty Delta`, `Discount Adjustment`, `Provisional Sums Equalized`, `Missing-Scope Plugs`, `Leveled Net Total`, `VAT`, `Leveled Gross Total`, `Variance vs. Lowest %`, and `Rank`).
3. **Line-by-Line BOQ Equalization Matrix**: Master BOQ columns (`Code`, `Description`, `Category`, `Unit`, `Master Qty`, `PTE Rate`, `PTE Total`) followed by grouped vendor super-headers (`Net Unit Rate`, `Submitted Amount`, `Equalized Plug`, `Leveled Net Total`, `Status / Audit Note`) and horizontal benchmark columns (`Lowest Net Rate`, `Average Net Rate`).
4. **Technical Exclusions & Risk Matrix (`TBE`)**: Directly beneath the BOQ totals, listing all detected commercial terms, AVL compliance checks, and itemized `DetectedTrap` rows with exact source page/clause citations.

### Mode B: `DualTab`
Splits the workbook into clean specialized worksheets (all tab names strictly `<= 31` UTF-16 characters):

- `01_Commercial_CBE`: Executive Commercial Summary + Line-by-Line BOQ Equalization Matrix.
- `02_Technical_TBE`: Commercial Terms Comparison + Technical Exclusions & AVL Compliance Matrix.
- `03_Audit_Notes`: Equalization basis notes, merged-item groups, and checkpoint provenance.

## 2. Classic Excel 2016+ Formula Discipline

To prevent `#NAME?` errors on perpetual Excel 2016 and Excel 2019 installations ([MS-XLSX Functions](https://learn.microsoft.com/en-us/openspecs/office_standards/ms-xlsx/5d1b6d44-6fc1-4ecd-8fef-0b27406cc2bf)):

- **Allowed Formulas**: `SUM`, `MIN`, `MAX`, `AVERAGE`, `IF`, `AND`, `OR`, `COUNTIF`, `ROUND`.
- **Forbidden Formulas**: `_xlfn.XLOOKUP`, `_xlfn.MINIFS`, `_xlfn.MAXIFS`, `_xlfn.IFS`, `_xlfn.LET`, `_xlfn.LAMBDA`, `_xlfn.FILTER`.
- **Line Leveling Formula**: Each vendor's `Leveled Total` cell is a live Excel formula (`=ROUND($E12*H12, 2)+J12` where `$E12` is Master Qty, `H12` is Net Unit Rate, and `J12` is Scope Plug; for `YellowHighlightAuto` anchor lump-sum rows, `=I12+J12` with `0` on follower rows).
- **Column Subtotals & Grand Totals**: Live `=SUM(...)` ranges so if the procurement engineer edits any unit rate or plug amount during a negotiation meeting, every vendor total, VAT line, variance %, and lowest-bidder indicator recalculates instantaneously in Excel.

## 3. Visual Highlighting & RTL/LTR Support

- **True Lowest Cell / Bidder**: Soft emerald fill (`#DCFCE7`) with dark green bold text (`#166534`).
- **Merged / Yellow Highlight Review Rows**: Warm amber fill (`#FEF3C7`) with dark amber text (`#92400E`) and explicit anchor reference note.
- **Unquoted / Scope Exclusion Risk**: Soft coral red fill (`#FEE2E2`) with dark crimson text (`#991B1B`).
- **Worksheet Direction**: `Worksheet::set_right_to_left(true)` is enabled automatically when `workbookDirection == "rtl"` or when `workbookDirection == "auto"` and Arabic script dominates the Master BOQ descriptions.
