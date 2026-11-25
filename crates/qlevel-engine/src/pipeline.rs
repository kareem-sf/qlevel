//! Session orchestrator combining resumable `SQLite WAL` extraction, `CancellationToken`,
//! merged-row resolution, TBE trap auditing, and deterministic `Decimal` commercial leveling.

use qlevel_ai::{
    ExtractedVendorBatch, ExtractionBatchInput, ImageAttachment, ProviderClient,
};
use qlevel_domain::{
    AlignmentGroup, AlignmentResolution, CellStatus, CommercialTerms, DecimalValue, DetectedTrap,
    LevelingResult, LevelingSessionConfig, MasterBoq, MergedItemMode, ModelInfo, PlugBasis,
    VendorLineQuote, VendorQuote,
};
use tokio_util::sync::CancellationToken;

use crate::auto_router::{route_batch_model_and_effort, BatchComplexitySignals};
use crate::checkpoint_db::{compute_batch_blake3_key, CheckpointStore};
use crate::commercial_calc::evaluate_commercial_leveling;
use crate::error::{EngineError, EngineResult};
use crate::merged_resolver::{apply_merged_group_resolutions, build_initial_alignment_groups};
use crate::technical_eval::evaluate_technical_traps;

/// Pipeline lifecycle phases broadcast to the desktop UI progress bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelinePhase {
    ExtractingVendorBatches,
    ResolvingMergedItems,
    AuditingTechnicalTraps,
    ComputingCommercialEqualization,
    Completed,
}

/// Real-time progress snapshot emitted during a leveling run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineProgressUpdate {
    pub phase: PipelinePhase,
    pub completed_steps: usize,
    pub total_steps: usize,
    pub active_vendor_name: Option<String>,
    pub checkpoint_hit: bool,
}

/// Prepared vendor input payload ready for AI batch extraction or checkpoint replay.
#[derive(Debug, Clone)]
pub struct PreparedVendorInput {
    pub vendor_id: String,
    pub vendor_name: String,
    pub document_blake3: String,
    pub text_batches: Vec<String>,
    pub images: Vec<ImageAttachment>,
    pub has_hidden_rows: bool,
}

fn convert_extracted_batches_to_quote(
    vendor_id: &str,
    vendor_name: &str,
    config: &LevelingSessionConfig,
    batches: &[ExtractedVendorBatch],
) -> (VendorQuote, Vec<DetectedTrap>) {
    let mut lines = Vec::new();
    let mut ai_traps = Vec::new();
    let mut raw_exclusions = Vec::new();
    let mut default_vat = config.vat_policy.default_vendor_mode;
    let mut terms = CommercialTerms::default();

    for (b_idx, batch) in batches.iter().enumerate() {
        if let Some(vm) = batch.default_vat_mode {
            default_vat = vm;
        }
        if let Some(p) = &batch.payment_terms {
            terms.payment_terms = Some(p.clone());
        }
        if let Some(inc) = &batch.delivery_incoterms {
            terms.delivery_incoterms = Some(inc.clone());
        }
        if let Some(w) = batch.delivery_weeks {
            terms.delivery_weeks = Some(w);
        }
        if let Some(val) = batch.validity_days {
            terms.validity_days = Some(val);
        }
        if let Some(war) = batch.warranty_months {
            terms.warranty_months = Some(war);
        }
        if let Some(disc_pct) = batch
            .general_discount_pct_raw
            .as_deref()
            .and_then(|s| DecimalValue::parse_clean(s).ok())
        {
            terms.general_discount_pct = disc_pct;
        }
        if let Some(disc_ls) = batch
            .lump_sum_discount_raw
            .as_deref()
            .and_then(|s| DecimalValue::parse_clean(s).ok())
        {
            terms.lump_sum_discount = disc_ls;
        }

        for item in &batch.lines {
            let Some(anchor_id) = item.matched_master_boq_ids.first() else {
                continue;
            };
            let qty = DecimalValue::parse_clean(&item.quoted_qty_raw).unwrap_or(DecimalValue::ZERO);
            let rate =
                DecimalValue::parse_clean(&item.quoted_unit_rate_raw).unwrap_or(DecimalValue::ZERO);
            let ext = DecimalValue::parse_clean(&item.quoted_extended_total_raw)
                .unwrap_or(DecimalValue::ZERO);

            let status = if item.is_unquoted_or_excluded {
                CellStatus::Unquoted
            } else if item.is_moved_to_optional {
                CellStatus::MovedToOptional
            } else {
                CellStatus::Quoted
            };

            lines.push(VendorLineQuote {
                boq_item_id: anchor_id.clone(),
                vendor_code: item.vendor_code.clone(),
                vendor_description: item.vendor_description.clone(),
                quoted_unit: item.quoted_unit.clone(),
                quoted_qty: qty,
                quoted_unit_rate: rate,
                quoted_extended_total: ext,
                vat_mode: item.vat_mode,
                vendor_vat_pct: None,
                status,
                proposed_brand: item.proposed_brand.clone(),
                source_page: item.source_page,
                notes: item.note.clone(),
            });

            // Emit follower IncludedIn rows when AI matched multiple Master BOQ IDs to one lump sum
            for follower_id in item.matched_master_boq_ids.iter().skip(1) {
                lines.push(VendorLineQuote {
                    boq_item_id: follower_id.clone(),
                    vendor_code: item.vendor_code.clone(),
                    vendor_description: item.vendor_description.clone(),
                    quoted_unit: item.quoted_unit.clone(),
                    quoted_qty: DecimalValue::ZERO,
                    quoted_unit_rate: DecimalValue::ZERO,
                    quoted_extended_total: DecimalValue::ZERO,
                    vat_mode: item.vat_mode,
                    vendor_vat_pct: None,
                    status: CellStatus::IncludedIn {
                        anchor_boq_id: anchor_id.clone(),
                    },
                    proposed_brand: item.proposed_brand.clone(),
                    source_page: item.source_page,
                    notes: Some(format!("Merged with {anchor_id}")),
                });
            }
        }

        for (t_idx, t) in batch.traps.iter().enumerate() {
            raw_exclusions.push(t.verbatim_quote.clone());
            let plug_basis = if let Some(pct_str) = &t.suggested_plug_pct_raw {
                DecimalValue::parse_clean(pct_str)
                    .ok()
                    .filter(|v| v.is_positive())
                    .map(|pct| PlugBasis::PctOfQuotedOnly { pct })
            } else if let Some(ls_str) = &t.suggested_plug_lump_sum_raw {
                DecimalValue::parse_clean(ls_str)
                    .ok()
                    .filter(|v| v.is_positive())
                    .map(|amount| PlugBasis::FixedSetupFee { amount })
            } else {
                t.related_boq_id
                    .as_ref()
                    .map(|boq_id| PlugBasis::AnchoredToBoq {
                        boq_id: boq_id.clone(),
                    })
            };

            ai_traps.push(DetectedTrap {
                id: format!("TRAP-AI-{vendor_id}-{b_idx}-{t_idx}"),
                vendor_id: vendor_id.to_owned(),
                category: t.category,
                severity: t.severity,
                related_boq_id: t.related_boq_id.clone(),
                title: t.title.clone(),
                detail: t.detail.clone(),
                source_snippet: t.verbatim_quote.clone(),
                source_page: t.source_page,
                plug_basis,
                computed_impact: None,
            });
        }
    }

    let quote = VendorQuote {
        vendor_id: vendor_id.to_owned(),
        vendor_name: vendor_name.to_owned(),
        currency: config.currency,
        fx_rate_to_project: DecimalValue::ONE,
        default_vat_mode: default_vat,
        stated_grand_total: None,
        terms,
        lines,
        raw_exclusion_clauses: raw_exclusions,
    };

    (quote, ai_traps)
}

/// Pure deterministic pipeline entry point for pre-extracted or golden-fixture `VendorQuote` sets.
///
/// # Errors
/// Returns [`EngineError`] on domain validation failure or unresolved interactive review groups.
pub fn evaluate_quotes_deterministic(
    config: &LevelingSessionConfig,
    master_boq: &MasterBoq,
    mut quotes: Vec<VendorQuote>,
    override_groups: Option<Vec<AlignmentGroup>>,
    seed_traps: &[DetectedTrap],
) -> EngineResult<LevelingResult> {
    let groups = override_groups
        .unwrap_or_else(|| build_initial_alignment_groups(&quotes, config.merged_item_mode));

    if config.merged_item_mode == MergedItemMode::InteractiveReview {
        if let Some(pending) = groups
            .iter()
            .find(|g| g.resolution == AlignmentResolution::PendingReview)
        {
            return Err(EngineError::PendingInteractiveResolution {
                group_id: pending.group_id.clone(),
            });
        }
    }

    apply_merged_group_resolutions(master_boq, &mut quotes, &groups)?;
    let audited_traps = evaluate_technical_traps(config, &quotes, seed_traps);
    evaluate_commercial_leveling(config, master_boq, &quotes, &groups, audited_traps)
}

/// Full asynchronous extraction + checkpointing + deterministic leveling orchestrator.
///
/// # Errors
/// Returns [`EngineError::Cancelled`] immediately if `cancel_token` is triggered, or
/// upstream [`EngineError`] on provider/checkpoint/validation errors.
pub async fn run_extraction_and_leveling_pipeline<F>(
    config: &LevelingSessionConfig,
    master_boq: &MasterBoq,
    vendors: &[PreparedVendorInput],
    provider_client: &dyn ProviderClient,
    available_models: &[ModelInfo],
    checkpoint_store: &CheckpointStore,
    cancel_token: &CancellationToken,
    mut on_progress: F,
) -> EngineResult<(Vec<VendorQuote>, Vec<AlignmentGroup>, Vec<DetectedTrap>)>
where
    F: FnMut(EngineProgressUpdate) + Send,
{
    if vendors.is_empty() {
        return Err(EngineError::NoVendorQuotes);
    }

    let boq_json = serde_json::to_string(&master_boq.items)
        .map_err(|err| EngineError::CheckpointDb(err.to_string()))?;
    let boq_fingerprint = blake3::hash(boq_json.as_bytes()).to_hex().to_string();

    let total_batches: usize = vendors.iter().map(|v| v.text_batches.len().max(1)).sum();
    let mut completed_batches = 0_usize;

    let mut extracted_quotes = Vec::with_capacity(vendors.len());
    let mut all_ai_traps = Vec::new();
    let provider_label = match provider_client.provider_kind() {
        qlevel_domain::ProviderKind::OpenAi => "openai",
        qlevel_domain::ProviderKind::Gemini => "gemini",
        qlevel_domain::ProviderKind::Claude => "claude",
    };

    for vendor in vendors {
        if cancel_token.is_cancelled() {
            return Err(EngineError::Cancelled);
        }

        let effective_chunks: Vec<String> = if vendor.text_batches.is_empty() {
            vec!["[Scanned image quotation]".to_owned()]
        } else {
            vendor.text_batches.clone()
        };

        let mut vendor_batches_out = Vec::with_capacity(effective_chunks.len());

        for (batch_idx, chunk_text) in effective_chunks.iter().enumerate() {
            if cancel_token.is_cancelled() {
                return Err(EngineError::Cancelled);
            }

            let lower_chunk = chunk_text.to_lowercase();
            let signals = BatchComplexitySignals {
                has_scanned_images: !vendor.images.is_empty() && batch_idx == 0,
                has_exclusion_or_terms_text: lower_chunk.contains("exclu")
                    || lower_chunk.contains("terms")
                    || lower_chunk.contains("استثناء")
                    || lower_chunk.contains("شروط"),
                has_hidden_excel_rows: vendor.has_hidden_rows,
                lacks_master_boq: master_boq.is_synthetic,
            };

            let routed = route_batch_model_and_effort(&config.ai, available_models, signals)?;
            let ckpt_key = compute_batch_blake3_key(
                provider_label,
                &routed.model_id,
                routed.effort,
                &vendor.document_blake3,
                batch_idx as u32,
                &boq_fingerprint,
                &config.custom_rules,
            );

            let (batch_result, was_cached) = if let Some(cached) = checkpoint_store.get_batch(&ckpt_key)? {
                (cached, true)
            } else {
                let batch_input = ExtractionBatchInput {
                    batch_id: format!("{}-{}", vendor.vendor_id, batch_idx),
                    vendor_name: vendor.vendor_name.clone(),
                    master_boq_context_json: boq_json.clone(),
                    vendor_raw_text_or_rows: chunk_text.clone(),
                    images: if batch_idx == 0 {
                        vendor.images.clone()
                    } else {
                        Vec::new()
                    },
                    approved_vendors: config.approved_vendors.clone(),
                    custom_rules: config.custom_rules.clone(),
                };

                let fresh = tokio::select! {
                    () = cancel_token.cancelled() => {
                        return Err(EngineError::Cancelled);
                    }
                    res = provider_client.extract_vendor_batch(&routed.model_id, routed.effort, &batch_input) => {
                        res?
                    }
                };
                checkpoint_store.save_batch(
                    &ckpt_key,
                    &config.session_id,
                    &vendor.vendor_name,
                    &fresh,
                )?;
                (fresh, false)
            };

            completed_batches += 1;
            on_progress(EngineProgressUpdate {
                phase: PipelinePhase::ExtractingVendorBatches,
                completed_steps: completed_batches,
                total_steps: total_batches,
                active_vendor_name: Some(vendor.vendor_name.clone()),
                checkpoint_hit: was_cached,
            });

            vendor_batches_out.push(batch_result);
        }

        let (quote, traps) = convert_extracted_batches_to_quote(
            &vendor.vendor_id,
            &vendor.vendor_name,
            config,
            &vendor_batches_out,
        );
        extracted_quotes.push(quote);
        all_ai_traps.extend(traps);
    }

    let groups = build_initial_alignment_groups(&extracted_quotes, config.merged_item_mode);
    Ok((extracted_quotes, groups, all_ai_traps))
}
