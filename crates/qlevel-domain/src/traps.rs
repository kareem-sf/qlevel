//! Technical & commercial trap taxonomy (`7` built-in categories) and equalization plugs.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::decimal::DecimalValue;

/// The 7 canonical tender trap categories audited by `qlevel`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum TrapCategory {
    ScopeExclusion,
    TechnicalDeviation,
    AvlRegulatory,
    QtyArithmetic,
    LogisticsIncoterms,
    CommercialRisk,
    CloseoutDocs,
}

/// Risk severity level assigned to a detected trap.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum TrapSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Explicit mathematical basis for pricing a clause-level exclusion without double counting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "basis", rename_all = "snake_case")]
pub enum PlugBasis {
    /// Links excluded scope directly to an unquoted Master BOQ item ID.
    AnchoredToBoq { boq_id: String },
    /// Applies `pct` strictly on net quoted measured items (never compounding on other plugs).
    PctOfQuotedOnly { pct: DecimalValue },
    /// Adds a prorated lump-sum allowance in project currency.
    ProratedLumpSum { amount: DecimalValue },
    /// Adds a fixed mobilization, testing, or certification fee in project currency.
    FixedSetupFee { amount: DecimalValue },
}

/// Concrete commercial or technical trap discovered on a vendor quotation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct DetectedTrap {
    pub id: String,
    pub vendor_id: String,
    pub category: TrapCategory,
    pub severity: TrapSeverity,
    pub related_boq_id: Option<String>,
    pub title: String,
    pub detail: String,
    pub source_snippet: String,
    pub source_page: Option<u32>,
    pub plug_basis: Option<PlugBasis>,
    pub computed_impact: Option<DecimalValue>,
}
