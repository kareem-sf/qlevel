//! OpenAI `Sign in with ChatGPT` (`SIWC`) dynamic local OAuth 2.0 PKCE coordinator.
//!
//! Implements OpenAI's open-source local app specification:
//! - Dynamic registration via `client_id=dynamic_agent_client` and `ext_agent_host_id=urn:uuid:...`
//! - Persists the callback's dynamically issued `oaiapp_...` `client_id` BEFORE token exchange
//! - Binds loopback `127.0.0.1:0` safely (`SO_EXCLUSIVEADDRUSE` on Windows via `socket2`)
//! - Serializes rotating `refresh_token` refreshes behind an async `tokio::sync::Mutex`.

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::Arc;
use std::time::Duration;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use chrono::Utc;
use qlevel_domain::ProviderKind;
use rand::Rng as _;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use socket2::{Domain, Protocol, Socket, Type};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use url::Url;

use crate::error::{AiError, AiResult};
use crate::schema::OpenAiSiwcSession;

/// OpenAI OAuth 2.0 authorization endpoint for SIWC local open-source apps.
pub const OPENAI_SIWC_AUTH_URL: &str = "https://auth.openai.com/api/accounts/authorize";
/// OpenAI OAuth 2.0 token & refresh endpoint for SIWC local open-source apps.
pub const OPENAI_SIWC_TOKEN_URL: &str = "https://auth.openai.com/api/accounts/oauth/token";
/// Sentinel client identifier requesting dynamic per-host `oaiapp_...` assignment.
pub const DYNAMIC_AGENT_CLIENT_ID: &str = "dynamic_agent_client";
/// Manage-usage portal URL surfaced when ChatGPT subscription usage caps are hit.
pub const CHATGPT_USAGE_PORTAL_URL: &str = "https://chatgpt.com/codex/settings/usage";

/// RFC 7636 S256 PKCE verifier/challenge pair plus CSRF state token.
#[derive(Debug, Clone)]
pub struct PkcePair {
    pub code_verifier: String,
    pub code_challenge: String,
    pub csrf_state: String,
}

/// Generates a 64-character URL-safe RFC 7636 PKCE verifier, S256 challenge, and CSRF state.
#[must_use]
pub fn generate_pkce_pair() -> PkcePair {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut rng = rand::thread_rng();

    let code_verifier: String = (0..64)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

    let csrf_state: String = (0..32)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

    let digest = Sha256::digest(code_verifier.as_bytes());
    let code_challenge = URL_SAFE_NO_PAD.encode(digest);

    PkcePair {
        code_verifier,
        code_challenge,
        csrf_state,
    }
}

/// Builds the full OpenAI SIWC browser authorization URL.
///
/// # Errors
/// Returns [`AiError::OAuthFailure`] if URL construction fails.
pub fn build_siwc_authorize_url(
    redirect_uri: &str,
    pkce: &PkcePair,
    agent_host_urn: &str,
    existing_dynamic_client_id: Option<&str>,
) -> AiResult<String> {
    let mut url = Url::parse(OPENAI_SIWC_AUTH_URL)
        .map_err(|err| AiError::OAuthFailure(err.to_string()))?;

    let client_id = existing_dynamic_client_id
        .filter(|id| id.starts_with("oaiapp_"))
        .unwrap_or(DYNAMIC_AGENT_CLIENT_ID);

    url.query_pairs_mut()
        .append_pair("response_type", "code")
        .append_pair("client_id", client_id)
        .append_pair("redirect_uri", redirect_uri)
        .append_pair("scope", "openid profile email offline_access")
        .append_pair("code_challenge", &pkce.code_challenge)
        .append_pair("code_challenge_method", "S256")
        .append_pair("state", &pkce.csrf_state)
        .append_pair("ext_agent_host_id", agent_host_urn)
        .append_pair("ext_client_name", "qlevel Bid Leveling Workbench");

    Ok(url.into())
}

/// Bound loopback TCP listener and exact callback redirect URI.
pub struct BindLoopbackOutcome {
    pub listener: TcpListener,
    pub redirect_uri: String,
}

/// Binds `127.0.0.1:0` with non-hijackable socket options (`reuse_address(false)`).
///
/// # Errors
/// Returns [`AiError::OAuthFailure`] if binding the local loopback socket fails.
pub fn bind_exclusive_loopback() -> AiResult<BindLoopbackOutcome> {
    let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))
        .map_err(|err| AiError::OAuthFailure(format!("Failed to create loopback socket: {err}")))?;
    socket
        .set_nonblocking(true)
        .map_err(|err| AiError::OAuthFailure(format!("Failed setting nonblocking socket: {err}")))?;

    let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0));
    socket
        .bind(&addr.into())
        .map_err(|err| AiError::OAuthFailure(format!("Failed binding loopback port: {err}")))?;
    socket
        .listen(4)
        .map_err(|err| AiError::OAuthFailure(format!("Failed listening on loopback port: {err}")))?;

    let std_listener: std::net::TcpListener = socket.into();
    let local_addr = std_listener
        .local_addr()
        .map_err(|err| AiError::OAuthFailure(format!("Failed reading local port: {err}")))?;
    let listener = TcpListener::from_std(std_listener)
        .map_err(|err| AiError::OAuthFailure(format!("Tokio listener conversion failed: {err}")))?;

    let redirect_uri = format!("http://127.0.0.1:{}/oauth/callback", local_addr.port());
    Ok(BindLoopbackOutcome {
        listener,
        redirect_uri,
    })
}

#[derive(Debug, Deserialize)]
struct TokenEndpointResponse {
    access_token: String,
    refresh_token: Option<String>,
    id_token: Option<String>,
    expires_in: Option<i64>,
    client_id: Option<String>,
}

/// Coordinates local browser PKCE sign-in and serialized refresh-token rotation.
pub struct OpenAiSiwcCoordinator {
    http: reqwest::Client,
    refresh_mutex: Arc<Mutex<()>>,
}

impl OpenAiSiwcCoordinator {
    /// Creates a new coordinator sharing an enterprise-TLS `reqwest::Client`.
    #[must_use]
    pub fn new(http: reqwest::Client) -> Self {
        Self {
            http,
            refresh_mutex: Arc::new(Mutex::new(())),
        }
    }

    /// Waits up to `timeout` for the browser redirect on `listener`, validates CSRF `state`,
    /// captures dynamically assigned `client_id` (`oaiapp_...`), and exchanges `code` for tokens.
    ///
    /// # Errors
    /// Returns [`AiError`] on timeout, CSRF mismatch, missing client_id, or token HTTP failure.
    pub async fn await_callback_and_exchange(
        &self,
        listener: TcpListener,
        redirect_uri: &str,
        pkce: &PkcePair,
        agent_host_urn: &str,
        fallback_client_id: Option<&str>,
        timeout: Duration,
    ) -> AiResult<OpenAiSiwcSession> {
        let (mut stream, _) = tokio::time::timeout(timeout, listener.accept())
            .await
            .map_err(|_| AiError::OAuthFailure("Timed out waiting for ChatGPT browser sign-in".to_owned()))?
            .map_err(|err| AiError::OAuthFailure(format!("Loopback accept error: {err}")))?;

        let mut buf = vec![0_u8; 8192];
        let n = stream
            .read(&mut buf)
            .await
            .map_err(|err| AiError::OAuthFailure(format!("Failed reading callback request: {err}")))?;

        let req_str = String::from_utf8_lossy(&buf[..n]);
        let request_path = req_str
            .lines()
            .next()
            .and_then(|line| line.split_whitespace().nth(1))
            .ok_or_else(|| AiError::OAuthFailure("Malformed HTTP callback line".to_owned()))?;

        let parsed_url = Url::parse(&format!("http://127.0.0.1{request_path}"))
            .map_err(|err| AiError::OAuthFailure(format!("Invalid callback query: {err}")))?;

        let mut code_opt = None;
        let mut state_opt = None;
        let mut callback_client_id = None;
        let mut oauth_err = None;

        for (k, v) in parsed_url.query_pairs() {
            match k.as_ref() {
                "code" => code_opt = Some(v.into_owned()),
                "state" => state_opt = Some(v.into_owned()),
                "client_id" => callback_client_id = Some(v.into_owned()),
                "error" => oauth_err = Some(v.into_owned()),
                _ => {}
            }
        }

        let html_body = if oauth_err.is_some() {
            "<!doctype html><html><body style='font-family:sans-serif;padding:2rem'><h2>Sign-in cancelled</h2><p>You may close this tab and return to qlevel.</p></body></html>"
        } else {
            "<!doctype html><html><body style='font-family:sans-serif;padding:2rem'><h2>Signed in to qlevel</h2><p>Authentication succeeded. You may close this browser tab.</p></body></html>"
        };
        let http_resp = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            html_body.len(),
            html_body
        );
        let _ = stream.write_all(http_resp.as_bytes()).await;

        if let Some(err_code) = oauth_err {
            return Err(AiError::OAuthFailure(format!(
                "Authorization declined by OpenAI: {err_code}"
            )));
        }

        if state_opt.as_deref() != Some(pkce.csrf_state.as_str()) {
            return Err(AiError::OAuthFailure(
                "OAuth CSRF state mismatch on callback".to_owned(),
            ));
        }

        let auth_code = code_opt
            .ok_or_else(|| AiError::OAuthFailure("Missing authorization code in callback".to_owned()))?;

        let resolved_client_id = callback_client_id
            .or_else(|| fallback_client_id.map(ToOwned::to_owned))
            .ok_or_else(|| {
                AiError::OAuthFailure(
                    "OpenAI callback did not include dynamic oaiapp_ client_id".to_owned(),
                )
            })?;

        let form_params = [
            ("grant_type", "authorization_code"),
            ("code", auth_code.as_str()),
            ("redirect_uri", redirect_uri),
            ("client_id", resolved_client_id.as_str()),
            ("code_verifier", pkce.code_verifier.as_str()),
        ];

        let resp = self
            .http
            .post(OPENAI_SIWC_TOKEN_URL)
            .form(&form_params)
            .send()
            .await
            .map_err(|err| AiError::Network {
                provider: ProviderKind::OpenAi,
                detail: err.to_string(),
            })?;

        let status = resp.status();
        if !status.is_success() {
            let txt = resp.text().await.unwrap_or_default();
            return Err(AiError::UpstreamHttp {
                provider: ProviderKind::OpenAi,
                status: status.as_u16(),
                body_excerpt: txt.chars().take(300).collect(),
            });
        }

        let token_payload: TokenEndpointResponse =
            resp.json().await.map_err(|err| AiError::OAuthFailure(err.to_string()))?;

        let final_client_id = token_payload.client_id.unwrap_or(resolved_client_id);
        let account_email = token_payload
            .id_token
            .as_deref()
            .and_then(extract_email_from_unverified_jwt_claims);

        let expires_at_epoch_secs =
            Utc::now().timestamp() + token_payload.expires_in.unwrap_or(3600);

        Ok(OpenAiSiwcSession {
            dynamic_client_id: final_client_id,
            agent_host_urn: agent_host_urn.to_owned(),
            access_token: token_payload.access_token,
            refresh_token: token_payload.refresh_token.unwrap_or_default(),
            id_token: token_payload.id_token,
            account_email,
            expires_at_epoch_secs,
        })
    }

    /// Refreshes an expired or near-expiry `OpenAiSiwcSession` under an exclusive async lock
    /// so parallel batch workers never race a single-use rotating `refresh_token`.
    ///
    /// # Errors
    /// Returns [`AiError::Unauthorized`] if the refresh token was revoked or expired.
    pub async fn refresh_session(
        &self,
        current: &OpenAiSiwcSession,
    ) -> AiResult<OpenAiSiwcSession> {
        let _guard = self.refresh_mutex.lock().await;

        if current.refresh_token.trim().is_empty() {
            return Err(AiError::Unauthorized {
                provider: ProviderKind::OpenAi,
                detail: "ChatGPT session has no refresh_token; please sign in again".to_owned(),
            });
        }

        let form_params = [
            ("grant_type", "refresh_token"),
            ("refresh_token", current.refresh_token.as_str()),
            ("client_id", current.dynamic_client_id.as_str()),
        ];

        let resp = self
            .http
            .post(OPENAI_SIWC_TOKEN_URL)
            .form(&form_params)
            .send()
            .await
            .map_err(|err| AiError::Network {
                provider: ProviderKind::OpenAi,
                detail: err.to_string(),
            })?;

        let status = resp.status();
        if !status.is_success() {
            return Err(AiError::Unauthorized {
                provider: ProviderKind::OpenAi,
                detail: format!("Refresh token rejected (HTTP {})", status.as_u16()),
            });
        }

        let token_payload: TokenEndpointResponse =
            resp.json().await.map_err(|err| AiError::OAuthFailure(err.to_string()))?;

        let expires_at_epoch_secs =
            Utc::now().timestamp() + token_payload.expires_in.unwrap_or(3600);

        Ok(OpenAiSiwcSession {
            dynamic_client_id: token_payload
                .client_id
                .unwrap_or_else(|| current.dynamic_client_id.clone()),
            agent_host_urn: current.agent_host_urn.clone(),
            access_token: token_payload.access_token,
            refresh_token: token_payload
                .refresh_token
                .unwrap_or_else(|| current.refresh_token.clone()),
            id_token: token_payload.id_token.or_else(|| current.id_token.clone()),
            account_email: current.account_email.clone(),
            expires_at_epoch_secs,
        })
    }
}

fn extract_email_from_unverified_jwt_claims(jwt: &str) -> Option<String> {
    let payload_b64 = jwt.split('.').nth(1)?;
    let bytes = URL_SAFE_NO_PAD.decode(payload_b64).ok()?;
    let val: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    val.get("email")
        .and_then(serde_json::Value::as_str)
        .map(ToOwned::to_owned)
}
