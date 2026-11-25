//! Native Excel 2016+ `.xlsx` comparison workbook generator for `qlevel`.
//!
//! Supports both user-selectable layouts (`SingleMasterSheet` and `DualTab`),
//! automatic or explicit `RTL`/`LTR` direction, classic formulas (`SUM`, `MIN`, `ROUND`, `IF`)
//! with cached formula results, amber highlighting for merged vendor rows, and OS file-lock guards.

#![forbid(unsafe_code)]

pub mod error;
pub mod exporter;
pub mod formulas;
pub mod layouts;
pub mod styles;

pub use error::{WorkbookError, WorkbookResult};
pub use exporter::{
    export_leveling_workbook_bytes, save_leveling_workbook_to_path, WorkbookWriteOutcome,
};
pub use formulas::{col_idx_to_excel_letters, sanitize_sheet_name, validate_excel_2016_formula};
