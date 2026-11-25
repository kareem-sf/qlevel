//! Deterministic fixed-point `Decimal` Commercial Bid Leveling (`CBE`) calculator.
//!
//! Executes:
//! 1. FX normalization and VAT stripping to Net VAT-Exclusive baseline
//! 2. Vendor arithmetic (`Qty * Rate != Extended`) and quantity-tamper detection
//! 3. RICS NRM 2 `ProvisionalSum` locking (overrides vendor under-pricing or discounts)
//! 4. Peer missing-scope plugs (`LowestCompliant`, `AverageCompliant`, `HighestCompliant`, `PteEstimate`)
//! 5. Clause-level `PlugBasis` evaluation with strict **Zero Double-Counting** (`AnchoredToBoq` dedup)
//! 6. Tender discount proration on `Measured` work only, statutory VAT application, and ranking.

use std::collections::{HashMap, HashSet};

use qlevel_domain::{
    AlignmentGroup, BoqItem, CellStatus, DecimalValue, DetectedTrap, DomainError, LeveledLineCell,
    LevelingResult, LevelingSessionConfig, LineItemKind, MasterBoq, PeerPlugStrategy, PlugBasis,
    TrapCategory, TrapSeverity, VendorEvaluationSummary, VendorLineQuote, VendorQuote,
};

use crate::error::{EngineError, EngineResult};

struct NormalizedVendorCell {
    status: CellStatus,
    net_unit_rate: DecimalValue,
    as_submitted_net_total: DecimalValue,
    normalized_measured_net: DecimalValue,
    arithmetic_delta: DecimalValue,
    highlight_yellow: bool,
    audit_note: Option<String>,
}

fn convert_fx_and_strip_vat(
    amount: DecimalValue,
    quote: &VendorQuote,
    line: Option<&VendorLineQuote>,
    config: &LevelingSessionConfig,
) -> EngineResult<DecimalValue> {
    if !quote.fx_rate_to_project.is_positive() {
        return Err(EngineError::Domain(DomainError::InvalidFxRate {
            vendor_currency: quote.currency.as_str().to_owned(),
            project_currency: config.currency.as_str().to_owned(),
        }));
    }
    let fx_converted = if quote.currency == config.currency {
        amount
    } else {
        amount.checked_mul(quote.fx_rate_to_project)?
    };

    let mode = line.map_or(quote.default_vat_mode, |l| l.vat_mode);
    let override_pct = line.and_then(|l| l.vendor_vat_pct);
    Ok(config
        .vat_policy
        .strip_to_net_exclusive(fx_converted, mode, override_pct)?)
}

fn compute_peer_plug(
    strategy: PeerPlugStrategy,
    item: &BoqItem,
    peer_net_totals: &[DecimalValue],
    scale: u32,
) -> EngineResult<DecimalValue> {
    let pte_fallback = item
        .pte_extended_total(scale)?
        .unwrap_or(DecimalValue::ZERO);

    if peer_net_totals.is_empty() {
        return Ok(pte_fallback);
    }

    match strategy {
        PeerPlugStrategy::LowestCompliant => Ok(peer_net_totals
            .iter()
            .copied()
            .min()
            .unwrap_or(pte_fallback)
            .round_commercial(scale)),
        PeerPlugStrategy::HighestCompliant => Ok(peer_net_totals
            .iter()
            .copied()
            .max()
            .unwrap_or(pte_fallback)
            .round_commercial(scale)),
        PeerPlugStrategy::AverageCompliant => {
            let mut sum = DecimalValue::ZERO;
            for val in peer_net_totals {
                sum = sum.checked_add(*val)?;
            }
            let count = DecimalValue::from_u64(peer_net_totals.len() as u64);
            Ok(sum.checked_div(count)?.round_commercial(scale))
        }
        PeerPlugStrategy::PteEstimate => {
            if pte_fallback.is_positive() {
                Ok(pte_fallback)
            } else {
                Ok(peer_net_totals
                    .iter()
                    .copied()
                    .min()
                    .unwrap_or(DecimalValue::ZERO)
                    .round_commercial(scale))
            }
        }
    }
}

/// Evaluates full commercial equalization across `master_boq`, `quotes`, `alignment_groups`, and `traps`.
///
/// # Errors
/// Returns [`EngineError`] if inputs violate domain invariants or have zero vendors.
pub fn evaluate_commercial_leveling(
    config: &LevelingSessionConfig,
    master_boq: &MasterBoq,
    quotes: &[VendorQuote],
    alignment_groups: &[AlignmentGroup],
    mut traps: Vec<DetectedTrap>,
) -> EngineResult<LevelingResult> {
    master_boq.validate()?;
    config.vat_policy.validate()?;
    if quotes.is_empty() {
        return Err(EngineError::NoVendorQuotes);
    }

    let scale = config.currency.decimal_scale();
    let tolerance = config.currency.minor_unit_tolerance();

    // Track merged anchor items so YellowHighlightAuto marks them amber in Excel
    let mut yellow_anchors: HashSet<(String, String)> = HashSet::new();
    for grp in alignment_groups {
        if let Some(first_id) = grp.master_boq_ids.first() {
            yellow_anchors.insert((grp.vendor_id.clone(), first_id.clone()));
        }
    }

    // Pass 1: Normalize every (BoqItem, VendorQuote) cell & detect arithmetic / qty traps
    let mut cell_grid: HashMap<(String, String), NormalizedVendorCell> = HashMap::new();

    for item in &master_boq.items {
        let fixed_ps_allowance = if matches!(item.kind, LineItemKind::ProvisionalSum) {
            item.pte_extended_total(scale)?
                .unwrap_or(item.quantity.round_commercial(scale))
        } else {
            DecimalValue::ZERO
        };

        for quote in quotes {
            let line_opt = quote.lines.iter().find(|l| l.boq_item_id == item.id);
            let is_yellow_anchor = yellow_anchors.contains(&(quote.vendor_id.clone(), item.id.clone()));

            let norm_cell = match (&item.kind, line_opt) {
                (LineItemKind::ProvisionalSum, maybe_line) => {
                    let submitted = match maybe_line {
                        Some(l) if matches!(l.status, CellStatus::Quoted) => {
                            convert_fx_and_strip_vat(l.quoted_extended_total, quote, Some(l), config)?
                                .round_commercial(scale)
                        }
                        _ => DecimalValue::ZERO,
                    };
                    let delta = fixed_ps_allowance.checked_sub(submitted)?;
                    let tampered = delta.abs() > tolerance;
                    NormalizedVendorCell {
                        status: if tampered {
                            CellStatus::UnderPricedAllowance
                        } else {
                            CellStatus::Quoted
                        },
                        net_unit_rate: fixed_ps_allowance,
                        as_submitted_net_total: submitted,
                        normalized_measured_net: fixed_ps_allowance,
                        arithmetic_delta: delta,
                        highlight_yellow: tampered,
                        audit_note: if tampered {
                            Some("Provisional Sum equalized to fixed employer allowance".to_owned())
                        } else {
                            None
                        },
                    }
                }
                (LineItemKind::RateOnly | LineItemKind::Optional | LineItemKind::Alternative { .. }, maybe_line) => {
                    let net_rate = match maybe_line {
                        Some(l) => convert_fx_and_strip_vat(l.quoted_unit_rate, quote, Some(l), config)?
                            .round_commercial(scale + 2),
                        None => DecimalValue::ZERO,
                    };
                    NormalizedVendorCell {
                        status: maybe_line.map_or(CellStatus::Unquoted, |l| l.status.clone()),
                        net_unit_rate: net_rate,
                        as_submitted_net_total: DecimalValue::ZERO,
                        normalized_measured_net: DecimalValue::ZERO,
                        arithmetic_delta: DecimalValue::ZERO,
                        highlight_yellow: false,
                        audit_note: Some("Memorandum item excluded from base measured sum".to_owned()),
                    }
                }
                (LineItemKind::Measured, Some(line)) => match &line.status {
                    CellStatus::IncludedIn { anchor_boq_id } => NormalizedVendorCell {
                        status: CellStatus::IncludedIn {
                            anchor_boq_id: anchor_boq_id.clone(),
                        },
                        net_unit_rate: DecimalValue::ZERO,
                        as_submitted_net_total: DecimalValue::ZERO,
                        normalized_measured_net: DecimalValue::ZERO,
                        arithmetic_delta: DecimalValue::ZERO,
                        highlight_yellow: true,
                        audit_note: Some(format!("Included in {anchor_boq_id}")),
                    },
                    CellStatus::Unquoted | CellStatus::MovedToOptional => NormalizedVendorCell {
                        status: line.status.clone(),
                        net_unit_rate: DecimalValue::ZERO,
                        as_submitted_net_total: DecimalValue::ZERO,
                        normalized_measured_net: DecimalValue::ZERO,
                        arithmetic_delta: DecimalValue::ZERO,
                        highlight_yellow: false,
                        audit_note: line.notes.clone(),
                    },
                    CellStatus::Quoted | CellStatus::UnderPricedAllowance => {
                        let net_rate =
                            convert_fx_and_strip_vat(line.quoted_unit_rate, quote, Some(line), config)?;
                        let submitted_net =
                            convert_fx_and_strip_vat(line.quoted_extended_total, quote, Some(line), config)?
                                .round_commercial(scale);

                        // If anchor of a merged lump-sum row with zero unit rate, preserve lump sum
                        let leveled_base = if is_yellow_anchor && net_rate.is_zero() && submitted_net.is_positive() {
                            submitted_net
                        } else {
                            item.quantity.checked_mul(net_rate)?.round_commercial(scale)
                        };

                        // Check vendor's own multiplication (quoted_qty * net_rate vs submitted_net)
                        if !is_yellow_anchor && line.quoted_qty.is_positive() && net_rate.is_positive() {
                            let vendor_math = line.quoted_qty.checked_mul(net_rate)?.round_commercial(scale);
                            let math_diff = vendor_math.checked_sub(submitted_net)?;
                            if math_diff.abs() > tolerance {
                                traps.push(DetectedTrap {
                                    id: format!("TRAP-MATH-{}-{}", quote.vendor_id, item.id),
                                    vendor_id: quote.vendor_id.clone(),
                                    category: TrapCategory::QtyArithmetic,
                                    severity: TrapSeverity::High,
                                    related_boq_id: Some(item.id.clone()),
                                    title: format!("Arithmetic multiplication error on {}", item.code),
                                    detail: format!(
                                        "Vendor printed {} instead of {} ({} * {})",
                                        submitted_net.to_fixed_string(scale),
                                        vendor_math.to_fixed_string(scale),
                                        line.quoted_qty,
                                        net_rate.round_commercial(scale)
                                    ),
                                    source_snippet: line.vendor_description.clone(),
                                    source_page: line.source_page,
                                    plug_basis: None,
                                    computed_impact: Some(leveled_base.checked_sub(submitted_net)?),
                                });
                            }
                        }

                        // Check quantity tampering (quoted_qty != item.quantity)
                        if !is_yellow_anchor
                            && line.quoted_qty.is_positive()
                            && (line.quoted_qty.checked_sub(item.quantity)?).abs() > tolerance
                        {
                            traps.push(DetectedTrap {
                                id: format!("TRAP-QTY-{}-{}", quote.vendor_id, item.id),
                                vendor_id: quote.vendor_id.clone(),
                                category: TrapCategory::QtyArithmetic,
                                severity: TrapSeverity::High,
                                related_boq_id: Some(item.id.clone()),
                                title: format!("Quantity deviation on {}", item.code),
                                detail: format!(
                                    "Vendor priced quantity {} instead of Master BOQ quantity {}",
                                    line.quoted_qty, item.quantity
                                ),
                                source_snippet: line.vendor_description.clone(),
                                source_page: line.source_page,
                                plug_basis: None,
                                computed_impact: Some(leveled_base.checked_sub(submitted_net)?),
                            });
                        }

                        NormalizedVendorCell {
                            status: CellStatus::Quoted,
                            net_unit_rate: net_rate.round_commercial(scale + 2),
                            as_submitted_net_total: submitted_net,
                            normalized_measured_net: leveled_base,
                            arithmetic_delta: leveled_base.checked_sub(submitted_net)?,
                            highlight_yellow: is_yellow_anchor,
                            audit_note: if is_yellow_anchor {
                                Some("Merged vendor lump-sum anchor; verify constituent items".to_owned())
                            } else {
                                line.notes.clone()
                            },
                        }
                    }
                },
                (LineItemKind::Measured, None) => NormalizedVendorCell {
                    status: CellStatus::Unquoted,
                    net_unit_rate: DecimalValue::ZERO,
                    as_submitted_net_total: DecimalValue::ZERO,
                    normalized_measured_net: DecimalValue::ZERO,
                    arithmetic_delta: DecimalValue::ZERO,
                    highlight_yellow: false,
                    audit_note: Some("Omitted by vendor; peer equalization plug applied".to_owned()),
                },
            };

            cell_grid.insert((item.id.clone(), quote.vendor_id.clone()), norm_cell);
        }
    }

    // Pass 2: Compute peer plugs for unquoted Measured items (excluding merged anchors from peer pool)
    let mut finalized_cells: Vec<LeveledLineCell> =
        Vec::with_capacity(master_boq.items.len() * quotes.len());
    let mut plugged_boq_pairs: HashSet<(String, String)> = HashSet::new();

    for item in &master_boq.items {
        let mut compliant_peer_totals = Vec::new();
        if matches!(item.kind, LineItemKind::Measured) {
            for quote in quotes {
                let key = (item.id.clone(), quote.vendor_id.clone());
                if let Some(c) = cell_grid.get(&key) {
                    if matches!(c.status, CellStatus::Quoted)
                        && !c.highlight_yellow
                        && c.normalized_measured_net.is_positive()
                    {
                        compliant_peer_totals.push(c.normalized_measured_net);
                    }
                }
            }
        }

        let peer_plug_val = if matches!(item.kind, LineItemKind::Measured) {
            compute_peer_plug(
                config.peer_plug_strategy,
                item,
                &compliant_peer_totals,
                scale,
            )?
        } else {
            DecimalValue::ZERO
        };

        for quote in quotes {
            let key = (item.id.clone(), quote.vendor_id.clone());
            let Some(c) = cell_grid.get(&key) else {
                continue;
            };

            let needs_line_plug = matches!(item.kind, LineItemKind::Measured)
                && matches!(c.status, CellStatus::Unquoted | CellStatus::MovedToOptional);

            let scope_plug_amount = if needs_line_plug {
                plugged_boq_pairs.insert((quote.vendor_id.clone(), item.id.clone()));
                peer_plug_val
            } else {
                DecimalValue::ZERO
            };

            let leveled_net_total = c.normalized_measured_net.checked_add(scope_plug_amount)?;

            finalized_cells.push(LeveledLineCell {
                boq_item_id: item.id.clone(),
                vendor_id: quote.vendor_id.clone(),
                status: c.status.clone(),
                net_unit_rate: c.net_unit_rate,
                as_submitted_net_total: c.as_submitted_net_total,
                arithmetic_delta: c.arithmetic_delta,
                scope_plug_amount,
                leveled_net_total,
                highlight_yellow: c.highlight_yellow,
                audit_note: c.audit_note.clone(),
            });
        }
    }

    // Pass 3: Aggregate per-vendor totals, discounts (Measured only), and clause-level PlugBasis
    let boq_kind_map: HashMap<&str, &LineItemKind> = master_boq
        .items
        .iter()
        .map(|i| (i.id.as_str(), &i.kind))
        .collect();

    let mut summaries = Vec::with_capacity(quotes.len());

    for quote in quotes {
        let mut as_submitted_total = DecimalValue::ZERO;
        let mut arithmetic_and_qty_delta = DecimalValue::ZERO;
        let mut quoted_measured_net = DecimalValue::ZERO;
        let mut provisional_sum_equalized_total = DecimalValue::ZERO;
        let mut line_scope_plugs_total = DecimalValue::ZERO;
        let mut quoted_items_count = 0_usize;
        let mut unquoted_items_count = 0_usize;
        let mut merged_items_count = 0_usize;

        for cell in finalized_cells.iter().filter(|c| c.vendor_id == quote.vendor_id) {
            let kind = boq_kind_map
                .get(cell.boq_item_id.as_str())
                .copied()
                .unwrap_or(&LineItemKind::Measured);

            as_submitted_total = as_submitted_total.checked_add(cell.as_submitted_net_total)?;
            arithmetic_and_qty_delta = arithmetic_and_qty_delta.checked_add(cell.arithmetic_delta)?;
            line_scope_plugs_total = line_scope_plugs_total.checked_add(cell.scope_plug_amount)?;

            match kind {
                LineItemKind::ProvisionalSum => {
                    provisional_sum_equalized_total =
                        provisional_sum_equalized_total.checked_add(cell.leveled_net_total)?;
                }
                LineItemKind::Measured => {
                    quoted_measured_net = quoted_measured_net.checked_add(
                        cell.leveled_net_total.checked_sub(cell.scope_plug_amount)?,
                    )?;
                    match cell.status {
                        CellStatus::Quoted => quoted_items_count += 1,
                        CellStatus::IncludedIn { .. } => merged_items_count += 1,
                        CellStatus::Unquoted | CellStatus::MovedToOptional => {
                            unquoted_items_count += 1;
                        }
                        CellStatus::UnderPricedAllowance => {}
                    }
                }
                LineItemKind::RateOnly | LineItemKind::Optional | LineItemKind::Alternative { .. } => {}
            }
        }

        // Apply general percentage discount & lump-sum discount strictly to Measured work (never PS or plugs)
        let pct_disc = if quote.terms.general_discount_pct.is_positive() {
            quoted_measured_net
                .checked_mul(quote.terms.general_discount_pct)?
                .checked_div(DecimalValue::ONE_HUNDRED)?
                .round_commercial(scale)
        } else {
            DecimalValue::ZERO
        };

        let ls_disc_net = if quote.terms.lump_sum_discount.is_positive() {
            convert_fx_and_strip_vat(quote.terms.lump_sum_discount, quote, None, config)?
                .round_commercial(scale)
        } else {
            DecimalValue::ZERO
        };

        let raw_discount = pct_disc.checked_add(ls_disc_net)?;
        let capped_discount = if raw_discount > quoted_measured_net {
            quoted_measured_net
        } else {
            raw_discount
        };
        let discounted_measured_net = quoted_measured_net.checked_sub(capped_discount)?;
        let discount_adjustment = DecimalValue::ZERO.checked_sub(capped_discount)?;

        // Evaluate clause-level PlugBasis without double counting already plugged BOQ items
        let mut clause_plugs_total = DecimalValue::ZERO;
        let mut vendor_traps_count = 0_usize;

        for trap in traps.iter_mut().filter(|t| t.vendor_id == quote.vendor_id) {
            vendor_traps_count += 1;
            let Some(basis) = &trap.plug_basis else {
                continue;
            };
            let impact = match basis {
                PlugBasis::AnchoredToBoq { boq_id } => {
                    // Zero double-counting guard: if line plug already applied on boq_id, skip!
                    if plugged_boq_pairs.contains(&(quote.vendor_id.clone(), boq_id.clone())) {
                        DecimalValue::ZERO
                    } else {
                        master_boq
                            .items
                            .iter()
                            .find(|b| &b.id == boq_id)
                            .and_then(|b| b.pte_extended_total(scale).ok().flatten())
                            .unwrap_or(DecimalValue::ZERO)
                    }
                }
                PlugBasis::PctOfQuotedOnly { pct } => discounted_measured_net
                    .checked_mul(*pct)?
                    .checked_div(DecimalValue::ONE_HUNDRED)?
                    .round_commercial(scale),
                PlugBasis::ProratedLumpSum { amount } | PlugBasis::FixedSetupFee { amount } => {
                    amount.round_commercial(scale)
                }
            };
            trap.computed_impact = Some(impact);
            clause_plugs_total = clause_plugs_total.checked_add(impact)?;
        }

        let leveled_net_total = discounted_measured_net
            .checked_add(provisional_sum_equalized_total)?
            .checked_add(line_scope_plugs_total)?
            .checked_add(clause_plugs_total)?
            .round_commercial(scale);

        let vat_amount = leveled_net_total
            .checked_mul(config.vat_policy.project_vat_pct)?
            .checked_div(DecimalValue::ONE_HUNDRED)?
            .round_commercial(scale);

        let leveled_gross_total = leveled_net_total.checked_add(vat_amount)?;

        summaries.push(VendorEvaluationSummary {
            vendor_id: quote.vendor_id.clone(),
            vendor_name: quote.vendor_name.clone(),
            as_submitted_total: as_submitted_total.round_commercial(scale),
            arithmetic_and_qty_delta: arithmetic_and_qty_delta.round_commercial(scale),
            discount_adjustment,
            provisional_sum_equalized_total,
            line_scope_plugs_total,
            clause_plugs_total,
            leveled_net_total,
            vat_amount,
            leveled_gross_total,
            variance_vs_lowest_pct: DecimalValue::ZERO,
            quoted_items_count,
            unquoted_items_count,
            merged_items_count,
            traps_count: vendor_traps_count,
            rank: 0,
            is_true_low_bidder: false,
        });
    }

    // Pass 4: Rank vendors by lowest `leveled_net_total` and compute % variance vs lowest
    let mut order_indices: Vec<usize> = (0..summaries.len()).collect();
    order_indices.sort_by(|&a, &b| {
        summaries[a]
            .leveled_net_total
            .cmp(&summaries[b].leveled_net_total)
            .then_with(|| summaries[a].traps_count.cmp(&summaries[b].traps_count))
    });

    let lowest_idx = *order_indices.first().ok_or(EngineError::NoVendorQuotes)?;
    let lowest_net = summaries[lowest_idx].leveled_net_total;
    let true_low_vendor_id = summaries[lowest_idx].vendor_id.clone();
    let true_low_vendor_name = summaries[lowest_idx].vendor_name.clone();

    for (rank_zero, &vec_idx) in order_indices.iter().enumerate() {
        let summary = &mut summaries[vec_idx];
        summary.rank = rank_zero + 1;
        summary.is_true_low_bidder = rank_zero == 0;
        summary.variance_vs_lowest_pct = if lowest_net.is_positive() && rank_zero > 0 {
            summary
                .leveled_net_total
                .checked_sub(lowest_net)?
                .checked_mul(DecimalValue::ONE_HUNDRED)?
                .checked_div(lowest_net)?
                .round_commercial(2)
        } else {
            DecimalValue::ZERO
        };
    }

    Ok(LevelingResult {
        session_id: config.session_id.clone(),
        config: config.clone(),
        master_boq: master_boq.clone(),
        cells: finalized_cells,
        alignment_groups: alignment_groups.to_vec(),
        traps,
        vendor_summaries: summaries,
        true_low_vendor_id,
        true_low_vendor_name,
        true_low_leveled_net: lowest_net,
    })
}
