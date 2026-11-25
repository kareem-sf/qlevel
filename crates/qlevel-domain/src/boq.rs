//! Master Bill of Quantities (`MasterBoq`) and RICS NRM 2 item classifications.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::currency::CurrencyCode;
use crate::decimal::DecimalValue;
use crate::error::{DomainError, DomainResult};

/// RICS NRM 2 classification governing commercial leveling behavior.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LineItemKind {
    /// Standard measured work (`Qty * NetRate`) included in base tender sum.
    Measured,
    /// Fixed employer Provisional Sum equalized identically across all bidders.
    ProvisionalSum,
    /// Unit-rate-only item recorded for variations; contributes `0` to base contract sum.
    RateOnly,
    /// Employer optional add-on priced outside the mandatory base sum.
    Optional,
    /// Bidder value-engineering alternative referencing a base item ID.
    Alternative { replaces_item_id: String },
}

/// Single baseline row inside a Master BOQ (either imported or synthesized).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct BoqItem {
    pub id: String,
    pub code: String,
    pub section: Option<String>,
    pub description: String,
    pub unit: String,
    pub quantity: DecimalValue,
    pub kind: LineItemKind,
    /// Optional Pre-Tender Estimate (`PTE`) unit rate or fixed Provisional Sum allowance.
    pub pte_unit_rate: Option<DecimalValue>,
}

impl BoqItem {
    /// Computes the Pre-Tender Estimate (`PTE`) extended total (`quantity * pte_unit_rate`).
    ///
    /// # Errors
    /// Returns [`DomainError::ArithmeticOverflow`] if multiplication overflows.
    pub fn pte_extended_total(&self, scale: u32) -> DomainResult<Option<DecimalValue>> {
        match self.pte_unit_rate {
            Some(rate) => {
                if matches!(self.kind, LineItemKind::RateOnly) {
                    return Ok(Some(DecimalValue::ZERO));
                }
                let raw = self.quantity.checked_mul(rate)?;
                Ok(Some(raw.round_commercial(scale)))
            }
            None => Ok(None),
        }
    }
}

/// Complete Master Bill of Quantities baseline for a leveling session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct MasterBoq {
    pub title: String,
    pub currency: CurrencyCode,
    /// `true` when automatically synthesized from raw vendor quotes without a client BOQ.
    pub is_synthetic: bool,
    pub items: Vec<BoqItem>,
}

impl MasterBoq {
    /// Validates unique non-empty item IDs and non-negative quantities.
    ///
    /// # Errors
    /// Returns [`DomainError`] if `items` is empty, contains duplicates, or has negative quantities.
    pub fn validate(&self) -> DomainResult<()> {
        if self.items.is_empty() {
            return Err(DomainError::EmptyMasterBoq);
        }
        let mut seen = HashSet::with_capacity(self.items.len());
        for item in &self.items {
            if item.id.trim().is_empty() {
                return Err(DomainError::InvariantViolation {
                    message: "BOQ item ID must not be empty".to_owned(),
                });
            }
            if !seen.insert(item.id.as_str()) {
                return Err(DomainError::DuplicateBoqId {
                    id: item.id.clone(),
                });
            }
            if item.quantity.is_negative() {
                return Err(DomainError::NegativeQuantity {
                    id: item.id.clone(),
                    qty: item.quantity.to_string(),
                });
            }
        }
        Ok(())
    }
}
