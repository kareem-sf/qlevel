//! Local `SQLite WAL` batch checkpoint store keyed by canonical `BLAKE3` digests.
//!
//! Ensures 150-page tender runs resume instantaneously from the exact completed batch
//! after network drops or rate limits without re-billing completed batches.

use std::path::Path;
use std::sync::Mutex;

use chrono::Utc;
use qlevel_ai::ExtractedVendorBatch;
use qlevel_domain::EffortLevel;
use rusqlite::{params, Connection, OptionalExtension};

use crate::error::{EngineError, EngineResult};

const CHECKPOINT_SCHEMA_VERSION: &str = "qlevel.checkpoint.v1";

/// Computes a deterministic `BLAKE3` hex key uniquely identifying an extraction batch.
#[must_use]
pub fn compute_batch_blake3_key(
    provider_name: &str,
    model_id: &str,
    effort: EffortLevel,
    document_blake3: &str,
    batch_index: u32,
    master_boq_fingerprint: &str,
    custom_rules: &[String],
) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(CHECKPOINT_SCHEMA_VERSION.as_bytes());
    hasher.update(b"|");
    hasher.update(provider_name.as_bytes());
    hasher.update(b"|");
    hasher.update(model_id.as_bytes());
    hasher.update(b"|");
    let effort_tag = match effort {
        EffortLevel::Auto => "auto",
        EffortLevel::Low => "low",
        EffortLevel::Medium => "medium",
        EffortLevel::High => "high",
    };
    hasher.update(effort_tag.as_bytes());
    hasher.update(b"|");
    hasher.update(document_blake3.as_bytes());
    hasher.update(b"|");
    hasher.update(&batch_index.to_le_bytes());
    hasher.update(b"|");
    hasher.update(master_boq_fingerprint.as_bytes());
    for rule in custom_rules {
        hasher.update(b"|rule:");
        hasher.update(rule.trim().as_bytes());
    }
    hasher.finalize().to_hex().to_string()
}

/// Thread-safe `SQLite WAL` store persisting extracted vendor batches (never credentials).
pub struct CheckpointStore {
    conn: Mutex<Connection>,
}

impl CheckpointStore {
    /// Opens or creates a persistent `SQLite WAL` checkpoint file on disk.
    ///
    /// # Errors
    /// Returns [`EngineError::CheckpointDb`] if SQLite initialization fails.
    pub fn open_file(path: &Path) -> EngineResult<Self> {
        let conn =
            Connection::open(path).map_err(|err| EngineError::CheckpointDb(err.to_string()))?;
        Self::init_connection(conn)
    }

    /// Opens an in-memory checkpoint store for unit tests and ephemeral runs.
    ///
    /// # Errors
    /// Returns [`EngineError::CheckpointDb`] if SQLite initialization fails.
    pub fn open_in_memory() -> EngineResult<Self> {
        let conn = Connection::open_in_memory()
            .map_err(|err| EngineError::CheckpointDb(err.to_string()))?;
        Self::init_connection(conn)
    }

    fn init_connection(conn: Connection) -> EngineResult<Self> {
        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(|err| EngineError::CheckpointDb(err.to_string()))?;
        conn.pragma_update(None, "synchronous", "NORMAL")
            .map_err(|err| EngineError::CheckpointDb(err.to_string()))?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS batch_checkpoints (
                blake3_key TEXT PRIMARY KEY NOT NULL,
                session_id TEXT NOT NULL,
                vendor_name TEXT NOT NULL,
                payload_json TEXT NOT NULL,
                created_at_epoch INTEGER NOT NULL
            );",
        )
        .map_err(|err| EngineError::CheckpointDb(err.to_string()))?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Looks up a previously completed extraction batch by its `BLAKE3` key.
    ///
    /// # Errors
    /// Returns [`EngineError::CheckpointDb`] on SQLite read or JSON deserialization errors.
    pub fn get_batch(&self, blake3_key: &str) -> EngineResult<Option<ExtractedVendorBatch>> {
        let guard = self
            .conn
            .lock()
            .map_err(|_| EngineError::CheckpointDb("Checkpoint mutex poisoned".to_owned()))?;

        let raw_json: Option<String> = guard
            .query_row(
                "SELECT payload_json FROM batch_checkpoints WHERE blake3_key = ?1",
                params![blake3_key],
                |row| row.get(0),
            )
            .optional()
            .map_err(|err| EngineError::CheckpointDb(err.to_string()))?;

        match raw_json {
            Some(json_str) => {
                let parsed =
                    serde_json::from_str::<ExtractedVendorBatch>(&json_str).map_err(|err| {
                        EngineError::CheckpointDb(format!("Corrupt checkpoint payload: {err}"))
                    })?;
                Ok(Some(parsed))
            }
            None => Ok(None),
        }
    }

    /// Atomically saves a completed `ExtractedVendorBatch` into the `WAL` store.
    ///
    /// # Errors
    /// Returns [`EngineError::CheckpointDb`] if serialization or SQLite write fails.
    pub fn save_batch(
        &self,
        blake3_key: &str,
        session_id: &str,
        vendor_name: &str,
        batch: &ExtractedVendorBatch,
    ) -> EngineResult<()> {
        let json_str = serde_json::to_string(batch)
            .map_err(|err| EngineError::CheckpointDb(err.to_string()))?;
        let now_epoch = Utc::now().timestamp();

        let guard = self
            .conn
            .lock()
            .map_err(|_| EngineError::CheckpointDb("Checkpoint mutex poisoned".to_owned()))?;

        guard
            .execute(
                "INSERT INTO batch_checkpoints (blake3_key, session_id, vendor_name, payload_json, created_at_epoch)
                 VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(blake3_key) DO UPDATE SET
                   payload_json = excluded.payload_json,
                   created_at_epoch = excluded.created_at_epoch",
                params![blake3_key, session_id, vendor_name, json_str, now_epoch],
            )
            .map_err(|err| EngineError::CheckpointDb(err.to_string()))?;

        Ok(())
    }
}
