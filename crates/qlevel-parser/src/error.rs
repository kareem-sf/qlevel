//! Structured ingestion and parsing errors for `qlevel-parser`.

use qlevel_domain::DomainError;

/// Canonical error returned during Excel, PDF, or clipboard ingestion.
#[derive(Debug, thiserror::Error)]
pub enum ParserError {
    #[error("File '{file_name}' is password-protected or encrypted; please save an unencrypted copy")]
    PasswordProtected { file_name: String },

    #[error("Unsupported file extension on '{file_name}' (expected .xlsx, .xlsm, .xls, .xlsb, .csv, or .pdf)")]
    UnsupportedFormat { file_name: String },

    #[error("Corrupted or unreadable workbook '{file_name}': {reason}")]
    WorkbookCorrupted { file_name: String, reason: String },

    #[error("Corrupted or unreadable PDF '{file_name}': {reason}")]
    PdfCorrupted { file_name: String, reason: String },

    #[error("No tabular rows found in '{source_name}'")]
    EmptyTable { source_name: String },

    #[error("Could not identify BOQ description and quantity/rate columns in '{source_name}'")]
    HeaderNotFound { source_name: String },

    #[error("Filesystem I/O error on '{path}': {reason}")]
    Io { path: String, reason: String },

    #[error("Domain conversion error: {0}")]
    Domain(#[from] DomainError),
}

/// Convenience result alias for parser operations.
pub type ParserResult<T> = Result<T, ParserError>;
