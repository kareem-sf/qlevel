# Commercial Bid Leveling (`CBE`) Mathematical Specification

This specification defines the exact fixed-point `Decimal` formulas executed in `crates/qlevel-engine/src/commercial_calc.rs`.

## 1. Currency Scale & Rounding

Each `CurrencyCode` enforces an ISO 4217 minor-unit scale using `MidpointAwayFromZero` (standard commercial half-up rounding):

- **2 Decimal Places (`0.01`)**: `SAR` ([ZATCA VAT](https://zatca.gov.sa/en/RulesRegulations/VAT/Pages/default.aspx)), `EGP` ([Egypt Tax Authority VAT](https://eta.gov.eg/ar/content/qwanyn-aldrybt-ly-alqymt-almdaft)), `AED`, `QAR`, `USD`, `EUR`.
- **3 Decimal Places (`0.001`)**: `KWD`, `BHD`, `OMR`.

Intermediate unit-rate stripping and discount allocations retain at least `6` decimal places before final line-total and summary rounding.

## 2. Vendor Arithmetic Audit & Quantity Tamper Normalization

Let $Q_m$ be the Master BOQ quantity, $Q_v$ be the vendor's stated quantity, $R_v$ be the vendor's quoted unit rate, and $A_v$ be the vendor's printed extended amount:

1. **Arithmetic Mismatch Check**: Compute $E_v = Q_v \times R_v$. If $|E_v - A_v| > \text{tolerance}$ (where tolerance is $10^{-\text{scale}}$), emit `TrapCategory::QtyArithmetic` recording both $A_v$ (as-submitted) and $E_v$ (true multiplication).
2. **Quantity Equalization**: All leveled base totals normalize onto the Master BOQ quantity $Q_m$:
   $$L_{\text{raw}}(i) = Q_m(i) \times R_{\text{net}}(i)$$
   If $Q_v(i) \neq Q_m(i)$, emit `TrapCategory::QtyArithmetic` with exact delta $(Q_m - Q_v) \times R_{\text{net}}$.

## 3. Tax (`VAT`) Stripping to Net Exclusive Baseline

All commercial comparisons equalize on a **Net VAT-Exclusive** baseline before applying the project's statutory VAT rate $v_{\text{proj}}$ (e.g., `15%` in KSA, `14%` general contracting rate in Egypt) once at the grand-total footer:

- If vendor quotes `VatMode::Exclusive`, `Exempt`, or `ZeroRated`:
  $$R_{\text{net}} = R_v$$
- If vendor quotes `VatMode::Inclusive` at rate $v_v \in (0, 100)$:
  $$R_{\text{net}} = \frac{R_v}{1 + \frac{v_v}{100}}$$

## 4. RICS NRM 2 Line-Item Taxonomies

In accordance with [RICS NRM 2](https://www.rics.org/content/dam/ricsglobal/documents/standards/NRM-2_Oct2022_Update.pdf) and [RICS Tendering Strategies](https://www.rics.org/content/dam/ricsglobal/documents/standards/tendering_strategies_1st_edition_rics.pdf):

- **`Measured`**: Normal measured work item included in base commercial totals and eligible for general tender discount proration.
- **`ProvisionalSum`**: Fixed employer allowance $P_m$. Any vendor modification or discount on a Provisional Sum is overridden back to $P_m$ during leveling so no bidder gains an artificial ranking advantage by undercutting a fixed allowance (`CellStatus::UnderPricedAllowance`).
- **`RateOnly`**: Unit rate recorded and compared for variation valuation, with base contract quantity $0$ in the measured sum.
- **`Optional` / `Alternative`**: Kept in dedicated memorandum rows outside the mandatory base contract sum so optional add-ons never distort base low-bidder selection.

## 5. Merged Vendor Rows (Zero Double-Counting Invariant)

When a vendor merges Master BOQ items $\{i_1, i_2, \dots, i_k\}$ into a single lump sum $M_v$:

- **`YellowHighlightAuto` Mode**: Item $i_1$ is designated the **Anchor** carrying $M_v$ (net of VAT) and highlighted in Amber/Yellow (`#FFF3CD`), while follower items $i_2 \dots i_k$ carry `CellStatus::IncludedIn { anchor_boq_id: i_1 }` with numeric leveled value `0` and note `"Included in <i_1>"`. Thus $\sum_{j=1}^k L(i_j) = M_v$ with **zero double counting** and **zero artificial missing-scope plugs** on follower items.
- **`InteractiveReview` Mode**: Pauses on `AmbiguityPromptPayload` so the engineer either accepts the equal lump sum (anchor + zero followers), distributes proportionally across peer weights, flags yellow, or excludes with a plug.

## 6. Missing-Scope Equalization Plugs (`PlugBasis`)

For genuinely `Unquoted` or excluded `Measured` items (excluding `IncludedIn` followers), `qlevel` computes a transparent commercial plug $P(i)$ stored in a separate **Scope Equalization Plug** column (never overwriting the vendor's submitted cell):

- **Peer Basis** (`LowestCompliant` / `AverageCompliant` / `HighestCompliant` / `PteEstimate`): Computes the min, arithmetic mean, or max net leveled amount among compliant peer bidders who priced item $i$, or falls back to the Master BOQ Pre-Tender Estimate (`PTE`) if present.
- **Clause-Level Plugs** (`DetectedTrap` with `PlugBasis`):
  - `AnchoredToBoq { boq_id }`: Plugs excluded sub-scope referenced to a BOQ item.
  - `PctOfQuotedOnly { pct }`: Applies percentage loading (such as excluded testing/commissioning or customs/DDP uplift per [ICC Incoterms 2020](https://iccwbo.org/business-solutions/incoterms-rules/incoterms-2020/)) strictly on net quoted measured work (never compounding on other plugs).
  - `ProratedLumpSum { amount }` / `FixedSetupFee { amount }`: Adds a discrete commercial risk adjustment in the Equalization Summary block.

## 7. True Leveled Net & Gross Totals

For each vendor $v$:
$$\text{Net Quoted Measured}(v) = \sum_{i \in \text{Measured}} L_{\text{discounted}}(i)$$
$$\text{Leveled Net Total}(v) = \text{Net Quoted Measured}(v) + \sum_{p \in \text{ProvisionalSum}} P_m(p) + \text{Total Scope Plugs}(v)$$
$$\text{Leveled Gross Total}(v) = \text{round}\left(\text{Leveled Net Total}(v) \times \left(1 + \frac{v_{\text{proj}}}{100}\right), \text{scale}\right)$$

The **True Low Bidder** is the vendor minimizing $\text{Leveled Net Total}(v)$.
