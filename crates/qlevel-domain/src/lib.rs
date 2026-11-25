//! Pure domain types and invariants for `qlevel`.
//!
//! This crate enforces zero floating-point arithmetic (`#![deny(clippy::float_arithmetic)]`),
//! zero unsafe blocks, and zero runtime panics across all BOQ, vendor quotation,
//! alignment group, technical trap, and commercial leveling structures.

#![forbid(unsafe_code)]
#![deny(clippy::float_arithmetic)]

pub mod alignment;
pub mod boq;
pub mod currency;
pub mod decimal;
pub mod error;
pub mod session;
pub mod traps;
pub mod vendor;

pub use alignment::{
    AlignmentGroup, AlignmentResolution, GroupCardinality, MergedItemMode,
};
pub use boq::{BoqItem, LineItemKind, MasterBoq};
pub use currency::{CurrencyCode, Locale, SheetDirection, VatMode, VatPolicy};
pub use decimal::DecimalValue;
pub use error::{DomainError, DomainResult};
pub use session::{
    AiConfig, AuthMethod, EffortLevel, LeveledLineCell, LevelingResult,
    LevelingSessionConfig, ModelInfo, PeerPlugStrategy, ProviderKind,
    VendorEvaluationSummary, WorkbookLayoutMode,
};
pub use traps::{DetectedTrap, PlugBasis, TrapCategory, TrapSeverity};
pub use vendor::{CellStatus, CommercialTerms, VendorLineQuote, VendorQuote};
