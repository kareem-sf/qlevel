//! Google Gemini `v1beta` direct Multimodal client (`gemini-3.8-flash`, `gemini-4-*`).
//!
//! Uses `x-goog-api-key` header (never query-string secret URLs), `responseMimeType: "application/json"`,
//! `thinkingConfig.thinkingLevel` (`LOW|MEDIUM|HIGH`, omitted in `Auto`), and skips `"thought": true` parts.

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

const GEMINI_BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta";

/// Direct Google Gemini `v1beta` client.
pub struct GeminiClient {
    http: reqwest::Client,
    api_key: Zeroizing<String>,
}

impl GeminiClient {
    #[must_use]
    pub fn new(http: reqwest::Client, api_key: String) -> Self {
        Self {
            http,
            api_key: Zeroizing::new(api_key),
        }
    }
}

/// Parses `GET /v1beta/models` keeping entries that support `generateContent`.
pub fn parse_gemini_models_response(body: &str) -> AiResult<Vec<RawProviderModel>> {
    let root: Value = serde_json::from_str(body).map_err(|err| AiError::MalformedStructuredOutput {
        provider: ProviderKind::Gemini,
        detail: err.to_string(),
    })?;

    let arr = root
        .get("models")
        .and_then(Value::as_array)
        .ok_or_else(|| AiError::MalformedStructuredOutput {
            provider: ProviderKind::Gemini,
            detail: "Missing `models` array in Gemini /v1beta/models".to_owned(),
        })?;

    let mut out = Vec::with_capacity(arr.len());
    for m in arr {
        let Some(name) = m.get("name").and_then(Value::as_str) else {
            continue;
        };
        let display_name = m
            .get("displayName")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned);
        let supports_gen = m
            .get("supportedGenerationMethods")
            .and_then(Value::as_array)
            .is_some_and(|methods| {
                methods
                    .iter()
                    .any(|v| v.as_str() == Some("generateContent"))
            });

        out.push(RawProviderModel {
            id: name.trim_start_matches("models/").to_owned(),
            display_name,
            supports_vision_hint: None,
            supports_generation_hint: Some(supports_gen),
            is_deprecated: false,
        });
    }
    Ok(out)
}

/// Builds the Gemini `generateContent` JSON request body.
#[must_use]
pub fn build_gemini_generate_payload(
    effort: EffortLevel,
    batch: &ExtractionBatchInput,
) -> Value {
    let sys_text = build_extraction_instructions(&batch.custom_rules, &batch.approved_vendors);

    let mut parts = Vec::with_capacity(batch.images.len() + 1);
    parts.push(json!({
        "text": format!(
            "VENDOR NAME: {}\n\nMASTER BOQ BASELINE:\n{}\n\nVENDOR QUOTATION CONTENT:\n{}",
            batch.vendor_name,
            batch.master_boq_context_json,
            batch.vendor_raw_text_or_rows
        )
    }));

    for img in &batch.images {
        parts.push(json!({
            "inlineData": {
                "mimeType": img.mime_type,
                "data": img.base64_data
            }
        }));
    }

    let mut gen_config = json!({
        "responseMimeType": "application/json"
    });

    match effort {
        EffortLevel::Auto => {}
        EffortLevel::Low => {
            gen_config["thinkingConfig"] = json!({ "thinkingLevel": "LOW" });
        }
        EffortLevel::Medium => {
            gen_config["thinkingConfig"] = json!({ "thinkingLevel": "MEDIUM" });
        }
        EffortLevel::High => {
            gen_config["thinkingConfig"] = json!({ "thinkingLevel": "HIGH" });
        }
    }

    json!({
        "systemInstruction": {
            "parts": [{ "text": sys_text }]
        },
        "contents": [
            {
                "role": "user",
                "parts": parts
            }
        ],
        "generationConfig": gen_config
    })
}

/// Extracts non-thought text parts from a Gemini `generateContent` response.
pub fn extract_gemini_non_thought_text(body: &str) -> AiResult<String> {
    let root: Value = serde_json::from_str(body).map_err(|err| AiError::MalformedStructuredOutput {
        provider: ProviderKind::Gemini,
        detail: err.to_string(),
    })?;

    let parts = root
        .get("candidates")
        .and_then(Value::as_array)
        .and_then(|cands| cands.first())
        .and_then(|c| c.get("content"))
        .and_then(|content| content.get("parts"))
        .and_then(Value::as_array)
        .ok_or_else(|| AiError::MalformedStructuredOutput {
            provider: ProviderKind::Gemini,
            detail: "Missing candidates[0].content.parts in Gemini response".to_owned(),
        })?;

    let mut combined = String::new();
    for part in parts {
        if part.get("thought").and_then(Value::as_bool) == Some(true) {
            continue;
        }
        if let Some(txt) = part.get("text").and_then(Value::as_str) {
            combined.push_str(txt);
        }
    }

    if combined.trim().is_empty() {
        return Err(AiError::MalformedStructuredOutput {
            provider: ProviderKind::Gemini,
            detail: "No non-thought JSON text found in Gemini response".to_owned(),
        });
    }
    Ok(combined)
}

#[async_trait]
impl ProviderClient for GeminiClient {
    fn provider_kind(&self) -> ProviderKind {
        ProviderKind::Gemini
    }

    async fn list_multimodal_models(&self) -> AiResult<Vec<ModelInfo>> {
        let url = format!("{GEMINI_BASE_URL}/models?pageSize=100");
        let resp = self
            .http
            .get(&url)
            .header("x-goog-api-key", self.api_key.as_str())
            .send()
            .await
            .map_err(|err| AiError::Network {
                provider: ProviderKind::Gemini,
                detail: err.to_string(),
            })?;

        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        if status.as_u16() == 400 || status.as_u16() == 401 || status.as_u16() == 403 {
            return Err(AiError::Unauthorized {
                provider: ProviderKind::Gemini,
                detail: "Gemini API key rejected".to_owned(),
            });
        }
        if !status.is_success() {
            return Err(AiError::UpstreamHttp {
                provider: ProviderKind::Gemini,
                status: status.as_u16(),
                body_excerpt: body.chars().take(300).collect(),
            });
        }

        let raw_models = parse_gemini_models_response(&body)?;
        Ok(filter_live_multimodal_models(
            ProviderKind::Gemini,
            raw_models,
        ))
    }

    async fn extract_vendor_batch(
        &self,
        model_id: &str,
        effort: EffortLevel,
        batch: &ExtractionBatchInput,
    ) -> AiResult<ExtractedVendorBatch> {
        let clean_model = model_id.trim_start_matches("models/");
        let url = format!("{GEMINI_BASE_URL}/models/{clean_model}:generateContent");
        let payload = build_gemini_generate_payload(effort, batch);

        let resp = self
            .http
            .post(&url)
            .header("x-goog-api-key", self.api_key.as_str())
            .json(&payload)
            .send()
            .await
            .map_err(|err| AiError::Network {
                provider: ProviderKind::Gemini,
                detail: err.to_string(),
            })?;

        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        if status.as_u16() == 401 || status.as_u16() == 403 {
            return Err(AiError::Unauthorized {
                provider: ProviderKind::Gemini,
                detail: "Gemini API key unauthorized".to_owned(),
            });
        }
        if status.as_u16() == 429 {
            return Err(AiError::RateLimited {
                provider: ProviderKind::Gemini,
                detail: body.chars().take(240).collect(),
                retry_after_seconds: None,
            });
        }
        if !status.is_success() {
            return Err(AiError::UpstreamHttp {
                provider: ProviderKind::Gemini,
                status: status.as_u16(),
                body_excerpt: body.chars().take(300).collect(),
            });
        }

        let text = extract_gemini_non_thought_text(&body)?;
        serde_json::from_str::<ExtractedVendorBatch>(clean_json_fence(&text)).map_err(|err| {
            AiError::MalformedStructuredOutput {
                provider: ProviderKind::Gemini,
                detail: err.to_string(),
            }
        })
    }
}
