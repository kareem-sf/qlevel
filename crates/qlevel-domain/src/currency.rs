//! Supported MENA/global currencies, locales, and VAT policy definitions.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::decimal::DecimalValue;
use crate::error::{DomainError, DomainResult};

/// Supported tender currencies across GCC, Egypt, and international contracts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
pub enum CurrencyCode {
    SAR,
    EGP,
    AED,
    QAR,
    KWD,
    BHD,
    OMR,
    USD,
    EUR,
}

impl CurrencyCode {
    /// Minor-unit decimal scale per ISO 4217 (`3` for `KWD`/`BHD`/`OMR`, `2` otherwise).
    #[must_use]
    pub const fn decimal_scale(self) -> u32 {
        match self {
            Self::KWD | Self::BHD | Self::OMR => 3,
            Self::SAR | Self::EGP | Self::AED | Self::QAR | Self::USD | Self::EUR => 2,
        }
    }

    /// Standard statutory VAT percentage default (`15%` KSA, `14%` Egypt, `5%` UAE/Bahrain/Oman).
    #[must_use]
    pub fn default_vat_pct(self) -> DecimalValue {
        match self {
            Self::SAR => DecimalValue::from_parts_i64(15, 0),
            Self::EGP => DecimalValue::from_parts_i64(14, 0),
            Self::AED | Self::BHD | Self::OMR => DecimalValue::from_parts_i64(5, 0),
            Self::QAR | Self::KWD | Self::USD | Self::EUR => DecimalValue::ZERO,
        }
    }

    /// Arithmetic comparison tolerance equal to `1` minor currency unit (`0.01` or `0.001`).
    #[must_use]
    pub fn minor_unit_tolerance(self) -> DecimalValue {
        DecimalValue::from_parts_i64(1, self.decimal_scale())
    }

    /// Classic Excel number format string matching the currency's minor-unit precision.
    #[must_use]
    pub const fn excel_number_format(self) -> &'static str {
        match self.decimal_scale() {
            3 => "#,##0.000",
            _ => "#,##0.00",
        }
    }

    /// Canonical 3-letter ISO code string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SAR => "SAR",
            Self::EGP => "EGP",
            Self::AED => "AED",
            Self::QAR => "QAR",
            Self::KWD => "KWD",
            Self::BHD => "BHD",
            Self::OMR => "OMR",
            Self::USD => "USD",
            Self::EUR => "EUR",
        }
    }
}

/// UI and report language selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum Locale {
    Auto,
    Ar,
    En,
}

/// Worksheet layout direction (`RTL` for Arabic tenders, `LTR` for English tenders).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum SheetDirection {
    Auto,
    Rtl,
    Ltr,
}

/// How VAT is treated inside a vendor's quoted unit rates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum VatMode {
    Exclusive,
    Inclusive,
    Exempt,
    ZeroRated,
}

/// Project-wide VAT equalization policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct VatPolicy {
    pub project_vat_pct: DecimalValue,
    pub default_vendor_mode: VatMode,
}

impl VatPolicy {
    /// Validates that `project_vat_pct` lies within `[0, 100)`.
    ///
    /// # Errors
    /// Returns [`DomainError::InvalidPercentage`] if negative or `>= 100`.
    pub fn validate(&self) -> DomainResult<()> {
        if self.project_vat_pct.is_negative() || self.project_vat_pct >= DecimalValue::ONE_HUNDRED {
            return Err(DomainError::InvalidPercentage {
                value: self.project_vat_pct.to_string(),
                min: "0".to_owned(),
                max: "99.99".to_owned(),
            });
        }
        Ok(())
    }

    /// Strips inclusive VAT from `gross_amount` (`gross / (1 + vat_pct / 100)`).
    ///
    /// # Errors
    /// Returns [`DomainError`] if `vat_pct` is invalid or division overflows.
    pub fn strip_to_net_exclusive(
        &self,
        amount: DecimalValue,
        mode: VatMode,
        override_vendor_vat_pct: Option<DecimalValue>,
    ) -> DomainResult<DecimalValue> {
        match mode {
            VatMode::Exclusive | VatMode::Exempt | VatMode::ZeroRated => Ok(amount),
            VatMode::Inclusive => {
                let effective_pct = override_vendor_vat_pct.unwrap_or(self.project_vat_pct);
                if effective_pct.is_negative() || effective_pct >= DecimalValue::ONE_HUNDRED {
                    return Err(DomainError::InvalidPercentage {
                        value: effective_pct.to_string(),
                        min: "0".to_owned(),
                        max: "99.99".to_owned(),
                    });
                }
                if effective_pct.is_zero() {
                    return Ok(amount);
                }
                let factor = effective_pct
                    .checked_div(DecimalValue::ONE_HUNDRED)?
                    .checked_add(DecimalValue::ONE)?;
                amount.checked_div(factor)
            }
        }
    }
}
