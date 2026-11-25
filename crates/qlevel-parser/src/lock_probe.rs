//! Pre-export file lock detector for Windows Excel (`ERROR_SHARING_VIOLATION`) and fallback naming.

use std::fs::OpenOptions;
use std::path::{Path, PathBuf};

/// Prefix written by Microsoft Excel (`ASCII 0x7E` + `$`) beside an open `.xlsx` workbook.
const EXCEL_OWNER_LOCK_PREFIX: &str = "\u{007e}$";

/// Result of checking whether a target `.xlsx` path can be overwritten safely.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileLockStatus {
    Writable,
    Locked {
        locked_path: PathBuf,
        suggested_fallback: PathBuf,
        hint: String,
    },
}

/// Computes an unlocked versioned sibling filename (e.g. `Comparison_v2.xlsx`, `Comparison_v3.xlsx`).
#[must_use]
pub fn suggest_unlocked_version_path(original_path: &Path) -> PathBuf {
    let parent = original_path.parent().unwrap_or_else(|| Path::new("."));
    let ext = original_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("xlsx");
    let raw_stem = original_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Bid_Leveling");

    let base_stem = match raw_stem.rsplit_once("_v") {
        Some((prefix, num_str))
            if !prefix.is_empty() && num_str.chars().all(|c| c.is_ascii_digit()) =>
        {
            prefix
        }
        _ => raw_stem,
    };

    for version in 2..=99_u32 {
        let candidate = parent.join(format!("{base_stem}_v{version}.{ext}"));
        if !candidate.exists() {
            return candidate;
        }
    }
    parent.join(format!("{base_stem}_export.{ext}"))
}

/// Probes whether `target_path` is locked by Microsoft Excel or another OS process without
/// truncating existing file contents.
#[must_use]
pub fn probe_output_path_writable(target_path: &Path) -> FileLockStatus {
    if let (Some(parent), Some(file_name)) = (
        target_path.parent(),
        target_path.file_name().and_then(|n| n.to_str()),
    ) {
        let excel_lock_file = parent.join(format!("{EXCEL_OWNER_LOCK_PREFIX}{file_name}"));
        if excel_lock_file.exists() {
            return FileLockStatus::Locked {
                locked_path: target_path.to_path_buf(),
                suggested_fallback: suggest_unlocked_version_path(target_path),
                hint: "Microsoft Excel owner lock file detected".to_owned(),
            };
        }
    }

    if !target_path.exists() {
        return FileLockStatus::Writable;
    }

    match OpenOptions::new().append(true).open(target_path) {
        Ok(_handle) => FileLockStatus::Writable,
        Err(err) => FileLockStatus::Locked {
            locked_path: target_path.to_path_buf(),
            suggested_fallback: suggest_unlocked_version_path(target_path),
            hint: err.to_string(),
        },
    }
}
