//! User-space portable executable self-updater (`self_update` + `self-replace`) with SHA-256 verification.

use std::env;
use std::fs::OpenOptions;
use std::path::Path;

use sha2::{Digest, Sha256};

use crate::dto::UpdateCheckStatusDto;
use crate::error::{BridgeError, BridgeResult};

const GITHUB_REPO_OWNER: &str = "sfkareem";
const GITHUB_REPO_NAME: &str = "qlevel";
const BIN_NAME: &str = "qlevel-desktop";

/// Verifies that `artifact_bytes` matches `expected_sha256_hex` (case-insensitive).
///
/// # Errors
/// Returns [`BridgeError::UpdateChecksumMismatch`] if digests differ.
pub fn verify_sha256_hex(artifact_bytes: &[u8], expected_sha256_hex: &str) -> BridgeResult<()> {
    let computed = format!("{:x}", Sha256::digest(artifact_bytes));
    let clean_expected = expected_sha256_hex.trim().to_lowercase();
    if computed != clean_expected {
        return Err(BridgeError::UpdateChecksumMismatch {
            expected: clean_expected,
            actual: computed,
        });
    }
    Ok(())
}

/// Probes whether `dir` allows creating and removing a temporary file in user space
/// (prevents failed updates inside read-only corporate `Program Files` folders).
pub fn ensure_directory_writable(dir: &Path) -> BridgeResult<()> {
    let probe_path = dir.join(".qlevel_update_write_probe.tmp");
    match OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&probe_path)
    {
        Ok(_) => {
            let _ = std::fs::remove_file(&probe_path);
            Ok(())
        }
        Err(_) => Err(BridgeError::UpdateDirNotWritable {
            path: dir.display().to_string(),
        }),
    }
}

/// Atomically swaps the currently running portable executable with a SHA-256 verified binary.
///
/// # Errors
/// Returns [`BridgeError`] if directory is read-only, SHA-256 mismatches, or `self-replace` fails.
pub fn replace_running_portable_binary(
    verified_new_binary_path: &Path,
    expected_sha256_hex: &str,
) -> BridgeResult<()> {
    let current_exe = env::current_exe().map_err(|e| BridgeError::Internal(e.to_string()))?;
    if let Some(parent_dir) = current_exe.parent() {
        ensure_directory_writable(parent_dir)?;
    }

    let bytes = std::fs::read(verified_new_binary_path)
        .map_err(|e| BridgeError::Internal(e.to_string()))?;
    verify_sha256_hex(&bytes, expected_sha256_hex)?;

    self_replace::self_replace(verified_new_binary_path)
        .map_err(|e| BridgeError::Internal(format!("Portable executable swap failed: {e}")))
}

/// Checks GitHub Releases (`sfkareem/qlevel`) for a newer SemVer release and optionally updates.
///
/// # Errors
/// Returns [`BridgeError`] if network or directory writability checks fail.
pub async fn check_or_apply_portable_update(
    apply_update: bool,
) -> BridgeResult<UpdateCheckStatusDto> {
    let current_version = env!("CARGO_PKG_VERSION").to_owned();

    if apply_update {
        let current_exe = env::current_exe().map_err(|e| BridgeError::Internal(e.to_string()))?;
        if let Some(parent) = current_exe.parent() {
            ensure_directory_writable(parent)?;
        }
    }

    tokio::task::spawn_blocking(move || {
        let updater = self_update::backends::github::Update::configure()
            .repo_owner(GITHUB_REPO_OWNER)
            .repo_name(GITHUB_REPO_NAME)
            .bin_name(BIN_NAME)
            .current_version(&current_version)
            .no_confirm(true)
            .show_download_progress(false)
            .build()
            .map_err(|e| BridgeError::Internal(e.to_string()))?;

        let latest = updater
            .get_latest_release()
            .map_err(|e| BridgeError::Network(e.to_string()))?;

        let update_available =
            self_update::version::bump_is_greater(&current_version, &latest.version)
                .unwrap_or(false);

        let mut applied = false;
        if update_available && apply_update {
            let status = updater
                .update()
                .map_err(|e| BridgeError::Internal(e.to_string()))?;
            applied = status.updated();
        }

        Ok(UpdateCheckStatusDto {
            current_version,
            latest_version: Some(latest.version),
            update_available,
            applied_portable_update: applied,
        })
    })
    .await
    .map_err(|e| BridgeError::Internal(e.to_string()))?
}
