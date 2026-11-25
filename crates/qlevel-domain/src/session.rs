//! Session configuration, AI provider metadata, and final `LevelingResult` structures.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::alignment::{AlignmentGroup, MergedItemMode};
use crate::boq::MasterBoq;
use crate::currency::{CurrencyCode, Locale, SheetDirection, VatPolicy};
use crate::decimal::DecimalValue;
use crate::traps::DetectedTrap;
use crate::vendor::CellStatus;

/// Supported direct cloud Multimodal AI providers (strictly 3; zero proxies or local LLMs).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum ProviderKind {
    OpenAi,
    Gemini,
    Claude,
}

/// Authentication mechanism for a direct cloud provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum AuthMethod {
    ChatGptOAuth,
    ApiKey,
}

/// Reasoning effort setting (`Auto` routes dynamically per batch complexity).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum EffortLevel {
    Auto,
    Low,
    Medium,
    High,
}

/// Live-discovered Multimodal model metadata returned by provider `/models` filtering.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfo {
    pub id: String,
    pub display_name: String,
    pub provider: ProviderKind,
    pub supports_vision: bool,
    pub supports_pdf_input: bool,
    pub supports_structured_output: bool,
    pub supported_efforts: Vec<EffortLevel>,
    pub is_fast_tier: bool,
}

/// AI routing configuration for a leveling session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct AiConfig {
    /// `None` denotes automatic provider selection from connected credentials.
    pub provider: Option<ProviderKind>,
    /// `None` or `"auto"` denotes automatic per-batch model routing.
    pub model_id: Option<String>,
    pub effort: EffortLevel,
}

/// User-selectable Excel comparison workbook structure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum WorkbookLayoutMode {
    SingleMasterSheet,
    DualTab,
}

/// Strategy used to plug unquoted `Measured` BOQ items from peer bids or PTE.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum PeerPlugStrategy {
    LowestCompliant,
    AverageCompliant,
    HighestCompliant,
    PteEstimate,
}

/// Full configuration controlling a commercial & technical leveling run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct LevelingSessionConfig {
    pub session_id: String,
    pub ai: AiConfig,
    pub currency: CurrencyCode,
    pub vat_policy: VatPolicy,
    pub layout_mode: WorkbookLayoutMode,
    pub merged_item_mode: MergedItemMode,
    pub peer_plug_strategy: PeerPlugStrategy,
    pub ui_locale: Locale,
    pub sheet_direction: SheetDirection,
    pub approved_vendors: Vec<String>,
    pub custom_rules: Vec<String>,
}

/// Computed cell for `(boq_item_id, vendor_id)` ready for Excel formula emission.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct LeveledLineCell {
    pub boq_item_id: String,
    pub vendor_id: String,
    pub status: CellStatus,
    pub net_unit_rate: DecimalValue,
    pub as_submitted_net_total: DecimalValue,
    pub arithmetic_delta: DecimalValue,
    pub scope_plug_amount: DecimalValue,
    pub leveled_net_total: DecimalValue,
    pub highlight_yellow: bool,
    pub audit_note: Option<String>,
}

/// Vendor-level equalized commercial and technical scorecard.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct VendorEvaluationSummary {
    pub vendor_id: String,
    pub vendor_name: String,
    pub as_submitted_total: DecimalValue,
    pub arithmetic_and_qty_delta: DecimalValue,
    pub discount_adjustment: DecimalValue,
    pub provisional_sum_equalized_total: DecimalValue,
    pub line_scope_plugs_total: DecimalValue,
    pub clause_plugs_total: DecimalValue,
    pub leveled_net_total: DecimalValue,
    pub vat_amount: DecimalValue,
    pub leveled_gross_total: DecimalValue,
    pub variance_vs_lowest_pct: DecimalValue,
    pub quoted_items_count: usize,
    pub unquoted_items_count: usize,
    pub merged_items_count: usize,
    pub traps_count: usize,
    pub rank: usize,
    pub is_true_low_bidder: bool,
}

/// Complete deterministic output of a `qlevel` leveling run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct LevelingResult {
    pub session_id: String,
    pub config: LevelingSessionConfig,
    pub master_boq: MasterBoq,
    pub cells: Vec<LeveledLineCell>,
    pub alignment_groups: Vec<AlignmentGroup>,
    pub traps: Vec<DetectedTrap>,
    pub vendor_summaries: Vec<VendorEvaluationSummary>,
    pub true_low_vendor_id: String,
    pub true_low_vendor_name: String,
    pub true_low_leveled_net: DecimalValue,
}
