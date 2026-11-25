//! Worksheet layout builders for `SingleMasterSheet` and `DualTab` comparison workbooks.

use std::collections::HashMap;

use chrono::Utc;
use qlevel_domain::{
    CellStatus, DecimalValue, LeveledLineCell, LevelingResult, Locale, SheetDirection,
    WorkbookLayoutMode,
};
use rust_xlsxwriter::{Formula, Workbook, Worksheet};

use crate::error::WorkbookResult;
use crate::formulas::{
    build_horizontal_min_formula, build_leveled_line_formula, build_vertical_sum_formula,
    cell_ref, sanitize_sheet_name,
};
use crate::styles::WorkbookStylePalette;

/// Converts an exact `DecimalValue` into an Excel numeric value strictly at the writer boundary.
fn dec_to_xlsx_num(val: DecimalValue) -> f64 {
    val.to_string().parse::<f64>().unwrap_or(0.0)
}

fn should_use_rtl(result: &LevelingResult) -> bool {
    match result.config.sheet_direction {
        SheetDirection::Rtl => true,
        SheetDirection::Ltr => false,
        SheetDirection::Auto => {
            if result.config.ui_locale == Locale::Ar {
                return true;
            }
            result
                .master_boq
                .items
                .iter()
                .take(15)
                .any(|item| item.description.chars().any(|c| ('\u{0600}'..='\u{06FF}').contains(&c)))
        }
    }
}

struct Labels {
    code: &'static str,
    section: &'static str,
    desc: &'static str,
    unit: &'static str,
    qty: &'static str,
    net_rate: &'static str,
    submitted: &'static str,
    plug: &'static str,
    leveled: &'static str,
    note: &'static str,
    min_leveled: &'static str,
    exec_summary: &'static str,
    tbe_header: &'static str,
}

fn labels_for(rtl: bool) -> Labels {
    if rtl {
        Labels {
            code: "الكود",
            section: "القسم",
            desc: "بيان الأعمال",
            unit: "الوحدة",
            qty: "الكمية الأساسية",
            net_rate: "سعر الوحدة الصافي",
            submitted: "الإجمالي المقدم",
            plug: "بدل توحيد النطاق",
            leveled: "الإجمالي المقارن الصافي",
            note: "حالة البند / ملاحظة",
            min_leveled: "أقل سعر مقارن",
            exec_summary: "الملخص التنفيذي للمقارنة المالية (CBE)",
            tbe_header: "مصفوفة الفخاخ والاستثناءات الفنية والتعاقدية (TBE)",
        }
    } else {
        Labels {
            code: "Code",
            section: "Section",
            desc: "Work Description",
            unit: "Unit",
            qty: "Master Qty",
            net_rate: "Net Unit Rate",
            submitted: "Submitted Net",
            plug: "Scope Plug",
            leveled: "Leveled Net Total",
            note: "Status / Audit Note",
            min_leveled: "Lowest Leveled",
            exec_summary: "Executive Commercial Equalization Summary (CBE)",
            tbe_header: "Technical Exclusions, Deviations & Risk Matrix (TBE)",
        }
    }
}

fn write_commercial_section(
    ws: &mut Worksheet,
    result: &LevelingResult,
    styles: &WorkbookStylePalette,
    rtl: bool,
) -> WorkbookResult<u32> {
    let lbl = labels_for(rtl);
    let scale = result.config.currency.decimal_scale();
    let vendor_count = result.vendor_summaries.len();
    let total_cols = (5 + vendor_count * 5) as u16;

    // Banner
    let title = format!(
        "qlevel - {} ({})",
        result.master_boq.title,
        result.config.currency.as_str()
    );
    ws.set_row_height(0, 28.0)?;
    ws.merge_range(0, 0, 0, total_cols, &title, &styles.title_banner)?;

    let meta = format!(
        "Date: {} | VAT: {}% | True Low Bidder: {} ({} {})",
        Utc::now().format("%Y-%m-%d"),
        result.config.vat_policy.project_vat_pct,
        result.true_low_vendor_name,
        result.true_low_leveled_net.to_fixed_string(scale),
        result.config.currency.as_str()
    );
    ws.merge_range(1, 0, 1, total_cols, &meta, &styles.subtitle_meta)?;

    // Executive Summary Block starting at row 3
    let mut row = 3_u32;
    ws.merge_range(row, 0, row, total_cols.min(9), lbl.exec_summary, &styles.vendor_group_header)?;
    row += 1;

    let sum_headers = [
        "Rank",
        "Vendor",
        "Submitted Net",
        "Math/Qty Delta",
        "Discounts",
        "Provisional Sums",
        "Line Scope Plugs",
        "Clause Risk Plugs",
        "Leveled Net Total",
        "VAT",
        "Leveled Gross Total",
        "Var vs Low %",
        "Traps",
    ];
    for (c_idx, h) in sum_headers.iter().enumerate() {
        ws.write_string_with_format(row, c_idx as u16, *h, &styles.table_header)?;
    }
    row += 1;

    let mut sorted_summaries = result.vendor_summaries.clone();
    sorted_summaries.sort_by_key(|s| s.rank);

    for s in &sorted_summaries {
        let money_fmt = if s.is_true_low_bidder {
            &styles.lowest_winner_money
        } else {
            &styles.money_cell
        };
        let txt_fmt = if s.is_true_low_bidder {
            &styles.lowest_winner_text
        } else {
            &styles.text_center
        };

        ws.write_number_with_format(row, 0, s.rank as f64, txt_fmt)?;
        ws.write_string_with_format(row, 1, &s.vendor_name, txt_fmt)?;
        ws.write_number_with_format(row, 2, dec_to_xlsx_num(s.as_submitted_total), money_fmt)?;
        ws.write_number_with_format(row, 3, dec_to_xlsx_num(s.arithmetic_and_qty_delta), money_fmt)?;
        ws.write_number_with_format(row, 4, dec_to_xlsx_num(s.discount_adjustment), money_fmt)?;
        ws.write_number_with_format(
            row,
            5,
            dec_to_xlsx_num(s.provisional_sum_equalized_total),
            money_fmt,
        )?;
        ws.write_number_with_format(row, 6, dec_to_xlsx_num(s.line_scope_plugs_total), money_fmt)?;
        ws.write_number_with_format(row, 7, dec_to_xlsx_num(s.clause_plugs_total), money_fmt)?;
        ws.write_number_with_format(row, 8, dec_to_xlsx_num(s.leveled_net_total), money_fmt)?;
        ws.write_number_with_format(row, 9, dec_to_xlsx_num(s.vat_amount), money_fmt)?;
        ws.write_number_with_format(row, 10, dec_to_xlsx_num(s.leveled_gross_total), money_fmt)?;
        ws.write_number_with_format(
            row,
            11,
            dec_to_xlsx_num(s.variance_vs_lowest_pct),
            &styles.pct_cell,
        )?;
        ws.write_number_with_format(row, 12, s.traps_count as f64, txt_fmt)?;
        row += 1;
    }

    row += 1;

    // BOQ Comparison Matrix Super-Header + Column Header
    let super_header_row = row;
    ws.merge_range(
        super_header_row,
        0,
        super_header_row,
        4,
        "Master BOQ Baseline",
        &styles.table_header,
    )?;

    for (v_idx, v_sum) in result.vendor_summaries.iter().enumerate() {
        let base_col = 5 + (v_idx as u16) * 5;
        ws.merge_range(
            super_header_row,
            base_col,
            super_header_row,
            base_col + 4,
            &v_sum.vendor_name,
            &styles.vendor_group_header,
        )?;
    }
    let min_col = 5 + (vendor_count as u16) * 5;
    ws.write_string_with_format(super_header_row, min_col, "Benchmark", &styles.table_header)?;

    row += 1;
    let col_header_row = row;
    ws.write_string_with_format(col_header_row, 0, lbl.code, &styles.table_header)?;
    ws.write_string_with_format(col_header_row, 1, lbl.section, &styles.table_header)?;
    ws.write_string_with_format(col_header_row, 2, lbl.desc, &styles.table_header)?;
    ws.write_string_with_format(col_header_row, 3, lbl.unit, &styles.table_header)?;
    ws.write_string_with_format(col_header_row, 4, lbl.qty, &styles.table_header)?;

    ws.set_column_width(0, 11.0)?;
    ws.set_column_width(1, 16.0)?;
    ws.set_column_width(2, 42.0)?;
    ws.set_column_width(3, 9.0)?;
    ws.set_column_width(4, 12.0)?;

    let mut leveled_cols_for_min = Vec::with_capacity(vendor_count);
    for v_idx in 0..vendor_count {
        let base_col = 5 + (v_idx as u16) * 5;
        ws.write_string_with_format(col_header_row, base_col, lbl.net_rate, &styles.table_header)?;
        ws.write_string_with_format(col_header_row, base_col + 1, lbl.submitted, &styles.table_header)?;
        ws.write_string_with_format(col_header_row, base_col + 2, lbl.plug, &styles.table_header)?;
        ws.write_string_with_format(col_header_row, base_col + 3, lbl.leveled, &styles.table_header)?;
        ws.write_string_with_format(col_header_row, base_col + 4, lbl.note, &styles.table_header)?;

        ws.set_column_width(base_col, 14.0)?;
        ws.set_column_width(base_col + 1, 15.0)?;
        ws.set_column_width(base_col + 2, 14.0)?;
        ws.set_column_width(base_col + 3, 16.0)?;
        ws.set_column_width(base_col + 4, 24.0)?;

        leveled_cols_for_min.push(base_col + 3);
    }
    ws.write_string_with_format(col_header_row, min_col, lbl.min_leveled, &styles.table_header)?;
    ws.set_column_width(min_col, 16.0)?;

    row += 1;
    let first_data_row = row;
    ws.set_freeze_panes(first_data_row, 3)?;

    let cell_lookup: HashMap<(&str, &str), &LeveledLineCell> = result
        .cells
        .iter()
        .map(|c| ((c.boq_item_id.as_str(), c.vendor_id.as_str()), c))
        .collect();

    for boq_item in &result.master_boq.items {
        ws.write_string_with_format(row, 0, &boq_item.code, &styles.text_center)?;
        ws.write_string_with_format(
            row,
            1,
            boq_item.section.as_deref().unwrap_or("-"),
            &styles.text_cell,
        )?;
        ws.write_string_with_format(row, 2, &boq_item.description, &styles.text_cell)?;
        ws.write_string_with_format(row, 3, &boq_item.unit, &styles.text_center)?;
        ws.write_number_with_format(row, 4, dec_to_xlsx_num(boq_item.quantity), &styles.qty_cell)?;

        let mut min_row_leveled: Option<DecimalValue> = None;
        for v_sum in &result.vendor_summaries {
            if let Some(cell) = cell_lookup.get(&(boq_item.id.as_str(), v_sum.vendor_id.as_str())) {
                if cell.leveled_net_total.is_positive()
                    && min_row_leveled.is_none_or(|prev| cell.leveled_net_total < prev)
                {
                    min_row_leveled = Some(cell.leveled_net_total);
                }
            }
        }

        for (v_idx, v_sum) in result.vendor_summaries.iter().enumerate() {
            let base_col = 5 + (v_idx as u16) * 5;
            let Some(cell) = cell_lookup.get(&(boq_item.id.as_str(), v_sum.vendor_id.as_str())) else {
                continue;
            };

            let is_unquoted = matches!(cell.status, CellStatus::Unquoted | CellStatus::MovedToOptional);
            let is_lowest_cell =
                min_row_leveled.is_some_and(|m| cell.leveled_net_total == m && !cell.highlight_yellow);

            let (money_style, note_style) = if cell.highlight_yellow {
                (&styles.yellow_merged_money, &styles.yellow_merged_text)
            } else if is_unquoted {
                (&styles.risk_alert_money, &styles.risk_alert_text)
            } else if is_lowest_cell {
                (&styles.lowest_winner_money, &styles.text_cell)
            } else {
                (&styles.money_cell, &styles.text_cell)
            };

            ws.write_number_with_format(
                row,
                base_col,
                dec_to_xlsx_num(cell.net_unit_rate),
                money_style,
            )?;
            ws.write_number_with_format(
                row,
                base_col + 1,
                dec_to_xlsx_num(cell.as_submitted_net_total),
                money_style,
            )?;
            ws.write_number_with_format(
                row,
                base_col + 2,
                dec_to_xlsx_num(cell.scope_plug_amount),
                money_style,
            )?;

            // Live Excel 2016+ formula: if merged lump-sum anchor with zero rate, sum submitted + plug;
            // otherwise =ROUND($E{r}*Rate, scale)+Plug
            let formula_str = if cell.highlight_yellow && cell.net_unit_rate.is_zero() {
                let sub_ref = cell_ref(row, base_col + 1, false, false);
                let plug_ref = cell_ref(row, base_col + 2, false, false);
                format!("={sub_ref}+{plug_ref}")
            } else if matches!(cell.status, CellStatus::IncludedIn { .. }) {
                "=0".to_owned()
            } else {
                build_leveled_line_formula(row, 4, base_col, base_col + 2, scale)?
            };

            let formula = Formula::new(&formula_str)
                .set_result(&cell.leveled_net_total.to_fixed_string(scale));
            ws.write_formula_with_format(row, base_col + 3, formula, money_style)?;

            let status_note = cell.audit_note.as_deref().unwrap_or(match &cell.status {
                CellStatus::Quoted => "Quoted",
                CellStatus::IncludedIn { .. } => "Included in anchor",
                CellStatus::MovedToOptional => "Moved to Optional",
                CellStatus::Unquoted => "Unquoted - Plugged",
                CellStatus::UnderPricedAllowance => "PS Equalized",
            });
            ws.write_string_with_format(row, base_col + 4, status_note, note_style)?;
        }

        let min_formula_str = build_horizontal_min_formula(row, &leveled_cols_for_min)?;
        let cached_min = min_row_leveled
            .unwrap_or(DecimalValue::ZERO)
            .to_fixed_string(scale);
        ws.write_formula_with_format(
            row,
            min_col,
            Formula::new(&min_formula_str).set_result(&cached_min),
            &styles.money_bold,
        )?;

        row += 1;
    }

    let last_data_row = row.saturating_sub(1);

    // Live Subtotal Footer Row (`=SUM(...)`)
    ws.merge_range(row, 0, row, 4, "Subtotal (BOQ Line Items + Line Plugs)", &styles.table_header)?;
    for (v_idx, v_sum) in result.vendor_summaries.iter().enumerate() {
        let base_col = 5 + (v_idx as u16) * 5;
        let sub_sum = build_vertical_sum_formula(base_col + 1, first_data_row, last_data_row)?;
        let plug_sum = build_vertical_sum_formula(base_col + 2, first_data_row, last_data_row)?;
        let lvl_sum = build_vertical_sum_formula(base_col + 3, first_data_row, last_data_row)?;

        ws.write_string_with_format(row, base_col, "", &styles.money_bold)?;
        ws.write_formula_with_format(
            row,
            base_col + 1,
            Formula::new(&sub_sum).set_result(&v_sum.as_submitted_total.to_fixed_string(scale)),
            &styles.money_bold,
        )?;
        ws.write_formula_with_format(
            row,
            base_col + 2,
            Formula::new(&plug_sum).set_result(&v_sum.line_scope_plugs_total.to_fixed_string(scale)),
            &styles.money_bold,
        )?;
        ws.write_formula_with_format(
            row,
            base_col + 3,
            Formula::new(&lvl_sum).set_result(&v_sum.leveled_net_total.to_fixed_string(scale)),
            &styles.money_bold,
        )?;
        ws.write_string_with_format(row, base_col + 4, "", &styles.money_bold)?;
    }
    row += 2;

    Ok(row)
}

fn write_technical_section(
    ws: &mut Worksheet,
    start_row: u32,
    result: &LevelingResult,
    styles: &WorkbookStylePalette,
    rtl: bool,
) -> WorkbookResult<()> {
    let lbl = labels_for(rtl);
    let scale = result.config.currency.decimal_scale();
    let mut row = start_row;

    ws.merge_range(row, 0, row, 7, lbl.tbe_header, &styles.vendor_group_header)?;
    row += 1;

    let headers = [
        "Vendor",
        "Category",
        "Severity",
        "BOQ Ref",
        "Finding Title",
        "Engineering Impact Detail",
        "Source Clause / Quote",
        "Financial Impact",
    ];
    for (c_idx, h) in headers.iter().enumerate() {
        ws.write_string_with_format(row, c_idx as u16, *h, &styles.table_header)?;
    }
    row += 1;

    let vendor_names: HashMap<&str, &str> = result
        .vendor_summaries
        .iter()
        .map(|s| (s.vendor_id.as_str(), s.vendor_name.as_str()))
        .collect();

    if result.traps.is_empty() {
        ws.merge_range(
            row,
            0,
            row,
            7,
            "No technical exclusions, AVL deviations, or arithmetic discrepancies detected.",
            &styles.lowest_winner_text,
        )?;
        return Ok(());
    }

    for trap in &result.traps {
        let v_name = vendor_names
            .get(trap.vendor_id.as_str())
            .copied()
            .unwrap_or(trap.vendor_id.as_str());
        let cat_label = format!("{:?}", trap.category);
        let sev_label = format!("{:?}", trap.severity);
        let impact_val = trap
            .computed_impact
            .unwrap_or(DecimalValue::ZERO)
            .round_commercial(scale);

        ws.write_string_with_format(row, 0, v_name, &styles.text_cell)?;
        ws.write_string_with_format(row, 1, &cat_label, &styles.text_center)?;
        ws.write_string_with_format(row, 2, &sev_label, &styles.risk_alert_text)?;
        ws.write_string_with_format(
            row,
            3,
            trap.related_boq_id.as_deref().unwrap_or("General"),
            &styles.text_center,
        )?;
        ws.write_string_with_format(row, 4, &trap.title, &styles.text_cell)?;
        ws.write_string_with_format(row, 5, &trap.detail, &styles.text_cell)?;
        ws.write_string_with_format(row, 6, &trap.source_snippet, &styles.yellow_merged_text)?;
        ws.write_number_with_format(row, 7, dec_to_xlsx_num(impact_val), &styles.risk_alert_money)?;
        row += 1;
    }

    Ok(())
}

/// Builds a complete `rust_xlsxwriter::Workbook` honoring `SingleMasterSheet` or `DualTab`.
pub fn build_workbook(result: &LevelingResult) -> WorkbookResult<Workbook> {
    let mut wb = Workbook::new();
    let styles = WorkbookStylePalette::new(result.config.currency);
    let rtl = should_use_rtl(result);

    match result.config.layout_mode {
        WorkbookLayoutMode::SingleMasterSheet => {
            let ws = wb.add_worksheet();
            ws.set_name(&sanitize_sheet_name("Bid_Leveling_Sheet"))?;
            ws.set_right_to_left(rtl);

            let next_row = write_commercial_section(ws, result, &styles, rtl)?;
            write_technical_section(ws, next_row, result, &styles, rtl)?;
        }
        WorkbookLayoutMode::DualTab => {
            {
                let ws_cbe = wb.add_worksheet();
                ws_cbe.set_name(&sanitize_sheet_name("01_Commercial_CBE"))?;
                ws_cbe.set_right_to_left(rtl);
                let _ = write_commercial_section(ws_cbe, result, &styles, rtl)?;
            }
            {
                let ws_tbe = wb.add_worksheet();
                ws_tbe.set_name(&sanitize_sheet_name("02_Technical_TBE"))?;
                ws_tbe.set_right_to_left(rtl);
                ws_tbe.set_column_width(0, 22.0)?;
                ws_tbe.set_column_width(1, 18.0)?;
                ws_tbe.set_column_width(2, 12.0)?;
                ws_tbe.set_column_width(3, 14.0)?;
                ws_tbe.set_column_width(4, 28.0)?;
                ws_tbe.set_column_width(5, 38.0)?;
                ws_tbe.set_column_width(6, 36.0)?;
                ws_tbe.set_column_width(7, 16.0)?;
                write_technical_section(ws_tbe, 0, result, &styles, rtl)?;
            }
            {
                let ws_notes = wb.add_worksheet();
                ws_notes.set_name(&sanitize_sheet_name("03_Audit_Notes"))?;
                ws_notes.set_right_to_left(rtl);
                ws_notes.set_column_width(0, 26.0)?;
                ws_notes.set_column_width(1, 56.0)?;
                ws_notes.write_string_with_format(0, 0, "Parameter", &styles.table_header)?;
                ws_notes.write_string_with_format(0, 1, "Value", &styles.table_header)?;
                ws_notes.write_string_with_format(1, 0, "Session ID", &styles.text_cell)?;
                ws_notes.write_string_with_format(1, 1, &result.session_id, &styles.text_cell)?;
                ws_notes.write_string_with_format(2, 0, "Synthetic BOQ Baseline", &styles.text_cell)?;
                ws_notes.write_string_with_format(
                    2,
                    1,
                    if result.master_boq.is_synthetic { "Yes" } else { "No" },
                    &styles.text_cell,
                )?;
                ws_notes.write_string_with_format(3, 0, "Merged Alignment Groups", &styles.text_cell)?;
                ws_notes.write_number_with_format(
                    3,
                    1,
                    result.alignment_groups.len() as f64,
                    &styles.text_center,
                )?;
            }
        }
    }

    Ok(wb)
}
