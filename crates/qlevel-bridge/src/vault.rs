//! Envelope-encrypted local credential vault (`32-byte` OS Keyring DEK + `AES-256-GCM`).
//!
//! Bypasses the Windows Credential Manager (`CREDENTIALW`) `2560-byte` secret ceiling by storing
//! only a random `32-byte` Data Encryption Key (`DEK`) in `keyring` and encrypting unbounded
//! API keys and `OpenAI Sign in with ChatGPT` OAuth bundles locally with `AES-256-GCM`.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::{Aes256Gcm, Nonce};
use qlevel_ai::siwc::CHATGPT_USAGE_PORTAL_URL;
use qlevel_ai::{OpenAiSiwcSession, ProviderAuthCredential};
use qlevel_domain::ProviderKind;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

use crate::dto::{ConnectedAuthKindDto, ProviderAuthStatusDto};
use crate::error::{BridgeError, BridgeResult};

const DEK_BYTES_LEN: usize = 32;
const NONCE_BYTES_LEN: usize = 12;
const MAGIC_HEADER: &[u8; 8] = b"QLVL_V1\0";
const VAULT_AAD: &[u8] = b"qlevel.vault.v1";

const KEYRING_SERVICE: &str = "com.kareemsafwat.qlevel.vault";
const KEYRING_ACCOUNT: &str = "master-dek-v1";

/// Pluggable 32-byte master key store (`OsKeyringMasterBackend` in production, `InMemoryMasterKeyBackend` in CI).
pub trait MasterKeyBackend: Send + Sync {
    /// Reads the 32-byte master DEK if present.
    fn load_dek(&self) -> BridgeResult<Option<Zeroizing<[u8; DEK_BYTES_LEN]>>>;
    /// Writes the 32-byte master DEK to the OS credential store.
    fn store_dek(&self, dek: &Zeroizing<[u8; DEK_BYTES_LEN]>) -> BridgeResult<()>;
}

/// Production backend storing the 32-byte DEK in Windows Credential Manager, macOS Keychain, or Secret Service.
#[derive(Debug, Default)]
pub struct OsKeyringMasterBackend;

impl MasterKeyBackend for OsKeyringMasterBackend {
    fn load_dek(&self) -> BridgeResult<Option<Zeroizing<[u8; DEK_BYTES_LEN]>>> {
        let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_ACCOUNT)
            .map_err(|e| BridgeError::VaultUnavailable(e.to_string()))?;
        match entry.get_secret() {
            Ok(raw) => {
                let zeroized = Zeroizing::new(raw);
                if zeroized.len() != DEK_BYTES_LEN {
                    return Err(BridgeError::VaultUnavailable(
                        "Invalid master DEK byte length in OS keyring".to_owned(),
                    ));
                }
                let mut key = Zeroizing::new([0_u8; DEK_BYTES_LEN]);
                key.copy_from_slice(&zeroized);
                Ok(Some(key))
            }
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(BridgeError::VaultUnavailable(e.to_string())),
        }
    }

    fn store_dek(&self, dek: &Zeroizing<[u8; DEK_BYTES_LEN]>) -> BridgeResult<()> {
        let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_ACCOUNT)
            .map_err(|e| BridgeError::VaultUnavailable(e.to_string()))?;
        entry
            .set_secret(dek.as_slice())
            .map_err(|e| BridgeError::VaultUnavailable(e.to_string()))
    }
}

/// Deterministic in-memory DEK backend for headless CI tests without a live OS keyring daemon.
#[derive(Default)]
pub struct InMemoryMasterKeyBackend {
    slot: Mutex<Option<[u8; DEK_BYTES_LEN]>>,
}

impl MasterKeyBackend for InMemoryMasterKeyBackend {
    fn load_dek(&self) -> BridgeResult<Option<Zeroizing<[u8; DEK_BYTES_LEN]>>> {
        let guard = self
            .slot
            .lock()
            .map_err(|_| BridgeError::Internal("In-memory keyring mutex poisoned".to_owned()))?;
        Ok(guard.map(Zeroizing::new))
    }

    fn store_dek(&self, dek: &Zeroizing<[u8; DEK_BYTES_LEN]>) -> BridgeResult<()> {
        let mut guard = self
            .slot
            .lock()
            .map_err(|_| BridgeError::Internal("In-memory keyring mutex poisoned".to_owned()))?;
        *guard = Some(**dek);
        Ok(())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
struct VaultPlaintextStore {
    agent_host_urn: String,
    openai_api_key: Option<String>,
    openai_siwc: Option<OpenAiSiwcSession>,
    gemini_api_key: Option<String>,
    gemini_privacy_ack: bool,
    claude_api_key: Option<String>,
}

/// Thread-safe envelope-encrypted local vault.
pub struct EnvelopeVault {
    vault_file_path: PathBuf,
    backend: Arc<dyn MasterKeyBackend>,
    io_lock: Mutex<()>,
}

impl EnvelopeVault {
    /// Creates an `EnvelopeVault` storing ciphertext at `vault_file_path`.
    #[must_use]
    pub fn new(vault_file_path: PathBuf, backend: Arc<dyn MasterKeyBackend>) -> Self {
        Self {
            vault_file_path,
            backend,
            io_lock: Mutex::new(()),
        }
    }

    fn get_or_init_dek(&self) -> BridgeResult<Zeroizing<[u8; DEK_BYTES_LEN]>> {
        if let Some(existing) = self.backend.load_dek()? {
            return Ok(existing);
        }
        let mut fresh = Zeroizing::new([0_u8; DEK_BYTES_LEN]);
        getrandom::getrandom(fresh.as_mut_slice())
            .map_err(|e| BridgeError::VaultUnavailable(e.to_string()))?;
        self.backend.store_dek(&fresh)?;
        Ok(fresh)
    }

    fn read_unlocked(&self) -> BridgeResult<VaultPlaintextStore> {
        if !self.vault_file_path.exists() {
            let mut empty_store = VaultPlaintextStore::default();
            empty_store.agent_host_urn = format!("urn:uuid:{}", Uuid::new_v4());
            return Ok(empty_store);
        }

        let raw = fs::read(&self.vault_file_path)
            .map_err(|e| BridgeError::VaultUnavailable(e.to_string()))?;
        if raw.len() < MAGIC_HEADER.len() + NONCE_BYTES_LEN + 16 || !raw.starts_with(MAGIC_HEADER) {
            return Err(BridgeError::VaultUnavailable(
                "Corrupted local vault header".to_owned(),
            ));
        }

        let dek = self.get_or_init_dek()?;
        let cipher = Aes256Gcm::new_from_slice(dek.as_slice())
            .map_err(|e| BridgeError::VaultUnavailable(e.to_string()))?;

        let nonce_start = MAGIC_HEADER.len();
        let ct_start = nonce_start + NONCE_BYTES_LEN;
        let nonce = Nonce::from_slice(&raw[nonce_start..ct_start]);
        let ciphertext = &raw[ct_start..];

        let plaintext = Zeroizing::new(
            cipher
                .decrypt(
                    nonce,
                    Payload {
                        msg: ciphertext,
                        aad: VAULT_AAD,
                    },
                )
                .map_err(|_| {
                    BridgeError::VaultUnavailable(
                        "Vault AES-256-GCM authentication failed".to_owned(),
                    )
                })?,
        );

        let mut parsed: VaultPlaintextStore = serde_json::from_slice(&plaintext)
            .map_err(|e| BridgeError::VaultUnavailable(e.to_string()))?;
        if parsed.agent_host_urn.trim().is_empty() {
            parsed.agent_host_urn = format!("urn:uuid:{}", Uuid::new_v4());
        }
        Ok(parsed)
    }

    fn write_unlocked(&self, store: &VaultPlaintextStore) -> BridgeResult<()> {
        let dek = self.get_or_init_dek()?;
        let cipher = Aes256Gcm::new_from_slice(dek.as_slice())
            .map_err(|e| BridgeError::VaultUnavailable(e.to_string()))?;

        let mut nonce_bytes = [0_u8; NONCE_BYTES_LEN];
        getrandom::getrandom(&mut nonce_bytes)
            .map_err(|e| BridgeError::VaultUnavailable(e.to_string()))?;
        let nonce = Nonce::from_slice(&nonce_bytes);

        let json_bytes = Zeroizing::new(
            serde_json::to_vec(store).map_err(|e| BridgeError::Internal(e.to_string()))?,
        );

        let ciphertext = cipher
            .encrypt(
                nonce,
                Payload {
                    msg: json_bytes.as_slice(),
                    aad: VAULT_AAD,
                },
            )
            .map_err(|_| BridgeError::Internal("Vault encryption failed".to_owned()))?;

        let mut envelope =
            Vec::with_capacity(MAGIC_HEADER.len() + NONCE_BYTES_LEN + ciphertext.len());
        envelope.extend_from_slice(MAGIC_HEADER);
        envelope.extend_from_slice(&nonce_bytes);
        envelope.extend_from_slice(&ciphertext);

        if let Some(parent) = self.vault_file_path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)
                    .map_err(|e| BridgeError::VaultUnavailable(e.to_string()))?;
            }
        }
        let tmp_path = self.vault_file_path.with_extension("enc.tmp");
        fs::write(&tmp_path, &envelope)
            .map_err(|e| BridgeError::VaultUnavailable(e.to_string()))?;
        fs::rename(&tmp_path, &self.vault_file_path)
            .map_err(|e| BridgeError::VaultUnavailable(e.to_string()))?;
        Ok(())
    }

    /// Returns the workstation's stable `urn:uuid:...` identifier and optional cached `oaiapp_...` client ID.
    pub fn get_siwc_registration_metadata(&self) -> BridgeResult<(String, Option<String>)> {
        let _guard = self
            .io_lock
            .lock()
            .map_err(|_| BridgeError::Internal("Vault mutex poisoned".to_owned()))?;
        let store = self.read_unlocked()?;
        self.write_unlocked(&store)?;
        let dyn_client = store
            .openai_siwc
            .as_ref()
            .map(|s| s.dynamic_client_id.clone());
        Ok((store.agent_host_urn.clone(), dyn_client))
    }

    /// Saves an `OpenAiSiwcSession` encrypted on disk.
    pub fn save_openai_siwc_session(
        &self,
        session: OpenAiSiwcSession,
    ) -> BridgeResult<ProviderAuthStatusDto> {
        let _guard = self
            .io_lock
            .lock()
            .map_err(|_| BridgeError::Internal("Vault mutex poisoned".to_owned()))?;
        let mut store = self.read_unlocked()?;
        store.openai_siwc = Some(session);
        self.write_unlocked(&store)?;
        Ok(Self::status_for(ProviderKind::OpenAi, &store))
    }

    /// Loads the stored `OpenAiSiwcSession` if present.
    pub fn load_openai_siwc_session(&self) -> BridgeResult<Option<OpenAiSiwcSession>> {
        let _guard = self
            .io_lock
            .lock()
            .map_err(|_| BridgeError::Internal("Vault mutex poisoned".to_owned()))?;
        let store = self.read_unlocked()?;
        Ok(store.openai_siwc.clone())
    }

    /// Saves a direct API key for OpenAI, Gemini, or Claude.
    pub fn save_api_key(
        &self,
        provider: ProviderKind,
        api_key: &str,
        acknowledge_gemini_privacy: bool,
    ) -> BridgeResult<ProviderAuthStatusDto> {
        let trimmed = api_key.trim();
        if trimmed.is_empty() {
            return Err(BridgeError::AuthExpired(
                "API key must not be empty".to_owned(),
            ));
        }
        if provider == ProviderKind::Gemini && !acknowledge_gemini_privacy {
            return Err(BridgeError::GeminiFreeTierUnacknowledged);
        }

        let _guard = self
            .io_lock
            .lock()
            .map_err(|_| BridgeError::Internal("Vault mutex poisoned".to_owned()))?;
        let mut store = self.read_unlocked()?;
        match provider {
            ProviderKind::OpenAi => {
                store.openai_api_key = Some(trimmed.to_owned());
                store.openai_siwc = None;
            }
            ProviderKind::Gemini => {
                store.gemini_api_key = Some(trimmed.to_owned());
                store.gemini_privacy_ack = acknowledge_gemini_privacy;
            }
            ProviderKind::Claude => {
                store.claude_api_key = Some(trimmed.to_owned());
            }
        }
        self.write_unlocked(&store)?;
        Ok(Self::status_for(provider, &store))
    }

    /// Disconnects and wipes stored credentials for `provider`.
    pub fn disconnect_provider(
        &self,
        provider: ProviderKind,
    ) -> BridgeResult<ProviderAuthStatusDto> {
        let _guard = self
            .io_lock
            .lock()
            .map_err(|_| BridgeError::Internal("Vault mutex poisoned".to_owned()))?;
        let mut store = self.read_unlocked()?;
        match provider {
            ProviderKind::OpenAi => {
                store.openai_api_key = None;
                store.openai_siwc = None;
            }
            ProviderKind::Gemini => {
                store.gemini_api_key = None;
                store.gemini_privacy_ack = false;
            }
            ProviderKind::Claude => {
                store.claude_api_key = None;
            }
        }
        self.write_unlocked(&store)?;
        Ok(Self::status_for(provider, &store))
    }

    /// Lists sanitized connection statuses across all 3 direct providers.
    pub fn list_statuses(&self) -> BridgeResult<Vec<ProviderAuthStatusDto>> {
        let _guard = self
            .io_lock
            .lock()
            .map_err(|_| BridgeError::Internal("Vault mutex poisoned".to_owned()))?;
        let store = self.read_unlocked()?;
        Ok(vec![
            Self::status_for(ProviderKind::OpenAi, &store),
            Self::status_for(ProviderKind::Gemini, &store),
            Self::status_for(ProviderKind::Claude, &store),
        ])
    }

    /// Resolves the active `ProviderAuthCredential` for `preferred` (or the first connected provider in `Auto`).
    pub fn resolve_credential(
        &self,
        preferred: Option<ProviderKind>,
    ) -> BridgeResult<(ProviderKind, ProviderAuthCredential)> {
        let _guard = self
            .io_lock
            .lock()
            .map_err(|_| BridgeError::Internal("Vault mutex poisoned".to_owned()))?;
        let store = self.read_unlocked()?;

        let candidates = match preferred {
            Some(p) => vec![p],
            None => vec![
                ProviderKind::OpenAi,
                ProviderKind::Claude,
                ProviderKind::Gemini,
            ],
        };

        for p in candidates {
            match p {
                ProviderKind::OpenAi => {
                    if let Some(siwc) = &store.openai_siwc {
                        return Ok((
                            ProviderKind::OpenAi,
                            ProviderAuthCredential::OpenAiSiwc {
                                access_token: siwc.access_token.clone(),
                                dynamic_client_id: siwc.dynamic_client_id.clone(),
                            },
                        ));
                    }
                    if let Some(k) = &store.openai_api_key {
                        return Ok((
                            ProviderKind::OpenAi,
                            ProviderAuthCredential::ApiKey { key: k.clone() },
                        ));
                    }
                }
                ProviderKind::Gemini => {
                    if let Some(k) = &store.gemini_api_key {
                        return Ok((
                            ProviderKind::Gemini,
                            ProviderAuthCredential::ApiKey { key: k.clone() },
                        ));
                    }
                }
                ProviderKind::Claude => {
                    if let Some(k) = &store.claude_api_key {
                        return Ok((
                            ProviderKind::Claude,
                            ProviderAuthCredential::ApiKey { key: k.clone() },
                        ));
                    }
                }
            }
        }

        Err(BridgeError::AuthExpired(
            "No AI provider account is connected. Click 'AI Connection' to sign in with ChatGPT or add an API key."
                .to_owned(),
        ))
    }

    /// Returns the path of the encrypted vault container for inspection in tests.
    #[must_use]
    pub fn vault_path(&self) -> &Path {
        &self.vault_file_path
    }

    fn status_for(provider: ProviderKind, store: &VaultPlaintextStore) -> ProviderAuthStatusDto {
        match provider {
            ProviderKind::OpenAi => {
                if let Some(siwc) = &store.openai_siwc {
                    ProviderAuthStatusDto {
                        provider,
                        connected: true,
                        auth_method: ConnectedAuthKindDto::ChatgptOauth,
                        account_label: siwc
                            .account_email
                            .clone()
                            .or_else(|| Some("ChatGPT Connected".to_owned())),
                        gemini_privacy_acknowledged: false,
                        expires_at_epoch_secs: Some(siwc.expires_at_epoch_secs),
                        manage_usage_url: Some(CHATGPT_USAGE_PORTAL_URL.to_owned()),
                    }
                } else if let Some(k) = &store.openai_api_key {
                    ProviderAuthStatusDto {
                        provider,
                        connected: true,
                        auth_method: ConnectedAuthKindDto::ApiKey,
                        account_label: Some(mask_key_label(k)),
                        gemini_privacy_acknowledged: false,
                        expires_at_epoch_secs: None,
                        manage_usage_url: None,
                    }
                } else {
                    Self::disconnected(provider)
                }
            }
            ProviderKind::Gemini => {
                if let Some(k) = &store.gemini_api_key {
                    ProviderAuthStatusDto {
                        provider,
                        connected: true,
                        auth_method: ConnectedAuthKindDto::ApiKey,
                        account_label: Some(mask_key_label(k)),
                        gemini_privacy_acknowledged: store.gemini_privacy_ack,
                        expires_at_epoch_secs: None,
                        manage_usage_url: None,
                    }
                } else {
                    Self::disconnected(provider)
                }
            }
            ProviderKind::Claude => {
                if let Some(k) = &store.claude_api_key {
                    ProviderAuthStatusDto {
                        provider,
                        connected: true,
                        auth_method: ConnectedAuthKindDto::ApiKey,
                        account_label: Some(mask_key_label(k)),
                        gemini_privacy_acknowledged: false,
                        expires_at_epoch_secs: None,
                        manage_usage_url: None,
                    }
                } else {
                    Self::disconnected(provider)
                }
            }
        }
    }

    fn disconnected(provider: ProviderKind) -> ProviderAuthStatusDto {
        ProviderAuthStatusDto {
            provider,
            connected: false,
            auth_method: ConnectedAuthKindDto::None,
            account_label: None,
            gemini_privacy_acknowledged: false,
            expires_at_epoch_secs: None,
            manage_usage_url: None,
        }
    }
}

fn mask_key_label(raw: &str) -> String {
    let chars: Vec<char> = raw.chars().collect();
    if chars.len() <= 8 {
        return "API Key (configured)".to_owned();
    }
    let prefix: String = chars.iter().take(4).collect();
    let suffix: String = chars.iter().skip(chars.len() - 4).collect();
    format!("{prefix}...{suffix}")
}
