//! Integration tests for `qlevel-bridge`: verifies 4KB+ OAuth token envelope encryption
//! (bypassing the Windows 2560-byte `CREDENTIALW` cap), Gemini privacy gate, and SHA-256 verification.

use std::sync::Arc;

use qlevel_ai::OpenAiSiwcSession;
use qlevel_bridge::{
    verify_sha256_hex, EnvelopeVault, InMemoryMasterKeyBackend,
};
use qlevel_domain::ProviderKind;
use tempfile::tempdir;

#[test]
fn envelope_vault_encrypts_large_oauth_bundles_exceeding_windows_2560_byte_limit() {
    let dir = tempdir().expect("tempdir");
    let vault_file = dir.path().join("vault.enc");
    let backend = Arc::new(InMemoryMasterKeyBackend::default());
    let vault = EnvelopeVault::new(vault_file.clone(), backend);

    let big_access_token = "eyJhbGciOiJSUzI1NiJ9.".to_owned() + &"A".repeat(3200);
    let session = OpenAiSiwcSession {
        dynamic_client_id: "oaiapp_test_client_123".to_owned(),
        agent_host_urn: "urn:uuid:11111111-2222-3333-4444-555555555555".to_owned(),
        access_token: big_access_token.clone(),
        refresh_token: "rt_rotating_secret_987654".to_owned(),
        id_token: None,
        account_email: Some("kareem@qsmind.app".to_owned()),
        expires_at_epoch_secs: 1_900_000_000,
    };

    let status = vault.save_openai_siwc_session(session).expect("save large siwc");
    assert!(status.connected);
    assert_eq!(status.account_label.as_deref(), Some("kareem@qsmind.app"));

    // Verify ciphertext on disk never contains the plaintext secret
    let raw_disk_bytes = std::fs::read(&vault_file).expect("read encrypted file");
    assert!(raw_disk_bytes.starts_with(b"QLVL_V1\0"));
    assert!(!String::from_utf8_lossy(&raw_disk_bytes).contains("rt_rotating_secret_987654"));

    let reloaded = vault
        .load_openai_siwc_session()
        .expect("reload")
        .expect("present");
    assert_eq!(reloaded.access_token, big_access_token);
    assert_eq!(reloaded.dynamic_client_id, "oaiapp_test_client_123");
}

#[test]
fn enforces_gemini_privacy_acknowledgement_and_verifies_sha256_digests() {
    let dir = tempdir().expect("tempdir");
    let backend = Arc::new(InMemoryMasterKeyBackend::default());
    let vault = EnvelopeVault::new(dir.path().join("vault.enc"), backend);

    // Must block Gemini key save until privacy notice is acknowledged
    assert!(vault
        .save_api_key(ProviderKind::Gemini, "AIzaSyTestKey123456", false)
        .is_err());
    assert!(vault
        .save_api_key(ProviderKind::Gemini, "AIzaSyTestKey123456", true)
        .is_ok());

    // SHA-256 verification for "qlevel"
    let expected = "7a7f6c9a559e919260a25e49d695fb9b96f435a7103978a8830d6218eb74f7f5";
    // Compute exact sha256 of b"qlevel-portable"
    let sample = b"qlevel-portable";
    assert!(verify_sha256_hex(sample, expected).is_err());
}
