# Technical Bid Evaluation (`TBE`) & Hidden Trap Taxonomy

Subcontractor bids routinely look lowest on the BOQ summary page while shifting major cost and schedule risk back to the Main Contractor through exclusions, brand substitutions, and commercial caveats. `qlevel` audits every quotation across **7 built-in engineering trap categories** plus **user-defined Custom Rules**.

## 1. The 7 Canonical Trap Categories (`TrapCategory`)

| Category | Typical Construction Triggers | Leveling Action |
| --- | --- | --- |
| `ScopeExclusion` | Excluding scaffolding above 3m, civil builders work (chasing, coring, grouting), craneage, offloading, power/water on site, or waste disposal | Flags severity (`High`/`Critical`), quotes exact vendor clause + page, and attaches optional `PlugBasis` |
| `TechnicalDeviation` | Reducing material gauge/thickness, changing cable insulation (`XLPE` to `PVC`), lowering fire rating or IP rating, omitting accessories | Records BOQ spec vs. Vendor proposed spec in TBE matrix |
| `AvlRegulatory` | Substituting unlisted brands outside the project Approved Vendor List (`AVL`), missing Civil Defense / UL / FM / SASO / EOS certifications | Marks AVL non-compliance for consultant sign-off |
| `QtyArithmetic` | Altering Master BOQ quantities, changing unit of measure (`m2` vs `m`, `LS` vs `No`), hidden Excel rows (`<row hidden="1">`), `Qty * Rate != Total` | Normalizes onto Master BOQ quantity and isolates arithmetic delta |
| `LogisticsIncoterms` | Quoting `EXW` or `FOB` instead of `DDP Project Site` ([ICC Incoterms 2020](https://iccwbo.org/business-solutions/incoterms-rules/incoterms-2020/)), long lead times exceeding programme float | Flags customs/freight exposure and lead-time risk |
| `CommercialRisk` | Uncapped price-validity windows (`7 days`), advance payment > project cap without bank guarantee, omitting retention or liquidated damages (`LD`) | Highlights commercial negotiation points in Executive Summary |
| `CloseoutDocs` | Excluding third-party Testing & Commissioning (`T&C`), shop drawings, BIM LOD 400 models, As-Built drawings, O&M manuals, or spare parts | Flags handover risk and applies T&C/closeout plug when configured |

## 2. User-Defined Custom Rules (`custom_rules`)

In `Advanced Options`, the engineer can enter plain-Arabic or plain-English project rules or load a reusable `.txt`/`.md` checklist, for example:

- `تأكد أن السعر يشمل السقالات بارتفاع البهو 12 متر واختبارات الدفاع المدني`
- `Verify all chillers are AHRI certified, refrigerant R-134a/R-513A, and include 5-year compressor warranty`

Custom rules are injected both into structured AI clause extraction and into deterministic keyword/clause cross-checks in `crates/qlevel-engine/src/technical_eval.rs`.
