//! Domain error types for `qlevel-domain`.

use serde::{Deserialize, Serialize};

/// Canonical domain invariant error.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, thiserror::Error)]
pub enum DomainError {
    #[error("Invalid decimal string '{raw}': {reason}")]
    InvalidDecimal { raw: String, reason: String },

    #[error("Arithmetic overflow during '{operation}'")]
    ArithmeticOverflow { operation: &'static str },

    #[error("Division by zero during '{operation}'")]
    DivisionByZero { operation: &'static str },

    #[error("Percentage value '{value}' is outside valid range [{min}, {max}]")]
    InvalidPercentage {
        value: String,
        min: String,
        max: String,
    },

    #[error("Master BOQ must contain at least one line item")]
    EmptyMasterBoq,

    #[error("Duplicate BOQ item identifier '{id}'")]
    DuplicateBoqId { id: String },

    #[error("Negative BOQ quantity '{qty}' on item '{id}'")]
    NegativeQuantity { id: String, qty: String },

    #[error("Foreign currency quote '{vendor_currency}' requires positive FX rate to '{project_currency}'")]
    InvalidFxRate {
        vendor_currency: String,
        project_currency: String,
    },

    #[error("Domain invariant violated: {message}")]
    InvariantViolation { message: String },
}

/// Convenience result alias for domain operations.
pub type DomainResult<T> = Result<T, DomainError>;
