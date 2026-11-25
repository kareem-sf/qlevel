//! Anthropic Claude `2026-10-21` direct Multimodal client (`claude-opus-5-5`, `claude-sonnet-5-5`).
//!
//! Complies strictly with Anthropic `2026-10-21` migration rules:
//! - Uses direct API key (`x-api-key` + `Authorization: Bearer`)
//! - NEVER sends removed `temperature`, `top_p`, or `top_k` fields (which return HTTP 400)
//! - Uses `thinking: {"type": "adaptive"}` on Medium/High/Auto reasoning requests.

use async_trait::async_trait;
use qlevel_domain::{EffortLevel, ModelInfo, ProviderKind};
use serde_json::{json, Value};
use zeroize::Zeroizing;

use crate::error::{AiError, AiResult};
use crate::model_filter::{filter_live_multimodal_models, RawProviderModel};
use crate::providers::clean_json_fence;
use crate::schema::{
    build_extraction_instructions, ExtractedVendorBatch, ExtractionBatchInput, ProviderClient,
};

const CLAUDE_MODELS_URL: &str = "https://api.anthropic.com/v1/models";
const CLAUDE_MESSAGES_URL: &str = "https://api.anthropic.com/v1/messages";
/// Official Anthropic API version header removing deprecated sampling parameters.
pub const ANTHROPIC_API_VERSION: &str = "2026-10-21";

/// Direct Anthropic Claude API client.
pub struct ClaudeClient {
    http: reqwest::Client,
    api_key: Zeroizing<String>,
}

impl ClaudeClient {
    #[must_use]
    pub fn new(http: reqwest::Client, api_key: String) -> Self {
        Self {
            http,
            api_key: Zeroizing::new(api_key),
        }
    }
}

/// Parses `GET https://api.anthropic.com/v1/models` response.
pub fn parse_claude_models_response(body: &str) -> AiResult<Vec<RawProviderModel>> {
    let root: Value = serde_json::from_str(body).map_err(|err| AiError::MalformedStructuredOutput {
        provider: ProviderKind::Claude,
        detail: err.to_string(),
    })?;

    let arr = root
        .get("data")
        .and_then(Value::as_array)
        .ok_or_else(|| AiError::MalformedStructuredOutput {
            provider: ProviderKind::Claude,
            detail: "Missing `data` array in Anthropic /v1/models".to_owned(),
        })?;

    let mut out = Vec::with_capacity(arr.len());
    for item in arr {
        let Some(id) = item.get("id").and_then(Value::as_str) else {
            continue;
        };
        let display_name = item
            .get("display_name")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned);
        out.push(RawProviderModel {
            id: id.to_owned(),
            display_name,
            supports_vision_hint: Some(true),
            supports_generation_hint: Some(true),
            is_deprecated: false,
        });
    }
    Ok(out)
}

/// Builds a `2026-10-21` compliant `/v1/messages` request payload (zero `temperature`/`top_p`/`top_k`).
#[must_use]
pub fn build_claude_messages_payload(
    model_id: &str,
    effort: EffortLevel,
    batch: &ExtractionBatchInput,
) -> Value {
    let sys_text = build_extraction_instructions(&batch.custom_rules, &batch.approved_vendors);

    let mut content_blocks = Vec::with_capacity(batch.images.len() + 1);
    for img in &batch.images {
        content_blocks.push(json!({
            "type": "image",
            "source": {
                "type": "base64",
                "media_type": img.mime_type,
                "data": img.base64_data
            }
        }));
    }

    content_blocks.push(json!({
        "type": "text",
        "text": format!(
            "VENDOR NAME: {}\n\nMASTER BOQ BASELINE:\n{}\n\nVENDOR QUOTATION CONTENT:\n{}\n\nReturn ONLY the raw JSON object.",
            batch.vendor_name,
            batch.master_boq_context_json,
            batch.vendor_raw_text_or_rows
        )
    }));

    let mut payload = json!({
        "model": model_id,
        "max_tokens": 16384,
        "system": sys_text,
        "messages": [
            {
                "role": "user",
                "content": content_blocks
            }
        ]
    });

    if !matches!(effort, EffortLevel::Low) {
        payload["thinking"] = json!({ "type": "adaptive" });
    }

    payload
}

/// Extracts assistant `type == "text"` blocks (ignoring `"thinking"` blocks) from Claude response.
pub fn extract_claude_text_blocks(body: &str) -> AiResult<String> {
    let root: Value = serde_json::from_str(body).map_err(|err| AiError::MalformedStructuredOutput {
        provider: ProviderKind::Claude,
        detail: err.to_string(),
    })?;

    let content = root
        .get("content")
        .and_then(Value::as_array)
        .ok_or_else(|| AiError::MalformedStructuredOutput {
            provider: ProviderKind::Claude,
            detail: "Missing `content` array in Claude /v1/messages response".to_owned(),
        })?;

    let mut combined = String::new();
    for block in content {
        if block.get("type").and_then(Value::as_str) == Some("text") {
            if let Some(txt) = block.get("text").and_then(Value::as_str) {
                combined.push_str(txt);
            }
        }
    }

    if combined.trim().is_empty() {
        return Err(AiError::MalformedStructuredOutput {
            provider: ProviderKind::Claude,
            detail: "No `text` block returned by Claude".to_owned(),
        });
    }
    Ok(combined)
}

#[async_trait]
impl ProviderClient for ClaudeClient {
    fn provider_kind(&self) -> ProviderKind {
        ProviderKind::Claude
    }

    async fn list_multimodal_models(&self) -> AiResult<Vec<ModelInfo>> {
        let resp = self
            .http
            .get(CLAUDE_MODELS_URL)
            .header("anthropic-version", ANTHROPIC_API_VERSION)
            .header("x-api-key", self.api_key.as_str())
            .bearer_auth(self.api_key.as_str())
            .send()
            .await
            .map_err(|err| AiError::Network {
                provider: ProviderKind::Claude,
                detail: err.to_string(),
            })?;

        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        if status.as_u16() == 401 || status.as_u16() == 403 {
            return Err(AiError::Unauthorized {
                provider: ProviderKind::Claude,
                detail: "Anthropic API key rejected".to_owned(),
            });
        }
        if !status.is_success() {
            return Err(AiError::UpstreamHttp {
                provider: ProviderKind::Claude,
                status: status.as_u16(),
                body_excerpt: body.chars().take(300).collect(),
            });
        }

        let raw_models = parse_claude_models_response(&body)?;
        Ok(filter_live_multimodal_models(
            ProviderKind::Claude,
            raw_models,
        ))
    }

    async fn extract_vendor_batch(
        &self,
        model_id: &str,
        effort: EffortLevel,
        batch: &ExtractionBatchInput,
    ) -> AiResult<ExtractedVendorBatch> {
        let payload = build_claude_messages_payload(model_id, effort, batch);
        let resp = self
            .http
            .post(CLAUDE_MESSAGES_URL)
            .header("anthropic-version", ANTHROPIC_API_VERSION)
            .header("x-api-key", self.api_key.as_str())
            .bearer_auth(self.api_key.as_str())
            .json(&payload)
            .send()
            .await
            .map_err(|err| AiError::Network {
                provider: ProviderKind::Claude,
                detail: err.to_string(),
            })?;

        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        if status.as_u16() == 401 || status.as_u16() == 403 {
            return Err(AiError::Unauthorized {
                provider: ProviderKind::Claude,
                detail: "Anthropic API key unauthorized".to_owned(),
            });
        }
        if status.as_u16() == 429 {
            return Err(AiError::RateLimited {
                provider: ProviderKind::Claude,
                detail: body.chars().take(240).collect(),
                retry_after_seconds: None,
            });
        }
        if !status.is_success() {
            return Err(AiError::UpstreamHttp {
                provider: ProviderKind::Claude,
                status: status.as_u16(),
                body_excerpt: body.chars().take(300).collect(),
            });
        }

        let text = extract_claude_text_blocks(&body)?;
        serde_json::from_str::<ExtractedVendorBatch>(clean_json_fence(&text)).map_err(|err| {
            AiError::MalformedStructuredOutput {
                provider: ProviderKind::Claude,
                detail: err.to_string(),
            }
        })
    }
}
