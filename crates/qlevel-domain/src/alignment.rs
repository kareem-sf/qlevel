//! Merged-item alignment groups and user-selectable resolution modes.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::decimal::DecimalValue;

/// Cardinality between Master BOQ rows and Vendor quotation rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum GroupCardinality {
    OneToOne,
    ManyMasterToOneVendor,
    OneMasterToManyVendor,
}

/// User-selectable workflow when a vendor merges multiple BOQ items into one lump sum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum MergedItemMode {
    /// Pause and display a plain-language quick resolution card (`Basic` interactive flow).
    InteractiveReview,
    /// Anchor lump sum on the lead item with `0` on followers and highlight yellow in Excel.
    YellowHighlightAuto,
}

/// Concrete resolution applied to an `AlignmentGroup`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum AlignmentResolution {
    PendingReview,
    AnchorAndZeroFollowers,
    ProportionalSplit,
    AcceptedAlternate,
    ExcludedWithPlug,
}

/// Group linking one or more Master BOQ items to one or more Vendor rows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct AlignmentGroup {
    pub group_id: String,
    pub vendor_id: String,
    pub cardinality: GroupCardinality,
    pub master_boq_ids: Vec<String>,
    pub vendor_descriptions: Vec<String>,
    pub quoted_lump_sum: DecimalValue,
    pub confidence: DecimalValue,
    pub resolution: AlignmentResolution,
    pub reviewer_note: Option<String>,
}
