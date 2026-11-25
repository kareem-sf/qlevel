//! Direct cloud provider HTTP clients (`OpenAI`, `Google Gemini`, `Anthropic Claude`).

pub mod claude;
pub mod gemini;
pub mod openai;

use std::time::Duration;

use qlevel_domain::ProviderKind;

use crate::error::{AiError, AiResult};
use crate::schema::{ProviderAuthCredential, ProviderClient};

pub use claude::ClaudeClient;
pub use gemini::GeminiClient;
pub use openai::OpenAiClient;

/// Builds a shared `reqwest::Client` using `rustls` + native OS root certificates
/// (`Windows CryptoAPI`, `macOS Security.framework`, `Linux SSL certs`) so corporate
/// proxies such as Zscaler and FortiGate work transparently.
///
/// # Errors
/// Returns [`AiError::Network`] if TLS initialization fails.
pub fn build_http_client() -> AiResult<reqwest::Client> {
    reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(20))
        .timeout(Duration::from_secs(180))
        .user_agent(concat!("qlevel/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|err| AiError::Network {
            provider: ProviderKind::OpenAi,
            detail: err.to_string(),
        })
}

/// Instantiates the matching direct cloud `ProviderClient`.
///
/// # Errors
/// Returns [`AiError::Unauthorized`] if the credential kind does not match the provider.
pub fn build_provider_client(
    provider: ProviderKind,
    credential: ProviderAuthCredential,
    http: reqwest::Client,
) -> AiResult<Box<dyn ProviderClient>> {
    match (provider, credential) {
        (ProviderKind::OpenAi, cred) => Ok(Box::new(OpenAiClient::new(http, cred))),
        (ProviderKind::Gemini, ProviderAuthCredential::ApiKey { ref key }) => {
            Ok(Box::new(GeminiClient::new(http, key.clone())))
        }
        (ProviderKind::Claude, ProviderAuthCredential::ApiKey { ref key }) => {
            Ok(Box::new(ClaudeClient::new(http, key.clone())))
        }
        (other, _) => Err(AiError::Unauthorized {
            provider: other,
            detail: "Sign in with ChatGPT OAuth is only supported for OpenAI".to_owned(),
        }),
    }
}

pub(crate) fn clean_json_fence(raw: &str) -> &str {
    let trimmed = raw.trim();
    if let Some(rest) = trimmed.strip_prefix("```json") {
        return rest.trim().trim_end_matches("```").trim();
    }
    if let Some(rest) = trimmed.strip_prefix("```") {
        return rest.trim().trim_end_matches("```").trim();
    }
    trimmed
}
