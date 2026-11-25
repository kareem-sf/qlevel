//! Direct cloud Multimodal AI integration for `qlevel`.
//!
//! Strictly supports three direct enterprise-grade providers:
//! 1. **OpenAI**: `Sign in with ChatGPT` dynamic local OAuth PKCE (`SIWC`) and `OPENAI_API_KEY`
//! 2. **Google Gemini**: `GEMINI_API_KEY` (`v1beta` structured extraction)
//! 3. **Anthropic Claude**: `ANTHROPIC_API_KEY` (`2026-10-21` adaptive thinking)
//!
//! Model lists are never hardcoded; `model_filter` filters live `/models` responses to
//! expose flagship Multimodal (text + vision/PDF) models only.

#![forbid(unsafe_code)]

pub mod error;
pub mod model_filter;
pub mod providers;
pub mod schema;
pub mod siwc;

pub use error::{AiError, AiResult};
pub use model_filter::{filter_live_multimodal_models, RawProviderModel};
pub use providers::{
    build_http_client, build_provider_client, ClaudeClient, GeminiClient, OpenAiClient,
};
pub use schema::{
    ExtractedLineItem, ExtractedTrapDto, ExtractedVendorBatch, ExtractionBatchInput,
    ImageAttachment, OpenAiSiwcSession, ProviderAuthCredential, ProviderClient,
};
pub use siwc::{
    build_siwc_authorize_url, generate_pkce_pair, BindLoopbackOutcome, OpenAiSiwcCoordinator,
    PkcePair,
};
