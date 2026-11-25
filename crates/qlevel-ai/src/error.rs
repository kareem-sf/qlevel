//! Structured error types for `qlevel-ai` with zero credential leakage.

use qlevel_domain::ProviderKind;

/// Canonical error returned by direct cloud AI providers and OAuth flows.
#[derive(Debug, thiserror::Error)]
pub enum AiError {
    #[error("Authentication required or invalid for provider {provider:?}: {detail}")]
    Unauthorized {
        provider: ProviderKind,
        detail: String,
    },

    #[error("OpenAI ChatGPT subscription usage cap reached; manage quota at {manage_url}")]
    OpenAiUsageLimitExceeded {
        manage_url: String,
        reset_seconds: Option<u64>,
    },

    #[error("Rate limit exceeded on {provider:?}: {detail}")]
    RateLimited {
        provider: ProviderKind,
        detail: String,
        retry_after_seconds: Option<u64>,
    },

    #[error("OAuth PKCE sign-in failed: {0}")]
    OAuthFailure(String),

    #[error("Network or corporate proxy TLS failure talking to {provider:?}: {detail}")]
    Network {
        provider: ProviderKind,
        detail: String,
    },

    #[error("Provider {provider:?} returned HTTP {status}: {body_excerpt}")]
    UpstreamHttp {
        provider: ProviderKind,
        status: u16,
        body_excerpt: String,
    },

    #[error("Structured JSON extraction output from {provider:?} was malformed: {detail}")]
    MalformedStructuredOutput {
        provider: ProviderKind,
        detail: String,
    },

    #[error("Operation cancelled by user")]
    Cancelled,
}

/// Convenience result alias for `qlevel-ai`.
pub type AiResult<T> = Result<T, AiError>;
