//! OpenAI direct client supporting both `Sign in with ChatGPT` (`SIWC`) and `OPENAI_API_KEY`.
//!
//! Complies with OpenAI SIWC `/v1/responses` requirements:
//! - Dual `/v1/models` parsing (`models[]` in SIWC vs `data[]` in API Key)
//! - `store: false`, `stream: true`, top-level `instructions` (no `role: "system"`, no `temperature`)
//! - `reasoning.effort` in `"low" | "medium" | "high"`
//! - Structured handling of `usage_limit_exceeded`.

use async_trait::async_trait;
use qlevel_domain::{EffortLevel, ModelInfo, ProviderKind};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::error::{AiError, AiResult};
use crate::model_filter::{filter_live_multimodal_models, RawProviderModel};
use crate::providers::clean_json_fence;
use crate::schema::{
    build_extraction_instructions, ExtractedVendorBatch, ExtractionBatchInput,
    ProviderAuthCredential, ProviderClient,
};
use crate::siwc::CHATGPT_USAGE_PORTAL_URL;

const OPENAI_MODELS_URL: &str = "https://api.openai.com/v1/models";
const OPENAI_RESPONSES_URL: &str = "https://api.openai.com/v1/responses";

/// Direct OpenAI client (`SIWC` OAuth or `API Key`).
pub struct OpenAiClient {
    http: reqwest::Client,
    credential: ProviderAuthCredential,
}

impl OpenAiClient {
    #[must_use]
    pub fn new(http: reqwest::Client, credential: ProviderAuthCredential) -> Self {
        Self { http, credential }
    }

    fn bearer_token(&self) -> &str {
        match &self.credential {
            ProviderAuthCredential::ApiKey { key } => key.as_str(),
            ProviderAuthCredential::OpenAiSiwc { access_token, .. } => access_token.as_str(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct OpenAiModelEntry {
    id: String,
    #[serde(default)]
    display_name: Option<String>,
}

/// Parses both `{"data": [...]}` (Platform API Key) and `{"models": [...]}` (SIWC OAuth).
pub fn parse_openai_models_response(body: &str) -> AiResult<Vec<RawProviderModel>> {
    let root: Value = serde_json::from_str(body).map_err(|err| AiError::MalformedStructuredOutput {
        provider: ProviderKind::OpenAi,
        detail: err.to_string(),
    })?;

    let arr = root
        .get("data")
        .and_then(Value::as_array)
        .or_else(|| root.get("models").and_then(Value::as_array))
        .ok_or_else(|| AiError::MalformedStructuredOutput {
            provider: ProviderKind::OpenAi,
            detail: "Expected `data` or `models` array in OpenAI /v1/models".to_owned(),
        })?;

    let mut out = Vec::with_capacity(arr.len());
    for item in arr {
        if let Ok(entry) = serde_json::from_value::<OpenAiModelEntry>(item.clone()) {
            out.push(RawProviderModel {
                id: entry.id,
                display_name: entry.display_name,
                supports_vision_hint: None,
                supports_generation_hint: Some(true),
                is_deprecated: false,
            });
        }
    }
    Ok(out)
}

/// Builds the `/v1/responses` JSON request body adhering strictly to SIWC & Platform rules.
#[must_use]
pub fn build_openai_responses_payload(
    model_id: &str,
    effort: EffortLevel,
    batch: &ExtractionBatchInput,
) -> Value {
    let instructions = build_extraction_instructions(&batch.custom_rules, &batch.approved_vendors);

    let mut content_parts = Vec::with_capacity(batch.images.len() + 1);
    content_parts.push(json!({
        "type": "input_text",
        "text": format!(
            "VENDOR NAME: {}\n\nMASTER BOQ BASELINE:\n{}\n\nVENDOR QUOTATION CONTENT:\n{}",
            batch.vendor_name,
            batch.master_boq_context_json,
            batch.vendor_raw_text_or_rows
        )
    }));

    for img in &batch.images {
        content_parts.push(json!({
            "type": "input_image",
            "image_url": format!("data:{};base64,{}", img.mime_type, img.base64_data)
        }));
    }

    let effort_str = match effort {
        EffortLevel::Low => Some("low"),
        EffortLevel::Medium | EffortLevel::Auto => Some("medium"),
        EffortLevel::High => Some("high"),
    };

    let mut payload = json!({
        "model": model_id,
        "store": false,
        "stream": true,
        "instructions": instructions,
        "input": [
            {
                "role": "user",
                "content": content_parts
            }
        ],
        "text": {
            "format": {
                "type": "json_object"
            }
        }
    });

    if let Some(eff) = effort_str {
        if !model_id.starts_with("gpt-4o") && !model_id.starts_with("gpt-4.1") {
            payload["reasoning"] = json!({ "effort": eff });
        }
    }

    payload
}

/// Extracts complete assistant text from an OpenAI `/v1/responses` SSE stream or JSON fallback.
pub fn collect_openai_sse_text(raw_body: &str) -> AiResult<String> {
    let trimmed = raw_body.trim();
    if trimmed.starts_with('{') {
        if let Ok(v) = serde_json::from_str::<Value>(trimmed) {
            if let Some(txt) = v.get("output_text").and_then(Value::as_str) {
                return Ok(txt.to_owned());
            }
        }
    }

    let mut delta_acc = String::new();
    let mut completed_text: Option<String> = None;

    for line in raw_body.lines() {
        let Some(data_str) = line.strip_prefix("data:") else {
            continue;
        };
        let data_trimmed = data_str.trim();
        if data_trimmed.is_empty() || data_trimmed == "[DONE]" {
            continue;
        }
        let Ok(ev) = serde_json::from_str::<Value>(data_trimmed) else {
            continue;
        };
        match ev.get("type").and_then(Value::as_str) {
            Some("response.output_text.delta") => {
                if let Some(delta) = ev.get("delta").and_then(Value::as_str) {
                    delta_acc.push_str(delta);
                }
            }
            Some("response.output_text.done") => {
                if let Some(full) = ev.get("text").and_then(Value::as_str) {
                    completed_text = Some(full.to_owned());
                }
            }
            _ => {}
        }
    }

    let final_str = completed_text.unwrap_or(delta_acc);
    if final_str.trim().is_empty() {
        return Err(AiError::MalformedStructuredOutput {
            provider: ProviderKind::OpenAi,
            detail: "Empty text stream returned by OpenAI /v1/responses".to_owned(),
        });
    }
    Ok(final_str)
}

#[async_trait]
impl ProviderClient for OpenAiClient {
    fn provider_kind(&self) -> ProviderKind {
        ProviderKind::OpenAi
    }

    async fn list_multimodal_models(&self) -> AiResult<Vec<ModelInfo>> {
        let resp = self
            .http
            .get(OPENAI_MODELS_URL)
            .bearer_auth(self.bearer_token())
            .send()
            .await
            .map_err(|err| AiError::Network {
                provider: ProviderKind::OpenAi,
                detail: err.to_string(),
            })?;

        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        if status.as_u16() == 401 || status.as_u16() == 403 {
            return Err(AiError::Unauthorized {
                provider: ProviderKind::OpenAi,
                detail: "OpenAI credential rejected".to_owned(),
            });
        }
        if !status.is_success() {
            return Err(AiError::UpstreamHttp {
                provider: ProviderKind::OpenAi,
                status: status.as_u16(),
                body_excerpt: body.chars().take(300).collect(),
            });
        }

        let raw_models = parse_openai_models_response(&body)?;
        Ok(filter_live_multimodal_models(
            ProviderKind::OpenAi,
            raw_models,
        ))
    }

    async fn extract_vendor_batch(
        &self,
        model_id: &str,
        effort: EffortLevel,
        batch: &ExtractionBatchInput,
    ) -> AiResult<ExtractedVendorBatch> {
        let payload = build_openai_responses_payload(model_id, effort, batch);
        let resp = self
            .http
            .post(OPENAI_RESPONSES_URL)
            .bearer_auth(self.bearer_token())
            .json(&payload)
            .send()
            .await
            .map_err(|err| AiError::Network {
                provider: ProviderKind::OpenAi,
                detail: err.to_string(),
            })?;

        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();

        if status.as_u16() == 401 {
            return Err(AiError::Unauthorized {
                provider: ProviderKind::OpenAi,
                detail: "OpenAI token expired or invalid".to_owned(),
            });
        }
        if status.as_u16() == 429 {
            if body.contains("usage_limit_exceeded") {
                return Err(AiError::OpenAiUsageLimitExceeded {
                    manage_url: CHATGPT_USAGE_PORTAL_URL.to_owned(),
                    reset_seconds: None,
                });
            }
            return Err(AiError::RateLimited {
                provider: ProviderKind::OpenAi,
                detail: body.chars().take(240).collect(),
                retry_after_seconds: None,
            });
        }
        if !status.is_success() {
            return Err(AiError::UpstreamHttp {
                provider: ProviderKind::OpenAi,
                status: status.as_u16(),
                body_excerpt: body.chars().take(300).collect(),
            });
        }

        let text = collect_openai_sse_text(&body)?;
        serde_json::from_str::<ExtractedVendorBatch>(clean_json_fence(&text)).map_err(|err| {
            AiError::MalformedStructuredOutput {
                provider: ProviderKind::OpenAi,
                detail: err.to_string(),
            }
        })
    }
}
