//! Classic Excel 2016+ coordinate helpers, formula builders, and 31-character tab-name sanitizer.

use crate::error::{WorkbookError, WorkbookResult};

const FORBIDDEN_FORMULA_TOKENS: &[&str] = &[
    "_xlfn.",
    "XLOOKUP(",
    "XMATCH(",
    "MINIFS(",
    "MAXIFS(",
    "IFS(",
    "TEXTJOIN(",
    "LET(",
    "LAMBDA(",
    "FILTER(",
    "UNIQUE(",
];

/// Converts a 0-based column index into an Excel column label (`0 -> "A"`, `25 -> "Z"`, `26 -> "AA"`).
#[must_use]
pub fn col_idx_to_excel_letters(col_idx: u16) -> String {
    let mut n = u32::from(col_idx) + 1;
    let mut chars = Vec::with_capacity(3);
    while n > 0 {
        let rem = (n - 1) % 26;
        chars.push((b'A' + (rem as u8)) as char);
        n = (n - 1) / 26;
    }
    chars.into_iter().rev().collect()
}

/// Formats a 1-based Excel cell coordinate from 0-based `(row, col)`.
#[must_use]
pub fn cell_ref(row: u32, col: u16, lock_col: bool, lock_row: bool) -> String {
    let col_letters = col_idx_to_excel_letters(col);
    let excel_row = row + 1;
    match (lock_col, lock_row) {
        (true, true) => format!("${col_letters}${excel_row}"),
        (true, false) => format!("${col_letters}{excel_row}"),
        (false, true) => format!("{col_letters}${excel_row}"),
        (false, false) => format!("{col_letters}{excel_row}"),
    }
}

/// Sanitizes an OOXML worksheet tab title: strips `\ / ? * : [ ]` and caps length at `31` UTF-16 units.
#[must_use]
pub fn sanitize_sheet_name(raw: &str) -> String {
    let cleaned: String = raw
        .chars()
        .map(|c| match c {
            '\\' | '/' | '?' | '*' | ':' | '[' | ']' => '_',
            other if other.is_control() => ' ',
            other => other,
        })
        .collect::<String>()
        .trim()
        .trim_matches('\'')
        .to_owned();

    let non_empty = if cleaned.is_empty() {
        "Sheet1".to_owned()
    } else {
        cleaned
    };

    let mut out = String::new();
    let mut utf16_count = 0_usize;
    for ch in non_empty.chars() {
        let units = ch.len_utf16();
        if utf16_count + units > 31 {
            break;
        }
        out.push(ch);
        utf16_count += units;
    }
    out
}

/// Rejects any formula containing modern `_xlfn.` dynamic array functions incompatible with Excel 2016.
///
/// # Errors
/// Returns [`WorkbookError::ForbiddenDynamicFormula`] if a prohibited function token is present.
pub fn validate_excel_2016_formula(formula: &str) -> WorkbookResult<()> {
    let upper = formula.to_uppercase();
    for bad in FORBIDDEN_FORMULA_TOKENS {
        if upper.contains(&bad.to_uppercase()) {
            return Err(WorkbookError::ForbiddenDynamicFormula {
                formula: formula.to_owned(),
            });
        }
    }
    Ok(())
}

/// Builds a classic `=SUM(ColStart:ColEnd)` vertical column sum formula.
pub fn build_vertical_sum_formula(col: u16, first_row: u32, last_row: u32) -> WorkbookResult<String> {
    if last_row < first_row {
        return Ok("=0".to_owned());
    }
    let start = cell_ref(first_row, col, false, false);
    let end = cell_ref(last_row, col, false, false);
    let f = format!("=SUM({start}:{end})");
    validate_excel_2016_formula(&f)?;
    Ok(f)
}

/// Builds a classic `=ROUND($QtyCol*RateCol, scale)+PlugCol` line leveling formula.
pub fn build_leveled_line_formula(
    row: u32,
    master_qty_col: u16,
    net_rate_col: u16,
    plug_col: u16,
    scale: u32,
) -> WorkbookResult<String> {
    let qty_ref = cell_ref(row, master_qty_col, true, false);
    let rate_ref = cell_ref(row, net_rate_col, false, false);
    let plug_ref = cell_ref(row, plug_col, false, false);
    let f = format!("=ROUND({qty_ref}*{rate_ref},{scale})+{plug_ref}");
    validate_excel_2016_formula(&f)?;
    Ok(f)
}

/// Builds a classic horizontal `=MIN(c1,c2,...)` formula across vendor leveled columns.
pub fn build_horizontal_min_formula(row: u32, cols: &[u16]) -> WorkbookResult<String> {
    if cols.is_empty() {
        return Ok("=0".to_owned());
    }
    let refs: Vec<String> = cols
        .iter()
        .map(|&c| cell_ref(row, c, false, false))
        .collect();
    let f = format!("=MIN({})", refs.join(","));
    validate_excel_2016_formula(&f)?;
    Ok(f)
}
