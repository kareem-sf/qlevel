//! Strongly typed Data Transfer Objects (`DTOs`) shared across Tauri IPC and `packages/bindings`.

use qlevel_domain::{
    AlignmentGroup, CurrencyCode, DecimalValue, LevelingSessionConfig, ProviderKind, TrapCategory,
    WorkbookLayoutMode,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Connected authentication method state displayed in the UI header.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum ConnectedAuthKindDto {
    ChatgptOauth,
    ApiKey,
    None,
}

/// Sanitized connection status for one of the 3 supported providers (zero secret values).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ProviderAuthStatusDto {
    pub provider: ProviderKind,
    pub connected: bool,
    pub auth_method: ConnectedAuthKindDto,
    pub account_label: Option<String>,
    pub gemini_privacy_acknowledged: bool,
    pub expires_at_epoch_secs: Option<i64>,
    pub manage_usage_url: Option<String>,
}

/// Request payload saving a direct provider API key into the `EnvelopeVault`.
#[derive(Debug, Clone, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct SaveApiKeyInput {
    pub provider: ProviderKind,
    pub api_key: String,
    pub acknowledge_gemini_free_tier_privacy: bool,
}

/// Assigned document role in the workspace dropzone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum DocumentRoleDto {
    MasterBoq,
    VendorQuote,
    SpecOrAvl,
}

/// Origin of a staged tender document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum InputSourceKindDto {
    FilePath,
    PastedTable,
    PastedImage,
}

/// Rasterized scanned PDF page sent from WebView `pdfjs-dist` `<canvas>` to Rust.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct RenderedPageImageDto {
    pub page_number: u32,
    pub width_px: u32,
    pub height_px: u32,
    pub mime_type: String,
    pub base64_data: String,
}

/// Staged input card displayed in the `Basic` dropzone workspace.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct StagedInputDocumentDto {
    pub id: String,
    pub file_name: String,
    pub file_path: Option<String>,
    pub source_kind: InputSourceKindDto,
    pub role: DocumentRoleDto,
    pub vendor_name: Option<String>,
    pub detected_row_count: Option<usize>,
    pub page_count: Option<usize>,
    pub hidden_rows_count: usize,
    pub pages_needing_rasterization: Vec<u32>,
    pub pasted_text_payload: Option<String>,
    pub pasted_image_base64: Option<String>,
    pub rendered_pages: Vec<RenderedPageImageDto>,
}

/// Clipboard paste payload (`Ctrl+V`) sent for triage.
#[derive(Debug, Clone, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct TriageClipboardInput {
    pub html: Option<String>,
    pub plain_text: Option<String>,
    pub image_base64: Option<String>,
    pub mime_type: Option<String>,
}

/// Submitted rasterized PDF page images for a scanned tender PDF.
#[derive(Debug, Clone, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct SubmitRenderedPdfPagesInput {
    pub document_id: String,
    pub pages: Vec<RenderedPageImageDto>,
}

/// Request payload launching a commercial & technical leveling session.
#[derive(Debug, Clone, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct StartLevelingSessionInput {
    pub documents: Vec<StagedInputDocumentDto>,
    pub config: LevelingSessionConfig,
}

/// High-level stages shown on the cancellable progress bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum ProgressStageDto {
    ReadingDocuments,
    RasterizingScannedPages,
    ExtractingVendorQuotes,
    AwaitingAmbiguityDecision,
    AuditingMathAndTraps,
    BuildingWorkbook,
    Completed,
}

/// Event payload emitted on `"qlevel://progress"`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct LevelingProgressEvent {
    pub session_id: String,
    pub stage: ProgressStageDto,
    pub completed_steps: usize,
    pub total_steps: usize,
    pub active_vendor_name: Option<String>,
    pub percent: u8,
    pub checkpoint_hit: bool,
}

/// Event payload emitted on `"qlevel://pdf-raster-request"` when scanned PDF pages need rendering.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct PdfRasterRequestEvent {
    pub document_id: String,
    pub file_name: String,
    pub file_path: String,
    pub page_numbers: Vec<u32>,
    pub target_dpi: u32,
}

/// Engineer's quick-card choice for resolving a merged vendor lump-sum row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum AmbiguityResolutionChoice {
    AnchorAndZeroFollowers,
    ProportionalSplit,
    AcceptedAlternate,
    ExcludedWithPlug,
}

/// Event payload emitted on `"qlevel://ambiguity-prompt"` in `InteractiveReview` mode.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct AmbiguityPromptPayload {
    pub session_id: String,
    pub group: AlignmentGroup,
    pub vendor_name: String,
    pub master_descriptions: Vec<String>,
    pub pending_count: usize,
}

/// Engineer's response resolving an interactive merged-item prompt.
#[derive(Debug, Clone, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct AmbiguityDecisionInput {
    pub session_id: String,
    pub group_id: String,
    pub choice: AmbiguityResolutionChoice,
    pub reviewer_note: Option<String>,
}

/// Vendor summary row presented on the `ResultView` screen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct VendorSummaryRowDto {
    pub vendor_id: String,
    pub vendor_name: String,
    pub submitted_total: DecimalValue,
    pub arithmetic_delta: DecimalValue,
    pub total_plugs: DecimalValue,
    pub leveled_net_total: DecimalValue,
    pub leveled_gross_total: DecimalValue,
    pub variance_vs_lowest_pct: DecimalValue,
    pub traps_count: usize,
    pub rank: usize,
    pub is_true_low_bidder: bool,
}

/// Technical/commercial trap highlight card presented on `ResultView`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct TrapFindingSummaryDto {
    pub category: TrapCategory,
    pub vendor_name: String,
    pub title: String,
    pub detail: String,
    pub source_snippet: String,
    pub financial_impact: Option<DecimalValue>,
}

/// Final UI summary payload returned when `qlevel_start_leveling_session` completes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct LevelingSessionResultDto {
    pub session_id: String,
    pub currency: CurrencyCode,
    pub vat_pct: DecimalValue,
    pub is_synthetic_boq: bool,
    pub total_boq_items: usize,
    pub merged_groups_count: usize,
    pub true_low_vendor_name: String,
    pub true_low_leveled_net: DecimalValue,
    pub layout_mode: WorkbookLayoutMode,
    pub vendors: Vec<VendorSummaryRowDto>,
    pub traps: Vec<TrapFindingSummaryDto>,
    pub suggested_output_path: String,
}

/// Request payload saving the active session's `.xlsx` comparison workbook.
#[derive(Debug, Clone, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ExportWorkbookInput {
    pub session_id: String,
    pub target_path: String,
    pub use_fallback_if_locked: bool,
}

/// Discriminated outcome of saving the `.xlsx` comparison workbook.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum WorkbookExportOutcomeDto {
    Saved {
        #[serde(rename = "outputPath")]
        output_path: String,
    },
    FileLocked {
        #[serde(rename = "lockedPath")]
        locked_path: String,
        #[serde(rename = "suggestedFallbackPath")]
        suggested_fallback_path: String,
        hint: String,
    },
}

/// Portable self-updater status payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCheckStatusDto {
    pub current_version: String,
    pub latest_version: Option<String>,
    pub update_available: bool,
    pub applied_portable_update: bool,
}

/// Machine-readable IPC error code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum BridgeErrorCode {
    AuthExpired,
    OpenaiUsageLimitExceeded,
    GeminiFreeTierBlocked,
    FilePasswordProtected,
    FileCorrupted,
    ExcelFileLocked,
    MissingVendorQuote,
    Cancelled,
    NetworkError,
    InternalError,
}

/// Sanitized IPC error object received by `packages/bindings`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct BridgeErrorPayload {
    pub code: BridgeErrorCode,
    pub message: String,
    pub affected_file_name: Option<String>,
    pub manage_usage_url: Option<String>,
    pub suggested_fallback_path: Option<String>,
}
