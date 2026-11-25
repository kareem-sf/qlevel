//! Byte-buffer and file-lock-aware disk exporter for `.xlsx` comparison workbooks.

use std::fs;
use std::path::{Path, PathBuf};

use qlevel_domain::LevelingResult;
use qlevel_parser::{probe_output_path_writable, FileLockStatus};

use crate::error::{WorkbookError, WorkbookResult};
use crate::layouts::build_workbook;

/// Outcome of writing a `.xlsx` workbook to disk, handling locked open Excel files gracefully.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkbookWriteOutcome {
    Saved {
        output_path: PathBuf,
    },
    FileLocked {
        locked_path: PathBuf,
        suggested_fallback_path: PathBuf,
        hint: String,
    },
}

/// Renders a `LevelingResult` into an in-memory `.xlsx` byte buffer.
///
/// # Errors
/// Returns [`WorkbookError`] if `rust_xlsxwriter` serialization or formula checks fail.
pub fn export_leveling_workbook_bytes(result: &LevelingResult) -> WorkbookResult<Vec<u8>> {
    let mut wb = build_workbook(result)?;
    Ok(wb.save_to_buffer()?)
}

/// Saves a `LevelingResult` to `target_path`, checking for Excel file locks first.
///
/// When `use_fallback_if_locked` is `true`, automatically writes to the suggested `_v2.xlsx`
/// sibling path if `target_path` is currently open in Microsoft Excel.
///
/// # Errors
/// Returns [`WorkbookError`] if generating bytes or writing to the resolved path fails.
pub fn save_leveling_workbook_to_path(
    result: &LevelingResult,
    target_path: &Path,
    use_fallback_if_locked: bool,
) -> WorkbookResult<WorkbookWriteOutcome> {
    let bytes = export_leveling_workbook_bytes(result)?;

    let resolved_path = match probe_output_path_writable(target_path) {
        FileLockStatus::Writable => target_path.to_path_buf(),
        FileLockStatus::Locked {
            locked_path,
            suggested_fallback,
            hint,
        } => {
            if use_fallback_if_locked {
                suggested_fallback
            } else {
                return Ok(WorkbookWriteOutcome::FileLocked {
                    locked_path,
                    suggested_fallback_path: suggested_fallback,
                    hint,
                });
            }
        }
    };

    if let Some(parent) = resolved_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|err| WorkbookError::Io {
                path: parent.to_path_buf(),
                reason: err.to_string(),
            })?;
        }
    }

    fs::write(&resolved_path, &bytes).map_err(|err| WorkbookError::Io {
        path: resolved_path.clone(),
        reason: err.to_string(),
    })?;

    Ok(WorkbookWriteOutcome::Saved {
        output_path: resolved_path,
    })
}
