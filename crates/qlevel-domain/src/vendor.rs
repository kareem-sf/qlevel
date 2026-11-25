//! Vendor quotation models, cell equalization statuses, and commercial terms.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::currency::{CurrencyCode, VatMode};
use crate::decimal::DecimalValue;

/// Cell-level commercial pricing state relative to the Master BOQ item.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum CellStatus {
    /// Explicitly priced by the vendor.
    Quoted,
    /// Merged into another BOQ item's lump sum (`0` follower value, zero double-counting).
    IncludedIn { anchor_boq_id: String },
    /// Vendor moved a mandatory base item into an optional add-on section.
    MovedToOptional,
    /// Vendor left the item blank, `N/A`, `Excluded`, or `By Others`.
    Unquoted,
    /// Vendor attempted to undercut or omit a fixed Provisional Sum allowance.
    UnderPricedAllowance,
}

/// Extracted and aligned line item from a single subcontractor quotation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct VendorLineQuote {
    pub boq_item_id: String,
    pub vendor_code: Option<String>,
    pub vendor_description: String,
    pub quoted_unit: String,
    pub quoted_qty: DecimalValue,
    pub quoted_unit_rate: DecimalValue,
    pub quoted_extended_total: DecimalValue,
    pub vat_mode: VatMode,
    pub vendor_vat_pct: Option<DecimalValue>,
    pub status: CellStatus,
    pub proposed_brand: Option<String>,
    pub source_page: Option<u32>,
    pub notes: Option<String>,
}

/// Commercial terms and tender-level discounts stated in a vendor quotation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct CommercialTerms {
    pub payment_terms: Option<String>,
    pub delivery_incoterms: Option<String>,
    pub delivery_weeks: Option<u32>,
    pub validity_days: Option<u32>,
    pub warranty_months: Option<u32>,
    pub general_discount_pct: DecimalValue,
    pub lump_sum_discount: DecimalValue,
}

impl Default for CommercialTerms {
    fn default() -> Self {
        Self {
            payment_terms: None,
            delivery_incoterms: None,
            delivery_weeks: None,
            validity_days: None,
            warranty_months: None,
            general_discount_pct: DecimalValue::ZERO,
            lump_sum_discount: DecimalValue::ZERO,
        }
    }
}

/// Complete subcontractor quotation aligned for commercial and technical evaluation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct VendorQuote {
    pub vendor_id: String,
    pub vendor_name: String,
    pub currency: CurrencyCode,
    /// Exchange rate multiplier converting `1` unit of `currency` into project currency (`1` when identical).
    pub fx_rate_to_project: DecimalValue,
    pub default_vat_mode: VatMode,
    pub stated_grand_total: Option<DecimalValue>,
    pub terms: CommercialTerms,
    pub lines: Vec<VendorLineQuote>,
    pub raw_exclusion_clauses: Vec<String>,
}
