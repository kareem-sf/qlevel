//! Thread-safe runtime state managing the `EnvelopeVault`, `CheckpointStore`, active
//! `CancellationToken` handles, interactive ambiguity channels, and completed `LevelingResult`s.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use qlevel_ai::{build_http_client, OpenAiSiwcCoordinator};
use qlevel_domain::LevelingResult;
use qlevel_engine::CheckpointStore;
use tokio::sync::{oneshot, Mutex};
use tokio_util::sync::CancellationToken;

use crate::dto::{AmbiguityDecisionInput, RenderedPageImageDto};
use crate::error::{BridgeError, BridgeResult};
use crate::vault::{EnvelopeVault, MasterKeyBackend, OsKeyringMasterBackend};

/// Shared application state managed inside Tauri (`app.manage(BridgeState)`).
pub struct BridgeState {
    pub app_data_dir: PathBuf,
    pub vault: Arc<EnvelopeVault>,
    pub checkpoints: Arc<CheckpointStore>,
    pub http_client: reqwest::Client,
    pub siwc_coordinator: Arc<OpenAiSiwcCoordinator>,
    pub active_cancellations: Mutex<HashMap<String, CancellationToken>>,
    pub pending_ambiguity_tx: Mutex<HashMap<(String, String), oneshot::Sender<AmbiguityDecisionInput>>>,
    pub rendered_pdf_cache: Mutex<HashMap<String, Vec<RenderedPageImageDto>>>,
    pub completed_results: Mutex<HashMap<String, LevelingResult>>,
}

impl BridgeState {
    /// Initializes production `BridgeState` under the user's OS local data folder (`%LOCALAPPDATA%/qlevel`).
    ///
    /// # Errors
    /// Returns [`BridgeError`] if local storage or TLS initialization fails.
    pub fn initialize_default() -> BridgeResult<Self> {
        let base_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("qlevel");
        std::fs::create_dir_all(&base_dir)
            .map_err(|e| BridgeError::Internal(format!("Cannot create app data dir: {e}")))?;

        let backend: Arc<dyn MasterKeyBackend> = Arc::new(OsKeyringMasterBackend);
        Self::initialize_with_backend(base_dir, backend)
    }

    /// Initializes `BridgeState` at a custom directory and `MasterKeyBackend` (ideal for headless tests).
    ///
    /// # Errors
    /// Returns [`BridgeError`] if SQLite or HTTP client initialization fails.
    pub fn initialize_with_backend(
        app_data_dir: PathBuf,
        key_backend: Arc<dyn MasterKeyBackend>,
    ) -> BridgeResult<Self> {
        std::fs::create_dir_all(&app_data_dir)
            .map_err(|e| BridgeError::Internal(format!("Cannot create data dir: {e}")))?;

        let vault_path = app_data_dir.join("vault.enc");
        let ckpt_path = app_data_dir.join("checkpoints.db");

        let vault = Arc::new(EnvelopeVault::new(vault_path, key_backend));
        let checkpoints = Arc::new(CheckpointStore::open_file(&ckpt_path)?);
        let http_client = build_http_client()?;
        let siwc_coordinator = Arc::new(OpenAiSiwcCoordinator::new(http_client.clone()));

        Ok(Self {
            app_data_dir,
            vault,
            checkpoints,
            http_client,
            siwc_coordinator,
            active_cancellations: Mutex::new(HashMap::new()),
            pending_ambiguity_tx: Mutex::new(HashMap::new()),
            rendered_pdf_cache: Mutex::new(HashMap::new()),
            completed_results: Mutex::new(HashMap::new()),
        })
    }
}
