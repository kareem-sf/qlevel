//! Structured error types for `qlevel-workbook`.

use std::path::PathBuf;

/// Canonical error returned during `.xlsx` generation or disk export.
#[derive(Debug, thiserror::Error)]
pub enum WorkbookError {
    #[error("Excel writer failure: {0}")]
    Xlsx(#[from] rust_xlsxwriter::XlsxError),

    #[error("Forbidden modern `_xlfn.` formula detected ('{formula}'); classic Excel 2016+ syntax required")]
    ForbiddenDynamicFormula { formula: String },

    #[error("Filesystem write error at '{path}': {reason}")]
    Io { path: PathBuf, reason: String },
}

/// Convenience result alias for workbook export operations.
pub type WorkbookResult<T> = Result<T, WorkbookError>;
