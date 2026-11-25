//! Structured error types for `qlevel-engine`.

use qlevel_ai::AiError;
use qlevel_domain::DomainError;
use qlevel_parser::ParserError;

/// Canonical error returned by the leveling engine and checkpoint database.
#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("Domain math or invariant error: {0}")]
    Domain(#[from] DomainError),

    #[error("Document parser error: {0}")]
    Parser(#[from] ParserError),

    #[error("AI provider error: {0}")]
    Ai(#[from] AiError),

    #[error("SQLite checkpoint database failure: {0}")]
    CheckpointDb(String),

    #[error("At least one vendor quotation is required to run bid leveling")]
    NoVendorQuotes,

    #[error("No Multimodal models available on the selected provider")]
    NoAvailableModels,

    #[error("Pending interactive merged-item review required for group '{group_id}'")]
    PendingInteractiveResolution { group_id: String },

    #[error("Leveling session cancelled by user")]
    Cancelled,
}

/// Convenience result alias for `qlevel-engine`.
pub type EngineResult<T> = Result<T, EngineError>;
