//! Synthetic `MasterBoq` builder for "Quotes-Only" tenders (when no client Master BOQ exists).
//!
//! Elects the most structured vendor quotation as seed baseline and performs unit-compatible
//! string similarity alignment (`rapidfuzz` integer edit distance -> `DecimalValue` score)
//! while appending any unmatched vendor items so `orphan_vendor_rows == 0`.

use std::collections::HashMap;

use qlevel_domain::{BoqItem, CurrencyCode, DecimalValue, LineItemKind, MasterBoq};
use rapidfuzz::distance::levenshtein;

use crate::error::{EngineError, EngineResult};

/// Raw line item candidate extracted from an unaligned vendor quotation.
#[derive(Debug, Clone)]
pub struct RawVendorCandidateItem {
    pub vendor_id: String,
    pub row_index: usize,
    pub code: Option<String>,
    pub description: String,
    pub unit: String,
    pub quantity: DecimalValue,
}

/// Output of `build_synthetic_master_boq`: unified `MasterBoq` plus `(vendor_id, row_index) -> boq_id`.
#[derive(Debug, Clone)]
pub struct SyntheticBoqOutcome {
    pub master_boq: MasterBoq,
    pub vendor_row_to_boq_id: HashMap<(String, usize), String>,
    pub orphan_vendor_rows: usize,
}

fn canonicalize_unit(raw: &str) -> &'static str {
    let norm = raw.trim().to_lowercase();
    match norm.as_str() {
        "m2" | "sqm" | "sq.m" | "م2" | "متر مربع" => "m2",
        "m3" | "cum" | "cu.m" | "م3" | "متر مكعب" => "m3",
        "m" | "lm" | "rm" | "م.ط" | "متر طولي" => "m",
        "no" | "nos" | "nr" | "pcs" | "ea" | "عدد" | "قطعة" | "حبة" => "No",
        "kg" | "كجم" | "كغ" => "kg",
        "ton" | "t" | "mt" | "طن" => "ton",
        "ls" | "item" | "sum" | "مقطوعية" | "بالمقطوعية" => "LS",
        _ => "OTHER",
    }
}

fn units_compatible(u1: &str, u2: &str) -> bool {
    let c1 = canonicalize_unit(u1);
    let c2 = canonicalize_unit(u2);
    if c1 == "OTHER" || c2 == "OTHER" {
        return u1.trim().eq_ignore_ascii_case(u2.trim());
    }
    c1 == c2
}

fn normalize_desc(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || ('\u{0600}'..='\u{06FF}').contains(c) || c.is_whitespace())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Computes integer similarity in `[0, 100]` using `rapidfuzz` Levenshtein distance (zero floats).
fn description_similarity_pct(a: &str, b: &str) -> u32 {
    let na = normalize_desc(a);
    let nb = normalize_desc(b);
    let len_a = na.chars().count();
    let len_b = nb.chars().count();
    let max_len = len_a.max(len_b);
    if max_len == 0 {
        return 100;
    }
    let dist = levenshtein::distance(na.chars(), nb.chars());
    let matched = max_len.saturating_sub(dist);
    let scaled = matched.saturating_mul(100) / max_len;
    scaled as u32
}

/// Synthesizes a unified `MasterBoq` from raw vendor candidate lines with `orphan_vendor_rows == 0`.
///
/// # Errors
/// Returns [`EngineError::NoVendorQuotes`] if `candidates_by_vendor` has no items.
pub fn build_synthetic_master_boq(
    title: &str,
    currency: CurrencyCode,
    candidates_by_vendor: &[Vec<RawVendorCandidateItem>],
) -> EngineResult<SyntheticBoqOutcome> {
    let non_empty: Vec<&Vec<RawVendorCandidateItem>> = candidates_by_vendor
        .iter()
        .filter(|v| !v.is_empty())
        .collect();

    if non_empty.is_empty() {
        return Err(EngineError::NoVendorQuotes);
    }

    // Elect the vendor quotation with the highest itemized row count as the seed baseline
    let (seed_idx, seed_vendor_items) = non_empty
        .iter()
        .enumerate()
        .max_by_key(|(_, list)| list.len())
        .ok_or(EngineError::NoVendorQuotes)?;

    let mut boq_items: Vec<BoqItem> = Vec::new();
    let mut vendor_row_to_boq_id: HashMap<(String, usize), String> = HashMap::new();

    for seed in *seed_vendor_items {
        let next_idx = boq_items.len() + 1;
        let boq_id = format!("SYN-{next_idx:04}");
        let code = seed
            .code
            .clone()
            .filter(|c| !c.trim().is_empty())
            .unwrap_or_else(|| format!("{next_idx}"));

        boq_items.push(BoqItem {
            id: boq_id.clone(),
            code,
            section: None,
            description: seed.description.clone(),
            unit: seed.unit.clone(),
            quantity: if seed.quantity.is_positive() {
                seed.quantity
            } else {
                DecimalValue::ONE
            },
            kind: LineItemKind::Measured,
            pte_unit_rate: None,
        });
        vendor_row_to_boq_id.insert((seed.vendor_id.clone(), seed.row_index), boq_id);
    }

    // Align remaining vendors via constrained greedy best-similarity matching (>= 65%),
    // appending any unmatched vendor row as a new synthetic BOQ item so zero rows are orphaned.
    for (v_idx, vendor_items) in non_empty.iter().enumerate() {
        if v_idx == seed_idx {
            continue;
        }
        let mut claimed_boq_indices = std::collections::HashSet::new();

        for cand in *vendor_items {
            let mut best_match: Option<(usize, u32)> = None;
            for (b_idx, existing) in boq_items.iter().enumerate() {
                if claimed_boq_indices.contains(&b_idx) {
                    continue;
                }
                if !units_compatible(&existing.unit, &cand.unit) {
                    continue;
                }
                let score = description_similarity_pct(&existing.description, &cand.description);
                if score >= 65 && best_match.is_none_or(|(_, prev)| score > prev) {
                    best_match = Some((b_idx, score));
                }
            }

            if let Some((matched_idx, _)) = best_match {
                claimed_boq_indices.insert(matched_idx);
                if let Some(target_item) = boq_items.get(matched_idx) {
                    vendor_row_to_boq_id
                        .insert((cand.vendor_id.clone(), cand.row_index), target_item.id.clone());
                }
            } else {
                let next_idx = boq_items.len() + 1;
                let boq_id = format!("SYN-{next_idx:04}");
                let code = cand
                    .code
                    .clone()
                    .filter(|c| !c.trim().is_empty())
                    .unwrap_or_else(|| format!("{next_idx}"));

                boq_items.push(BoqItem {
                    id: boq_id.clone(),
                    code,
                    section: None,
                    description: cand.description.clone(),
                    unit: cand.unit.clone(),
                    quantity: if cand.quantity.is_positive() {
                        cand.quantity
                    } else {
                        DecimalValue::ONE
                    },
                    kind: LineItemKind::Measured,
                    pte_unit_rate: None,
                });
                claimed_boq_indices.insert(boq_items.len() - 1);
                vendor_row_to_boq_id.insert((cand.vendor_id.clone(), cand.row_index), boq_id);
            }
        }
    }

    let total_input_rows: usize = non_empty.iter().map(|v| v.len()).sum();
    let orphan_vendor_rows = total_input_rows.saturating_sub(vendor_row_to_boq_id.len());

    let master_boq = MasterBoq {
        title: title.to_owned(),
        currency,
        is_synthetic: true,
        items: boq_items,
    };
    master_boq.validate()?;

    Ok(SyntheticBoqOutcome {
        master_boq,
        vendor_row_to_boq_id,
        orphan_vendor_rows,
    })
}
