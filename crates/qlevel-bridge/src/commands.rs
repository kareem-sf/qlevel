//! Pure framework-agnostic command handlers invoked by the thin Tauri binary shell.

use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::Utc;
use qlevel_ai::siwc::bind_exclusive_loopback;
use qlevel_ai::{
    build_provider_client, build_siwc_authorize_url, generate_pkce_pair, ImageAttachment,
    ProviderAuthCredential,
};
use qlevel_domain::{
    AlignmentResolution, DecimalValue, LevelingResult, MasterBoq, MergedItemMode, ModelInfo,
    ProviderKind,
};
use qlevel_engine::pipeline::PreparedVendorInput;
use qlevel_engine::{
    build_synthetic_master_boq, evaluate_quotes_deterministic, run_extraction_and_leveling_pipeline,
    RawVendorCandidateItem,
};
use qlevel_parser::{
    extract_master_boq_from_sheet, inspect_pdf_bytes, parse_clipboard_table, parse_excel_bytes,
    DocumentClassification, ParsedDocument,
};
use qlevel_workbook::{save_leveling_workbook_to_path, WorkbookWriteOutcome};
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::dto::{
    AmbiguityDecisionInput, AmbiguityPromptPayload, AmbiguityResolutionChoice, DocumentRoleDto,
    ExportWorkbookInput, InputSourceKindDto, LevelingProgressEvent, LevelingSessionResultDto,
    PdfRasterRequestEvent, ProgressStageDto, ProviderAuthStatusDto, SaveApiKeyInput,
    StagedInputDocumentDto, StartLevelingSessionInput, SubmitRenderedPdfPagesInput,
    TrapFindingSummaryDto, TriageClipboardInput, VendorSummaryRowDto, WorkbookExportOutcomeDto,
};
use crate::error::{BridgeError, BridgeResult};
use crate::state::BridgeState;

/// Event sink trait implemented by `tauri::AppHandle` in `apps/desktop/src-tauri`.
pub trait BridgeEventEmitter: Send + Sync {
    fn emit_progress(&self, payload: &LevelingProgressEvent);
    fn emit_ambiguity_prompt(&self, payload: &AmbiguityPromptPayload);
    fn emit_pdf_raster_request(&self, payload: &PdfRasterRequestEvent);
}

fn map_role(c: DocumentClassification) -> DocumentRoleDto {
    match c {
        DocumentClassification::MasterBoq => DocumentRoleDto::MasterBoq,
        DocumentClassification::VendorQuote => DocumentRoleDto::VendorQuote,
        DocumentClassification::SpecOrAvl => DocumentRoleDto::SpecOrAvl,
    }
}

/// Lists connection statuses across OpenAI, Gemini, and Claude.
pub fn handle_list_provider_statuses(state: &BridgeState) -> BridgeResult<Vec<ProviderAuthStatusDto>> {
    state.vault.list_statuses()
}

/// Runs the local browser `Sign in with ChatGPT` (`SIWC`) OAuth PKCE flow and stores the encrypted session.
pub async fn handle_start_openai_chatgpt_signin(
    state: &BridgeState,
) -> BridgeResult<ProviderAuthStatusDto> {
    let (agent_host_urn, cached_client_id) = state.vault.get_siwc_registration_metadata()?;
    let bound = bind_exclusive_loopback()?;
    let pkce = generate_pkce_pair();
    let auth_url = build_siwc_authorize_url(
        &bound.redirect_uri,
        &pkce,
        &agent_host_urn,
        cached_client_id.as_deref(),
    )?;

    open::that(&auth_url)
        .map_err(|e| BridgeError::Internal(format!("Could not launch system browser: {e}")))?;

    let session = state
        .siwc_coordinator
        .await_callback_and_exchange(
            bound.listener,
            &bound.redirect_uri,
            &pkce,
            &agent_host_urn,
            cached_client_id.as_deref(),
            Duration::from_secs(240),
        )
        .await?;

    state.vault.save_openai_siwc_session(session)
}

/// Saves a direct API key into `EnvelopeVault`.
pub fn handle_save_provider_api_key(
    state: &BridgeState,
    input: SaveApiKeyInput,
) -> BridgeResult<ProviderAuthStatusDto> {
    state.vault.save_api_key(
        input.provider,
        &input.api_key,
        input.acknowledge_gemini_free_tier_privacy,
    )
}

/// Disconnects a provider and scrubs its credentials from `EnvelopeVault`.
pub fn handle_disconnect_provider(
    state: &BridgeState,
    provider: ProviderKind,
) -> BridgeResult<ProviderAuthStatusDto> {
    state.vault.disconnect_provider(provider)
}

/// Resolves fresh credentials (refreshing near-expiry SIWC tokens automatically) and queries live `/models`.
pub async fn handle_fetch_multimodal_models(
    state: &BridgeState,
    provider: Option<ProviderKind>,
) -> BridgeResult<Vec<ModelInfo>> {
    let (resolved_provider, mut cred) = state.vault.resolve_credential(provider)?;

    if resolved_provider == ProviderKind::OpenAi {
        if let Some(siwc) = state.vault.load_openai_siwc_session()? {
            if siwc.expires_at_epoch_secs <= Utc::now().timestamp() + 90 {
                let refreshed = state.siwc_coordinator.refresh_session(&siwc).await?;
                cred = ProviderAuthCredential::OpenAiSiwc {
                    access_token: refreshed.access_token.clone(),
                    dynamic_client_id: refreshed.dynamic_client_id.clone(),
                };
                let _ = state.vault.save_openai_siwc_session(refreshed)?;
            }
        }
    }

    let client = build_provider_client(resolved_provider, cred, state.http_client.clone())?;
    Ok(client.list_multimodal_models().await?)
}

/// Triages dropped Excel/PDF/CSV files into `StagedInputDocumentDto` cards.
pub fn handle_triage_dropped_files(
    emitter: &dyn BridgeEventEmitter,
    file_paths: Vec<String>,
) -> BridgeResult<Vec<StagedInputDocumentDto>> {
    let mut staged = Vec::with_capacity(file_paths.len());

    for raw_path in file_paths {
        let path = Path::new(&raw_path);
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("document")
            .to_owned();
        let bytes = std::fs::read(path).map_err(|e| BridgeError::FileCorrupted {
            file_name: file_name.clone(),
            reason: e.to_string(),
        })?;

        let lower = file_name.to_lowercase();
        let parsed = if lower.ends_with(".pdf") {
            inspect_pdf_bytes(&file_name, &bytes)?
        } else if lower.ends_with(".csv") || lower.ends_with(".tsv") || lower.ends_with(".txt") {
            let txt = String::from_utf8_lossy(&bytes);
            parse_clipboard_table(&file_name, None, Some(&txt))?
        } else {
            parse_excel_bytes(&file_name, &bytes)?
        };

        let doc_id = format!("DOC-{}", Uuid::new_v4().simple());
        let raster_pages = parsed.pages_needing_rasterization();
        if !raster_pages.is_empty() {
            emitter.emit_pdf_raster_request(&PdfRasterRequestEvent {
                document_id: doc_id.clone(),
                file_name: file_name.clone(),
                file_path: raw_path.clone(),
                page_numbers: raster_pages.clone(),
                target_dpi: 200,
            });
        }

        let hidden_rows_count: usize = parsed.sheets.iter().map(|s| s.hidden_row_numbers.len()).sum();

        staged.push(StagedInputDocumentDto {
            id: doc_id,
            file_name,
            file_path: Some(raw_path),
            source_kind: InputSourceKindDto::FilePath,
            role: map_role(parsed.suggested_role),
            vendor_name: parsed.detected_vendor_name.clone(),
            detected_row_count: if parsed.sheets.is_empty() {
                None
            } else {
                Some(parsed.total_tabular_rows())
            },
            page_count: if parsed.pdf_pages.is_empty() {
                None
            } else {
                Some(parsed.pdf_pages.len())
            },
            hidden_rows_count,
            pages_needing_rasterization: raster_pages,
            pasted_text_payload: None,
            pasted_image_base64: None,
            rendered_pages: Vec::new(),
        });
    }

    Ok(staged)
}

/// Triages a `Ctrl+V` clipboard table or WhatsApp quote screenshot into a `StagedInputDocumentDto`.
pub fn handle_triage_pasted_clipboard(
    input: TriageClipboardInput,
) -> BridgeResult<StagedInputDocumentDto> {
    let doc_id = format!("CLIP-{}", Uuid::new_v4().simple());

    if let Some(img_b64) = input.image_base64 {
        return Ok(StagedInputDocumentDto {
            id: doc_id,
            file_name: "Pasted Quote Image.png".to_owned(),
            file_path: None,
            source_kind: InputSourceKindDto::PastedImage,
            role: DocumentRoleDto::VendorQuote,
            vendor_name: Some("Pasted Vendor Quote".to_owned()),
            detected_row_count: None,
            page_count: Some(1),
            hidden_rows_count: 0,
            pages_needing_rasterization: Vec::new(),
            pasted_text_payload: None,
            pasted_image_base64: Some(img_b64),
            rendered_pages: Vec::new(),
        });
    }

    let parsed = parse_clipboard_table(
        "Pasted Table",
        input.html.as_deref(),
        input.plain_text.as_deref(),
    )?;
    let serialized_rows = parsed
        .sheets
        .first()
        .map(|s| {
            s.rows
                .iter()
                .map(|r| r.cells.join("\t"))
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default();

    Ok(StagedInputDocumentDto {
        id: doc_id,
        file_name: "Pasted Spreadsheet Table".to_owned(),
        file_path: None,
        source_kind: InputSourceKindDto::PastedTable,
        role: map_role(parsed.suggested_role),
        vendor_name: Some("Pasted Vendor".to_owned()),
        detected_row_count: Some(parsed.total_tabular_rows()),
        page_count: None,
        hidden_rows_count: 0,
        pages_needing_rasterization: Vec::new(),
        pasted_text_payload: Some(serialized_rows),
        pasted_image_base64: None,
        rendered_pages: Vec::new(),
    })
}

/// Caches WebView-rasterized scanned PDF pages inside `BridgeState`.
pub async fn handle_submit_rendered_pdf_pages(
    state: &BridgeState,
    input: SubmitRenderedPdfPagesInput,
) -> BridgeResult<()> {
    let mut guard = state.rendered_pdf_cache.lock().await;
    guard.insert(input.document_id, input.pages);
    Ok(())
}

fn load_parsed_doc(doc: &StagedInputDocumentDto) -> BridgeResult<Option<ParsedDocument>> {
    match doc.source_kind {
        InputSourceKindDto::FilePath => {
            let Some(fp) = &doc.file_path else {
                return Ok(None);
            };
            let bytes = std::fs::read(fp).map_err(|e| BridgeError::FileCorrupted {
                file_name: doc.file_name.clone(),
                reason: e.to_string(),
            })?;
            let lower = doc.file_name.to_lowercase();
            if lower.ends_with(".pdf") {
                Ok(Some(inspect_pdf_bytes(&doc.file_name, &bytes)?))
            } else if lower.ends_with(".csv") || lower.ends_with(".tsv") || lower.ends_with(".txt") {
                let txt = String::from_utf8_lossy(&bytes);
                Ok(Some(parse_clipboard_table(&doc.file_name, None, Some(&txt))?))
            } else {
                Ok(Some(parse_excel_bytes(&doc.file_name, &bytes)?))
            }
        }
        InputSourceKindDto::PastedTable => {
            let txt = doc.pasted_text_payload.as_deref().unwrap_or("");
            Ok(Some(parse_clipboard_table(&doc.file_name, None, Some(txt))?))
        }
        InputSourceKindDto::PastedImage => Ok(None),
    }
}

fn format_parsed_doc_into_batches(parsed: &ParsedDocument) -> Vec<String> {
    let mut batches = Vec::new();
    for sheet in &parsed.sheets {
        for chunk in sheet.rows.chunks(60) {
            let mut buf = format!("Sheet: {}\n", sheet.sheet_name);
            for r in chunk {
                if r.is_hidden {
                    buf.push_str("[HIDDEN ROW] ");
                }
                buf.push_str(&format!("R{}: {}\n", r.row_number, r.cells.join(" | ")));
            }
            batches.push(buf);
        }
    }
    for page_chunk in parsed.pdf_pages.chunks(4) {
        let mut buf = String::new();
        for p in page_chunk {
            if !p.extracted_text.is_empty() {
                buf.push_str(&format!("--- Page {} ---\n{}\n", p.page_number, p.extracted_text));
            }
        }
        if !buf.trim().is_empty() {
            batches.push(buf);
        }
    }
    batches
}

/// Executes the complete Commercial & Technical Bid Leveling session.
pub async fn handle_start_leveling_session(
    state: &BridgeState,
    emitter: &dyn BridgeEventEmitter,
    input: StartLevelingSessionInput,
) -> BridgeResult<LevelingSessionResultDto> {
    let session_id = input.config.session_id.clone();
    let cancel_token = CancellationToken::new();
    {
        let mut guard = state.active_cancellations.lock().await;
        guard.insert(session_id.clone(), cancel_token.clone());
    }

    emitter.emit_progress(&LevelingProgressEvent {
        session_id: session_id.clone(),
        stage: ProgressStageDto::ReadingDocuments,
        completed_steps: 0,
        total_steps: input.documents.len().max(1),
        active_vendor_name: None,
        percent: 5,
        checkpoint_hit: false,
    });

    let vendor_docs: Vec<&StagedInputDocumentDto> = input
        .documents
        .iter()
        .filter(|d| d.role == DocumentRoleDto::VendorQuote)
        .collect();
    if vendor_docs.is_empty() {
        return Err(BridgeError::MissingVendorQuote);
    }

    // 1. Build or extract MasterBoq (supports Synthetic BOQ when no Master BOQ file is uploaded!)
    let master_boq_doc = input
        .documents
        .iter()
        .find(|d| d.role == DocumentRoleDto::MasterBoq);

    let master_boq: MasterBoq = if let Some(m_doc) = master_boq_doc {
        let parsed = load_parsed_doc(m_doc)?.ok_or_else(|| BridgeError::FileCorrupted {
            file_name: m_doc.file_name.clone(),
            reason: "Master BOQ has no readable table".to_owned(),
        })?;
        let first_sheet = parsed
            .sheets
            .first()
            .ok_or_else(|| BridgeError::FileCorrupted {
                file_name: m_doc.file_name.clone(),
                reason: "Master BOQ must contain tabular rows".to_owned(),
            })?;
        extract_master_boq_from_sheet(&m_doc.file_name, input.config.currency, first_sheet)?
    } else {
        // Synthesize baseline from tabular vendor quotes (`orphan_vendor_rows == 0`)
        let mut candidate_lists = Vec::new();
        for (v_idx, v_doc) in vendor_docs.iter().enumerate() {
            if let Some(parsed) = load_parsed_doc(v_doc)? {
                if let Some(sheet) = parsed.sheets.first() {
                    if let Ok(extracted) =
                        extract_master_boq_from_sheet(&v_doc.file_name, input.config.currency, sheet)
                    {
                        let cands: Vec<RawVendorCandidateItem> = extracted
                            .items
                            .into_iter()
                            .enumerate()
                            .map(|(r_i, item)| RawVendorCandidateItem {
                                vendor_id: format!("V-{}", v_idx + 1),
                                row_index: r_i,
                                code: Some(item.code),
                                description: item.description,
                                unit: item.unit,
                                quantity: item.quantity,
                            })
                            .collect();
                        candidate_lists.push(cands);
                    }
                }
            }
        }
        build_synthetic_master_boq(
            "Synthesized Unified BOQ",
            input.config.currency,
            &candidate_lists,
        )?
        .master_boq
    };

    // 2. Prepare vendor extraction payloads (merging cached rasterized PDF images)
    let rendered_cache = state.rendered_pdf_cache.lock().await;
    let mut prepared_vendors = Vec::with_capacity(vendor_docs.len());

    for (idx, v_doc) in vendor_docs.iter().enumerate() {
        let vendor_id = format!("V-{}", idx + 1);
        let vendor_name = v_doc
            .vendor_name
            .clone()
            .filter(|n| !n.trim().is_empty())
            .unwrap_or_else(|| format!("Vendor {}", idx + 1));

        let parsed_opt = load_parsed_doc(v_doc)?;
        let text_batches = parsed_opt
            .as_ref()
            .map(format_parsed_doc_into_batches)
            .unwrap_or_default();
        let blake3_hex = parsed_opt
            .as_ref()
            .map(|p| p.blake3_hex.clone())
            .unwrap_or_else(|| blake3::hash(v_doc.file_name.as_bytes()).to_hex().to_string());

        let mut images = Vec::new();
        if let Some(img_b64) = &v_doc.pasted_image_base64 {
            images.push(ImageAttachment {
                page_number: 1,
                mime_type: "image/png".to_owned(),
                base64_data: img_b64.clone(),
            });
        }
        let pages_src = if !v_doc.rendered_pages.is_empty() {
            Some(&v_doc.rendered_pages)
        } else {
            rendered_cache.get(&v_doc.id)
        };
        if let Some(pages) = pages_src {
            for p in pages {
                images.push(ImageAttachment {
                    page_number: p.page_number,
                    mime_type: p.mime_type.clone(),
                    base64_data: p.base64_data.clone(),
                });
            }
        }

        prepared_vendors.push(PreparedVendorInput {
            vendor_id,
            vendor_name,
            document_blake3: blake3_hex,
            text_batches,
            images,
            has_hidden_rows: v_doc.hidden_rows_count > 0,
        });
    }
    drop(rendered_cache);

    // 3. Resolve provider client & live Multimodal models
    let (resolved_provider, cred) = state.vault.resolve_credential(input.config.ai.provider)?;
    let provider_client =
        build_provider_client(resolved_provider, cred, state.http_client.clone())?;
    let available_models = provider_client.list_multimodal_models().await?;

    let session_id_progress = session_id.clone();
    let (quotes, mut groups, ai_traps) = run_extraction_and_leveling_pipeline(
        &input.config,
        &master_boq,
        &prepared_vendors,
        provider_client.as_ref(),
        &available_models,
        &state.checkpoints,
        &cancel_token,
        |upd| {
            let pct = if upd.total_steps == 0 {
                50
            } else {
                15 + ((upd.completed_steps * 65) / upd.total_steps).min(65) as u8
            };
            emitter.emit_progress(&LevelingProgressEvent {
                session_id: session_id_progress.clone(),
                stage: ProgressStageDto::ExtractingVendorQuotes,
                completed_steps: upd.completed_steps,
                total_steps: upd.total_steps,
                active_vendor_name: upd.active_vendor_name,
                percent: pct,
                checkpoint_hit: upd.checkpoint_hit,
            });
        },
    )
    .await?;

    // 4. InteractiveReview merged-item loop if enabled
    if input.config.merged_item_mode == MergedItemMode::InteractiveReview {
        let total_pending = groups
            .iter()
            .filter(|g| g.resolution == AlignmentResolution::PendingReview)
            .count();
        let mut resolved_so_far = 0_usize;

        for grp in &mut groups {
            if grp.resolution != AlignmentResolution::PendingReview {
                continue;
            }
            let (tx, rx) = oneshot::channel();
            {
                let mut map = state.pending_ambiguity_tx.lock().await;
                map.insert((session_id.clone(), grp.group_id.clone()), tx);
            }

            let v_name = quotes
                .iter()
                .find(|q| q.vendor_id == grp.vendor_id)
                .map(|q| q.vendor_name.clone())
                .unwrap_or_else(|| grp.vendor_id.clone());

            let master_descs: Vec<String> = grp
                .master_boq_ids
                .iter()
                .filter_map(|id| {
                    master_boq
                        .items
                        .iter()
                        .find(|b| &b.id == id)
                        .map(|b| format!("{} - {}", b.code, b.description))
                })
                .collect();

            emitter.emit_ambiguity_prompt(&AmbiguityPromptPayload {
                session_id: session_id.clone(),
                group: grp.clone(),
                vendor_name: v_name,
                master_descriptions: master_descs,
                pending_count: total_pending.saturating_sub(resolved_so_far),
            });

            let decision = tokio::select! {
                () = cancel_token.cancelled() => return Err(BridgeError::Cancelled),
                res = rx => res.map_err(|_| BridgeError::Cancelled)?,
            };

            grp.resolution = match decision.choice {
                AmbiguityResolutionChoice::AnchorAndZeroFollowers => {
                    AlignmentResolution::AnchorAndZeroFollowers
                }
                AmbiguityResolutionChoice::ProportionalSplit => {
                    AlignmentResolution::ProportionalSplit
                }
                AmbiguityResolutionChoice::AcceptedAlternate => {
                    AlignmentResolution::AcceptedAlternate
                }
                AmbiguityResolutionChoice::ExcludedWithPlug => {
                    AlignmentResolution::ExcludedWithPlug
                }
            };
            grp.reviewer_note = decision.reviewer_note;
            resolved_so_far += 1;
        }
    }

    emitter.emit_progress(&LevelingProgressEvent {
        session_id: session_id.clone(),
        stage: ProgressStageDto::AuditingMathAndTraps,
        completed_steps: 1,
        total_steps: 1,
        active_vendor_name: None,
        percent: 90,
        checkpoint_hit: false,
    });

    // 5. Deterministic Commercial & Technical Leveling
    let result: LevelingResult = evaluate_quotes_deterministic(
        &input.config,
        &master_boq,
        quotes,
        Some(groups),
        &ai_traps,
    )?;

    let summary_dto = map_result_to_dto(&result);

    {
        let mut store = state.completed_results.lock().await;
        store.insert(session_id.clone(), result);
    }
    {
        let mut cancels = state.active_cancellations.lock().await;
        cancels.remove(&session_id);
    }

    emitter.emit_progress(&LevelingProgressEvent {
        session_id,
        stage: ProgressStageDto::Completed,
        completed_steps: 1,
        total_steps: 1,
        active_vendor_name: None,
        percent: 100,
        checkpoint_hit: false,
    });

    Ok(summary_dto)
}

fn map_result_to_dto(result: &LevelingResult) -> LevelingSessionResultDto {
    let mut sorted = result.vendor_summaries.clone();
    sorted.sort_by_key(|v| v.rank);

    let vendor_name_by_id: std::collections::HashMap<&str, &str> = result
        .vendor_summaries
        .iter()
        .map(|s| (s.vendor_id.as_str(), s.vendor_name.as_str()))
        .collect();

    let vendors = sorted
        .into_iter()
        .map(|s| {
            let total_plugs = s
                .line_scope_plugs_total
                .checked_add(s.clause_plugs_total)
                .unwrap_or(DecimalValue::ZERO);
            VendorSummaryRowDto {
                vendor_id: s.vendor_id,
                vendor_name: s.vendor_name,
                submitted_total: s.as_submitted_total,
                arithmetic_delta: s.arithmetic_and_qty_delta,
                total_plugs,
                leveled_net_total: s.leveled_net_total,
                leveled_gross_total: s.leveled_gross_total,
                variance_vs_lowest_pct: s.variance_vs_lowest_pct,
                traps_count: s.traps_count,
                rank: s.rank,
                is_true_low_bidder: s.is_true_low_bidder,
            }
        })
        .collect();

    let traps = result
        .traps
        .iter()
        .map(|t| TrapFindingSummaryDto {
            category: t.category,
            vendor_name: vendor_name_by_id
                .get(t.vendor_id.as_str())
                .copied()
                .unwrap_or(t.vendor_id.as_str())
                .to_owned(),
            title: t.title.clone(),
            detail: t.detail.clone(),
            source_snippet: t.source_snippet.clone(),
            financial_impact: t.computed_impact,
        })
        .collect();

    let default_dir = dirs::document_dir()
        .or_else(dirs::download_dir)
        .unwrap_or_else(|| PathBuf::from("."));
    let suggested_output_path = default_dir
        .join(format!("qlevel_Bid_Comparison_{}.xlsx", result.session_id))
        .display()
        .to_string();

    LevelingSessionResultDto {
        session_id: result.session_id.clone(),
        currency: result.config.currency,
        vat_pct: result.config.vat_policy.project_vat_pct,
        is_synthetic_boq: result.master_boq.is_synthetic,
        total_boq_items: result.master_boq.items.len(),
        merged_groups_count: result.alignment_groups.len(),
        true_low_vendor_name: result.true_low_vendor_name.clone(),
        true_low_leveled_net: result.true_low_leveled_net,
        layout_mode: result.config.layout_mode,
        vendors,
        traps,
        suggested_output_path,
    }
}

/// Cancels an active leveling session cooperatively via its `CancellationToken`.
pub async fn handle_cancel_leveling_session(
    state: &BridgeState,
    session_id: &str,
) -> BridgeResult<()> {
    let guard = state.active_cancellations.lock().await;
    if let Some(tok) = guard.get(session_id) {
        tok.cancel();
    }
    Ok(())
}

/// Delivers an interactive merged-item resolution decision to the waiting session pipeline.
pub async fn handle_resolve_ambiguity_prompt(
    state: &BridgeState,
    decision: AmbiguityDecisionInput,
) -> BridgeResult<()> {
    let tx_opt = {
        let mut guard = state.pending_ambiguity_tx.lock().await;
        guard.remove(&(decision.session_id.clone(), decision.group_id.clone()))
    };
    if let Some(tx) = tx_opt {
        let _ = tx.send(decision);
    }
    Ok(())
}

/// Exports the session's live-formula `.xlsx` workbook to disk with Excel lock detection.
pub async fn handle_export_workbook(
    state: &BridgeState,
    input: ExportWorkbookInput,
) -> BridgeResult<WorkbookExportOutcomeDto> {
    let result_clone = {
        let guard = state.completed_results.lock().await;
        guard
            .get(&input.session_id)
            .cloned()
            .ok_or_else(|| BridgeError::Internal("Completed session result not found".to_owned()))?
    };

    let outcome = save_leveling_workbook_to_path(
        &result_clone,
        Path::new(&input.target_path),
        input.use_fallback_if_locked,
    )?;

    match outcome {
        WorkbookWriteOutcome::Saved { output_path } => Ok(WorkbookExportOutcomeDto::Saved {
            output_path: output_path.display().to_string(),
        }),
        WorkbookWriteOutcome::FileLocked {
            locked_path,
            suggested_fallback_path,
            hint,
        } => Ok(WorkbookExportOutcomeDto::FileLocked {
            locked_path: locked_path.display().to_string(),
            suggested_fallback_path: suggested_fallback_path.display().to_string(),
            hint,
        }),
    }
}
