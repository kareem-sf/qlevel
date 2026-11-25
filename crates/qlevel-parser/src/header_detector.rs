//! Bilingual Arabic & English BOQ header detection and `MasterBoq` extraction.

use qlevel_domain::{BoqItem, CurrencyCode, DecimalValue, LineItemKind, MasterBoq};

use crate::error::{ParserError, ParserResult};
use crate::model::{DetectedColumnMap, DocumentClassification, ParsedSheet};

/// Converts Eastern Arabic-Indic (`٠-٩`) and Persian (`۰-۹`) digits into ASCII `0-9`,
/// and normalizes Arabic decimal/thousands separators.
#[must_use]
pub fn normalize_arabic_digits(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '\u{0660}' | '\u{06F0}' => out.push('0'),
            '\u{0661}' | '\u{06F1}' => out.push('1'),
            '\u{0662}' | '\u{06F2}' => out.push('2'),
            '\u{0663}' | '\u{06F3}' => out.push('3'),
            '\u{0664}' | '\u{06F4}' => out.push('4'),
            '\u{0665}' | '\u{06F5}' => out.push('5'),
            '\u{0666}' | '\u{06F6}' => out.push('6'),
            '\u{0667}' | '\u{06F7}' => out.push('7'),
            '\u{0668}' | '\u{06F8}' => out.push('8'),
            '\u{0669}' | '\u{06F9}' => out.push('9'),
            '\u{066B}' => out.push('.'),
            '\u{066C}' | '\u{060C}' => out.push(','),
            _ => out.push(ch),
        }
    }
    out
}

fn normalize_header_token(raw: &str) -> String {
    normalize_arabic_digits(raw)
        .to_lowercase()
        .chars()
        .filter(|c| {
            c.is_alphanumeric() || ('\u{0600}'..='\u{06FF}').contains(c) || c.is_whitespace()
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn matches_any(haystack: &str, patterns: &[&str]) -> bool {
    patterns.iter().any(|p| haystack.contains(p))
}

/// Classifies a document's probable role (`MasterBoq`, `VendorQuote`, or `SpecOrAvl`)
/// from its filename and leading content.
#[must_use]
pub fn classify_document_role(file_name: &str, sample_text: &str) -> DocumentClassification {
    let lower_name = file_name.to_lowercase();
    let lower_text = sample_text.to_lowercase();

    let spec_terms = [
        "avl",
        "approved vendor",
        "vendor list",
        "specification",
        "قائمة الموردين",
        "الموردين المعتمدين",
        "المواصفات الفنية",
    ];
    if matches_any(&lower_name, &spec_terms) || matches_any(&lower_text, &spec_terms) {
        return DocumentClassification::SpecOrAvl;
    }

    let boq_terms = [
        "master boq",
        "boq",
        "bill of quantities",
        "tender boq",
        "unpriced",
        "مقايسة",
        "جدول الكميات",
        "كراسة",
    ];
    let quote_terms = [
        "quotation",
        "quote",
        "offer",
        "commercial offer",
        "proposal",
        "عرض سعر",
        "عرض مالي",
        "تسعير",
    ];

    if matches_any(&lower_name, &quote_terms) {
        return DocumentClassification::VendorQuote;
    }
    if matches_any(&lower_name, &boq_terms) {
        return DocumentClassification::MasterBoq;
    }
    if matches_any(&lower_text, &quote_terms) {
        return DocumentClassification::VendorQuote;
    }
    DocumentClassification::VendorQuote
}

/// Scans the top 35 rows of a worksheet to locate bilingual BOQ column headers.
///
/// # Errors
/// Returns [`ParserError::HeaderNotFound`] if no plausible BOQ header row is present.
pub fn detect_header_columns(sheet: &ParsedSheet) -> ParserResult<DetectedColumnMap> {
    const CODE_PATTERNS: &[&str] = &[
        "item",
        "code",
        "ref",
        "s n",
        "sn",
        "no",
        "بند",
        "م",
        "رقم البند",
        "كود",
    ];
    const DESC_PATTERNS: &[&str] = &[
        "description",
        "particulars",
        "scope of work",
        "work description",
        "بيان الأعمال",
        "بيان الاعمال",
        "الوصف",
        "توصيف",
        "البيان",
    ];
    const UNIT_PATTERNS: &[&str] = &["unit", "uom", "الوحدة", "وحدة القياس", "وحده"];
    const QTY_PATTERNS: &[&str] = &["qty", "quantity", "الكمية", "الكميه", "كمية"];
    const RATE_PATTERNS: &[&str] = &[
        "unit rate",
        "rate",
        "unit price",
        "price",
        "الفئة",
        "سعر الوحدة",
        "افرادي",
        "إفرادي",
    ];
    const TOTAL_PATTERNS: &[&str] = &[
        "amount",
        "total",
        "extended",
        "الإجمالي",
        "الاجمالي",
        "القيمة",
        "جملة",
    ];

    for (row_idx, row) in sheet.rows.iter().take(35).enumerate() {
        let mut code_col = None;
        let mut desc_col = None;
        let mut unit_col = None;
        let mut qty_col = None;
        let mut rate_col = None;
        let mut total_col = None;

        for (col_idx, cell) in row.cells.iter().enumerate() {
            let norm = normalize_header_token(cell);
            if norm.is_empty() {
                continue;
            }
            if desc_col.is_none() && matches_any(&norm, DESC_PATTERNS) {
                desc_col = Some(col_idx);
            } else if qty_col.is_none() && matches_any(&norm, QTY_PATTERNS) {
                qty_col = Some(col_idx);
            } else if unit_col.is_none() && matches_any(&norm, UNIT_PATTERNS) {
                unit_col = Some(col_idx);
            } else if rate_col.is_none() && matches_any(&norm, RATE_PATTERNS) {
                rate_col = Some(col_idx);
            } else if total_col.is_none() && matches_any(&norm, TOTAL_PATTERNS) {
                total_col = Some(col_idx);
            } else if code_col.is_none() && matches_any(&norm, CODE_PATTERNS) {
                code_col = Some(col_idx);
            }
        }

        if let Some(description_col) = desc_col {
            if qty_col.is_some() || rate_col.is_some() || total_col.is_some() {
                return Ok(DetectedColumnMap {
                    header_row_index: row_idx,
                    code_col,
                    description_col,
                    unit_col,
                    qty_col,
                    rate_col,
                    total_col,
                });
            }
        }
    }

    Err(ParserError::HeaderNotFound {
        source_name: sheet.sheet_name.clone(),
    })
}

fn classify_line_kind(desc: &str, unit: &str, qty: DecimalValue) -> LineItemKind {
    let lower_desc = desc.to_lowercase();
    let lower_unit = unit.to_lowercase();

    if lower_desc.contains("provisional sum")
        || lower_desc.contains("مبلغ احتياطي")
        || lower_unit == "ps"
    {
        return LineItemKind::ProvisionalSum;
    }
    if lower_desc.contains("rate only")
        || lower_desc.contains("فئة فقط")
        || lower_unit == "ro"
        || (qty.is_zero() && lower_desc.contains("rate"))
    {
        return LineItemKind::RateOnly;
    }
    if lower_desc.contains("optional") || lower_desc.contains("اختياري") {
        return LineItemKind::Optional;
    }
    LineItemKind::Measured
}

fn is_subtotal_or_footer_row(desc: &str) -> bool {
    let lower = desc.trim().to_lowercase();
    let footer_prefixes = [
        "total",
        "sub-total",
        "subtotal",
        "grand total",
        "carried to summary",
        "vat ",
        "الإجمالي",
        "الاجمالي",
        "المجموع",
        "جملة القسم",
    ];
    footer_prefixes
        .iter()
        .any(|p| lower.starts_with(p) || lower == *p)
}

/// Deterministically extracts a `MasterBoq` from a parsed worksheet.
///
/// # Errors
/// Returns [`ParserError`] if headers cannot be detected or no valid BOQ items exist.
pub fn extract_master_boq_from_sheet(
    title: &str,
    currency: CurrencyCode,
    sheet: &ParsedSheet,
) -> ParserResult<MasterBoq> {
    let cols = detect_header_columns(sheet)?;
    let mut items = Vec::new();
    let mut current_section: Option<String> = None;

    for row in sheet.rows.iter().skip(cols.header_row_index + 1) {
        if row.is_hidden {
            continue;
        }
        let desc = row
            .cells
            .get(cols.description_col)
            .map(|s| s.trim())
            .unwrap_or("");
        if desc.is_empty() || is_subtotal_or_footer_row(desc) {
            continue;
        }

        let unit_raw = cols
            .unit_col
            .and_then(|idx| row.cells.get(idx))
            .map(|s| s.trim())
            .unwrap_or("");
        let qty_raw = cols
            .qty_col
            .and_then(|idx| row.cells.get(idx))
            .map(|s| normalize_arabic_digits(s))
            .unwrap_or_default();
        let rate_raw = cols
            .rate_col
            .and_then(|idx| row.cells.get(idx))
            .map(|s| normalize_arabic_digits(s))
            .unwrap_or_default();

        let parsed_qty = DecimalValue::parse_clean(&qty_raw).ok();
        let parsed_rate = DecimalValue::parse_clean(&rate_raw).ok();

        // Section divider row: has description text, no unit, and no quantity/rate
        if unit_raw.is_empty() && parsed_qty.is_none() && parsed_rate.is_none() {
            current_section = Some(desc.to_owned());
            continue;
        }

        let qty = parsed_qty.unwrap_or(DecimalValue::ONE);
        if qty.is_negative() {
            continue;
        }

        let next_num = items.len() + 1;
        let code = cols
            .code_col
            .and_then(|idx| row.cells.get(idx))
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| format!("{next_num}"));

        let unit = if unit_raw.is_empty() {
            "LS".to_owned()
        } else {
            unit_raw.to_owned()
        };
        let kind = classify_line_kind(desc, &unit, qty);

        items.push(BoqItem {
            id: format!("BOQ-{next_num:04}"),
            code,
            section: current_section.clone(),
            description: desc.to_owned(),
            unit,
            quantity: qty,
            kind,
            pte_unit_rate: parsed_rate,
        });
    }

    let boq = MasterBoq {
        title: title.to_owned(),
        currency,
        is_synthetic: false,
        items,
    };
    boq.validate()?;
    Ok(boq)
}
