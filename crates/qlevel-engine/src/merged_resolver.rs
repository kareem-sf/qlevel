//! Merged vendor row alignment & resolution (`InteractiveReview` vs `YellowHighlightAuto`).
//!
//! Enforces the Anchor-and-Zero-Followers invariant on `ManyMasterToOneVendor` groups so a
//! vendor's lump-sum price is counted once on the anchor BOQ item while follower BOQ items
//! carry `CellStatus::IncludedIn { anchor_boq_id }` with `0` leveled amount and zero plugs.

use std::collections::HashMap;

use qlevel_domain::{
    AlignmentGroup, AlignmentResolution, BoqItem, CellStatus, DecimalValue, GroupCardinality,
    MasterBoq, MergedItemMode, VendorLineQuote, VendorQuote,
};

use crate::error::EngineResult;

/// Scans vendor quotes for `IncludedIn` relationships and constructs `AlignmentGroup` records.
#[must_use]
pub fn build_initial_alignment_groups(
    quotes: &[VendorQuote],
    mode: MergedItemMode,
) -> Vec<AlignmentGroup> {
    let mut groups = Vec::new();

    for quote in quotes {
        let mut followers_by_anchor: HashMap<String, Vec<String>> = HashMap::new();
        for line in &quote.lines {
            if let CellStatus::IncludedIn { anchor_boq_id } = &line.status {
                followers_by_anchor
                    .entry(anchor_boq_id.clone())
                    .or_default()
                    .push(line.boq_item_id.clone());
            }
        }

        for (anchor_id, mut followers) in followers_by_anchor {
            followers.sort();
            followers.dedup();

            let anchor_line = quote.lines.iter().find(|l| l.boq_item_id == anchor_id);
            let lump_sum = anchor_line
                .map(|l| l.quoted_extended_total)
                .unwrap_or(DecimalValue::ZERO);
            let vendor_desc = anchor_line
                .map(|l| l.vendor_description.clone())
                .unwrap_or_else(|| "Merged vendor lump-sum item".to_owned());

            let mut master_boq_ids = Vec::with_capacity(followers.len() + 1);
            master_boq_ids.push(anchor_id.clone());
            master_boq_ids.extend(followers);

            let resolution = match mode {
                MergedItemMode::YellowHighlightAuto => AlignmentResolution::AnchorAndZeroFollowers,
                MergedItemMode::InteractiveReview => AlignmentResolution::PendingReview,
            };

            groups.push(AlignmentGroup {
                group_id: format!("GRP-{}-{}", quote.vendor_id, anchor_id),
                vendor_id: quote.vendor_id.clone(),
                cardinality: GroupCardinality::ManyMasterToOneVendor,
                master_boq_ids,
                vendor_descriptions: vec![vendor_desc],
                quoted_lump_sum: lump_sum,
                confidence: DecimalValue::from_i64(85),
                resolution,
                reviewer_note: None,
            });
        }
    }

    groups
}

/// Applies finalized `AlignmentGroup` resolutions onto `VendorQuote` lines prior to commercial math.
///
/// # Errors
/// Returns [`crate::error::EngineError`] if proportional split arithmetic overflows.
pub fn apply_merged_group_resolutions(
    master_boq: &MasterBoq,
    quotes: &mut [VendorQuote],
    groups: &[AlignmentGroup],
) -> EngineResult<()> {
    let boq_by_id: HashMap<&str, &BoqItem> = master_boq
        .items
        .iter()
        .map(|item| (item.id.as_str(), item))
        .collect();
    let scale = master_boq.currency.decimal_scale();

    for group in groups {
        let Some(quote) = quotes.iter_mut().find(|q| q.vendor_id == group.vendor_id) else {
            continue;
        };
        let Some(anchor_id) = group.master_boq_ids.first() else {
            continue;
        };

        match group.resolution {
            AlignmentResolution::PendingReview
            | AlignmentResolution::AnchorAndZeroFollowers
            | AlignmentResolution::AcceptedAlternate => {
                for boq_id in group.master_boq_ids.iter().skip(1) {
                    if let Some(line) = quote.lines.iter_mut().find(|l| &l.boq_item_id == boq_id) {
                        line.status = CellStatus::IncludedIn {
                            anchor_boq_id: anchor_id.clone(),
                        };
                        line.quoted_unit_rate = DecimalValue::ZERO;
                        line.quoted_extended_total = DecimalValue::ZERO;
                    }
                }
            }
            AlignmentResolution::ExcludedWithPlug => {
                for boq_id in &group.master_boq_ids {
                    if let Some(line) = quote.lines.iter_mut().find(|l| &l.boq_item_id == boq_id) {
                        line.status = CellStatus::Unquoted;
                        line.quoted_unit_rate = DecimalValue::ZERO;
                        line.quoted_extended_total = DecimalValue::ZERO;
                    }
                }
            }
            AlignmentResolution::ProportionalSplit => {
                // Allocate lump sum across Master BOQ items by PTE weight (or equal weight if no PTE)
                let count_u64 = group.master_boq_ids.len() as u64;
                if count_u64 == 0 {
                    continue;
                }
                let mut weights: Vec<DecimalValue> = Vec::with_capacity(group.master_boq_ids.len());
                let mut weight_sum = DecimalValue::ZERO;

                for boq_id in &group.master_boq_ids {
                    let w = boq_by_id
                        .get(boq_id.as_str())
                        .and_then(|b| b.pte_extended_total(scale).ok().flatten())
                        .filter(|v| v.is_positive())
                        .unwrap_or(DecimalValue::ONE);
                    weight_sum = weight_sum.checked_add(w)?;
                    weights.push(w);
                }

                let mut allocated_so_far = DecimalValue::ZERO;
                let last_idx = group.master_boq_ids.len() - 1;

                for (idx, boq_id) in group.master_boq_ids.iter().enumerate() {
                    let share = if idx == last_idx {
                        group.quoted_lump_sum.checked_sub(allocated_so_far)?
                    } else {
                        let w = weights.get(idx).copied().unwrap_or(DecimalValue::ONE);
                        let raw_share = group
                            .quoted_lump_sum
                            .checked_mul(w)?
                            .checked_div(weight_sum)?
                            .round_commercial(scale);
                        allocated_so_far = allocated_so_far.checked_add(raw_share)?;
                        raw_share
                    };

                    let master_qty = boq_by_id
                        .get(boq_id.as_str())
                        .map(|b| b.quantity)
                        .filter(|q| q.is_positive())
                        .unwrap_or(DecimalValue::ONE);
                    let derived_rate = share.checked_div(master_qty)?.round_commercial(scale + 2);

                    if let Some(line) = quote.lines.iter_mut().find(|l| &l.boq_item_id == boq_id) {
                        line.status = CellStatus::Quoted;
                        line.quoted_qty = master_qty;
                        line.quoted_unit_rate = derived_rate;
                        line.quoted_extended_total = share;
                    }
                }
            }
        }
    }

    Ok(())
}
