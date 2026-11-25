//! Thin Tauri v2 wiring delegating all business, vault, and leveling logic to `qlevel-bridge`.

#![forbid(unsafe_code)]

use std::path::Path;

use qlevel_bridge::commands::{
    handle_cancel_leveling_session, handle_disconnect_provider, handle_export_workbook,
    handle_fetch_multimodal_models, handle_list_provider_statuses, handle_resolve_ambiguity_prompt,
    handle_save_provider_api_key, handle_start_leveling_session, handle_start_openai_chatgpt_signin,
    handle_submit_rendered_pdf_pages, handle_triage_dropped_files, handle_triage_pasted_clipboard,
    BridgeEventEmitter,
};
use qlevel_bridge::{
    check_or_apply_portable_update, AmbiguityDecisionInput, AmbiguityPromptPayload, BridgeError,
    BridgeResult, BridgeState, ExportWorkbookInput, LevelingProgressEvent,
    LevelingSessionResultDto, PdfRasterRequestEvent, ProviderAuthStatusDto, SaveApiKeyInput,
    StagedInputDocumentDto, StartLevelingSessionInput, SubmitRenderedPdfPagesInput,
    TriageClipboardInput, UpdateCheckStatusDto, WorkbookExportOutcomeDto,
};
use qlevel_domain::{ModelInfo, ProviderKind};
use tauri::{AppHandle, Emitter, State};

struct TauriWindowEmitter {
    app: AppHandle,
}

impl BridgeEventEmitter for TauriWindowEmitter {
    fn emit_progress(&self, payload: &LevelingProgressEvent) {
        let _ = self.app.emit("qlevel://progress", payload);
    }

    fn emit_ambiguity_prompt(&self, payload: &AmbiguityPromptPayload) {
        let _ = self.app.emit("qlevel://ambiguity-prompt", payload);
    }

    fn emit_pdf_raster_request(&self, payload: &PdfRasterRequestEvent) {
        let _ = self.app.emit("qlevel://pdf-raster-request", payload);
    }
}

#[tauri::command]
fn qlevel_list_provider_statuses(
    state: State<'_, BridgeState>,
) -> BridgeResult<Vec<ProviderAuthStatusDto>> {
    handle_list_provider_statuses(&state)
}

#[tauri::command]
async fn qlevel_start_openai_chatgpt_signin(
    state: State<'_, BridgeState>,
) -> BridgeResult<ProviderAuthStatusDto> {
    handle_start_openai_chatgpt_signin(&state).await
}

#[tauri::command]
fn qlevel_save_provider_api_key(
    state: State<'_, BridgeState>,
    input: SaveApiKeyInput,
) -> BridgeResult<ProviderAuthStatusDto> {
    handle_save_provider_api_key(&state, input)
}

#[tauri::command]
fn qlevel_disconnect_provider(
    state: State<'_, BridgeState>,
    provider: ProviderKind,
) -> BridgeResult<ProviderAuthStatusDto> {
    handle_disconnect_provider(&state, provider)
}

#[tauri::command]
async fn qlevel_fetch_multimodal_models(
    state: State<'_, BridgeState>,
    provider: Option<ProviderKind>,
) -> BridgeResult<Vec<ModelInfo>> {
    handle_fetch_multimodal_models(&state, provider).await
}

#[tauri::command]
async fn qlevel_pick_input_files() -> BridgeResult<Vec<String>> {
    let selected = tokio::task::spawn_blocking(|| {
        rfd::FileDialog::new()
            .add_filter(
                "Tender Documents",
                &["xlsx", "xlsm", "xls", "xlsb", "csv", "pdf"],
            )
            .pick_files()
    })
    .await
    .map_err(|e| BridgeError::Internal(e.to_string()))?;

    Ok(selected
        .unwrap_or_default()
        .into_iter()
        .map(|p| p.display().to_string())
        .collect())
}

#[tauri::command]
async fn qlevel_pick_custom_rules_file() -> BridgeResult<Option<String>> {
    let picked = tokio::task::spawn_blocking(|| {
        rfd::FileDialog::new()
            .add_filter("Audit Rules Text", &["txt", "md"])
            .pick_file()
    })
    .await
    .map_err(|e| BridgeError::Internal(e.to_string()))?;

    Ok(picked.map(|p| p.display().to_string()))
}

#[tauri::command]
async fn qlevel_pick_save_workbook_path(suggested_path: String) -> BridgeResult<Option<String>> {
    let picked = tokio::task::spawn_blocking(move || {
        let default_name = Path::new(&suggested_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("qlevel_Bid_Comparison.xlsx");
        rfd::FileDialog::new()
            .set_file_name(default_name)
            .add_filter("Excel Workbook", &["xlsx"])
            .save_file()
    })
    .await
    .map_err(|e| BridgeError::Internal(e.to_string()))?;

    Ok(picked.map(|p| p.display().to_string()))
}

#[tauri::command]
fn qlevel_read_pdf_bytes(file_path: String) -> BridgeResult<Vec<u8>> {
    std::fs::read(&file_path).map_err(|e| BridgeError::FileCorrupted {
        file_name: file_path,
        reason: e.to_string(),
    })
}

#[tauri::command]
fn qlevel_triage_dropped_files(
    app: AppHandle,
    file_paths: Vec<String>,
) -> BridgeResult<Vec<StagedInputDocumentDto>> {
    let emitter = TauriWindowEmitter { app };
    handle_triage_dropped_files(&emitter, file_paths)
}

#[tauri::command]
fn qlevel_triage_pasted_clipboard(
    input: TriageClipboardInput,
) -> BridgeResult<StagedInputDocumentDto> {
    handle_triage_pasted_clipboard(input)
}

#[tauri::command]
async fn qlevel_submit_rendered_pdf_pages(
    state: State<'_, BridgeState>,
    input: SubmitRenderedPdfPagesInput,
) -> BridgeResult<()> {
    handle_submit_rendered_pdf_pages(&state, input).await
}

#[tauri::command]
async fn qlevel_start_leveling_session(
    app: AppHandle,
    state: State<'_, BridgeState>,
    input: StartLevelingSessionInput,
) -> BridgeResult<LevelingSessionResultDto> {
    let emitter = TauriWindowEmitter { app };
    handle_start_leveling_session(&state, &emitter, input).await
}

#[tauri::command]
async fn qlevel_cancel_leveling_session(
    state: State<'_, BridgeState>,
    session_id: String,
) -> BridgeResult<()> {
    handle_cancel_leveling_session(&state, &session_id).await
}

#[tauri::command]
async fn qlevel_resolve_ambiguity_prompt(
    state: State<'_, BridgeState>,
    decision: AmbiguityDecisionInput,
) -> BridgeResult<()> {
    handle_resolve_ambiguity_prompt(&state, decision).await
}

#[tauri::command]
async fn qlevel_export_workbook(
    state: State<'_, BridgeState>,
    input: ExportWorkbookInput,
) -> BridgeResult<WorkbookExportOutcomeDto> {
    handle_export_workbook(&state, input).await
}

#[tauri::command]
fn qlevel_open_workbook_in_excel(file_path: String) -> BridgeResult<()> {
    open::that(&file_path).map_err(|e| BridgeError::Internal(e.to_string()))
}

#[tauri::command]
fn qlevel_reveal_workbook_in_folder(file_path: String) -> BridgeResult<()> {
    let parent = Path::new(&file_path)
        .parent()
        .unwrap_or_else(|| Path::new("."));
    open::that(parent).map_err(|e| BridgeError::Internal(e.to_string()))
}

#[tauri::command]
fn qlevel_open_external_url(url: String) -> BridgeResult<()> {
    if !url.starts_with("https://") {
        return Err(BridgeError::Internal("Only https:// URLs are permitted".to_owned()));
    }
    open::that(&url).map_err(|e| BridgeError::Internal(e.to_string()))
}

#[tauri::command]
async fn qlevel_check_for_updates(apply_portable_update: bool) -> BridgeResult<UpdateCheckStatusDto> {
    check_or_apply_portable_update(apply_portable_update).await
}

/// Initializes logging, `BridgeState`, and launches the Tauri v2 application loop.
pub fn run_desktop_app() -> Result<(), String> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .try_init();

    let bridge_state = BridgeState::initialize_default().map_err(|e| e.to_string())?;

    tauri::Builder::default()
        .manage(bridge_state)
        .invoke_handler(tauri::generate_handler![
            qlevel_list_provider_statuses,
            qlevel_start_openai_chatgpt_signin,
            qlevel_save_provider_api_key,
            qlevel_disconnect_provider,
            qlevel_fetch_multimodal_models,
            qlevel_pick_input_files,
            qlevel_pick_custom_rules_file,
            qlevel_pick_save_workbook_path,
            qlevel_read_pdf_bytes,
            qlevel_triage_dropped_files,
            qlevel_triage_pasted_clipboard,
            qlevel_submit_rendered_pdf_pages,
            qlevel_start_leveling_session,
            qlevel_cancel_leveling_session,
            qlevel_resolve_ambiguity_prompt,
            qlevel_export_workbook,
            qlevel_open_workbook_in_excel,
            qlevel_reveal_workbook_in_folder,
            qlevel_open_external_url,
            qlevel_check_for_updates,
        ])
        .run(tauri::generate_context!())
        .map_err(|e| e.to_string())
}
