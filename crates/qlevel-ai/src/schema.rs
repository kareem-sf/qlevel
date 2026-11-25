//! Structured semantic extraction contracts and provider client trait.
//!
//! Notice: AI models NEVER compute arithmetic totals or VAT conversions; they return
//! verbatim decimal strings and clause citations for deterministic verification in `qlevel-engine`.

use async_trait::async_trait;
use qlevel_domain::{EffortLevel, ModelInfo, ProviderKind, TrapCategory, TrapSeverity, VatMode};
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::error::AiResult;

/// Persisted OAuth PKCE session bundle for `OpenAI Sign in with ChatGPT` (`SIWC`).
#[derive(Debug, Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
#[serde(rename_all = "camelCase")]
pub struct OpenAiSiwcSession {
    /// Dynamically assigned `oaiapp_...` client identifier returned on callback.
    pub dynamic_client_id: String,
    /// Stable local host URN (`urn:uuid:...`) registered for this workstation.
    pub agent_host_urn: String,
    pub access_token: String,
    pub refresh_token: String,
    pub id_token: Option<String>,
    pub account_email: Option<String>,
    /// Unix epoch timestamp (seconds) when `access_token` expires.
    pub expires_at_epoch_secs: i64,
}

/// Runtime credential supplied to a `ProviderClient` (never logged or serialized over IPC).
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub enum ProviderAuthCredential {
    ApiKey {
        key: String,
    },
    OpenAiSiwc {
        access_token: String,
        dynamic_client_id: String,
    },
}

/// Rasterized scanned PDF page or pasted quote screenshot attached to an extraction batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageAttachment {
    pub page_number: u32,
    pub mime_type: String,
    pub base64_data: String,
}

/// Single semantic extraction batch sent to a Multimodal provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractionBatchInput {
    pub batch_id: String,
    pub vendor_name: String,
    pub master_boq_context_json: String,
    pub vendor_raw_text_or_rows: String,
    pub images: Vec<ImageAttachment>,
    pub approved_vendors: Vec<String>,
    pub custom_rules: Vec<String>,
}

/// Single vendor line extracted semantically by the Multimodal model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractedLineItem {
    /// Matched Master BOQ item ID(s); >1 indicates a merged vendor lump-sum row.
    pub matched_master_boq_ids: Vec<String>,
    pub vendor_code: Option<String>,
    pub vendor_description: String,
    pub quoted_unit: String,
    /// Verbatim numeric string as printed in the vendor quote (`"12"`, `"1250.50"`).
    pub quoted_qty_raw: String,
    pub quoted_unit_rate_raw: String,
    pub quoted_extended_total_raw: String,
    pub vat_mode: VatMode,
    pub is_unquoted_or_excluded: bool,
    pub is_moved_to_optional: bool,
    pub proposed_brand: Option<String>,
    pub source_page: Option<u32>,
    pub confidence_pct: u8,
    pub note: Option<String>,
}

/// Technical or commercial trap clause extracted from fine print or table notes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractedTrapDto {
    pub category: TrapCategory,
    pub severity: TrapSeverity,
    pub related_boq_id: Option<String>,
    pub title: String,
    pub detail: String,
    pub verbatim_quote: String,
    pub source_page: Option<u32>,
    pub suggested_plug_pct_raw: Option<String>,
    pub suggested_plug_lump_sum_raw: Option<String>,
}

/// Structured JSON response returned by a Multimodal provider for one batch.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractedVendorBatch {
    pub detected_currency_code: Option<String>,
    pub default_vat_mode: Option<VatMode>,
    pub general_discount_pct_raw: Option<String>,
    pub lump_sum_discount_raw: Option<String>,
    pub payment_terms: Option<String>,
    pub delivery_incoterms: Option<String>,
    pub delivery_weeks: Option<u32>,
    pub validity_days: Option<u32>,
    pub warranty_months: Option<u32>,
    pub lines: Vec<ExtractedLineItem>,
    pub traps: Vec<ExtractedTrapDto>,
}

/// Canonical system instructions shared across OpenAI, Gemini, and Claude.
#[must_use]
pub fn build_extraction_instructions(custom_rules: &[String], avl: &[String]) -> String {
    let mut prompt = String::from(
        "You are a Senior Quantity Surveyor and Tendering Auditor performing Commercial and Technical Bid Leveling.\n\
         STRICT ZERO-ARITHMETIC RULE: Copy vendor quantities, unit rates, and extended totals verbatim as ASCII decimal strings. Do NOT multiply or adjust VAT yourself.\n\
         ALIGNMENT RULES:\n\
         1. Map each vendor line to its exact Master BOQ item ID(s) in `matchedMasterBoqIds`.\n\
         2. If a vendor combines multiple Master BOQ items into one lump sum ('Included', 'شامل أعلاه'), list ALL covered Master BOQ IDs in `matchedMasterBoqIds` on that single row.\n\
         3. Audit all footnotes, cover letters, and last-page terms for the 7 trap categories: `scope_exclusion`, `technical_deviation`, `avl_regulatory`, `qty_arithmetic`, `logistics_incoterms`, `commercial_risk`, and `closeout_docs`.\n\
         4. Output strictly valid JSON matching the `ExtractedVendorBatch` schema with camelCase keys.",
    );

    if !avl.is_empty() {
        prompt.push_str("\nAPPROVED VENDOR LIST (AVL): ");
        prompt.push_str(&avl.join(", "));
    }
    if !custom_rules.is_empty() {
        prompt.push_str("\nPROJECT CUSTOM AUDIT RULES:\n");
        for (idx, rule) in custom_rules.iter().enumerate() {
            prompt.push_str(&format!("{}. {}\n", idx + 1, rule));
        }
    }
    prompt
}

/// Unified trait implemented by the 3 direct cloud Multimodal clients.
#[async_trait]
pub trait ProviderClient: Send + Sync {
    /// Returns the provider identifier (`OpenAi`, `Gemini`, or `Claude`).
    fn provider_kind(&self) -> ProviderKind;

    /// Queries the live `/models` endpoint and returns strictly filtered Multimodal models.
    async fn list_multimodal_models(&self) -> AiResult<Vec<ModelInfo>>;

    /// Executes structured semantic extraction on a tender batch.
    async fn extract_vendor_batch(
        &self,
        model_id: &str,
        effort: EffortLevel,
        batch: &ExtractionBatchInput,
    ) -> AiResult<ExtractedVendorBatch>;
}
