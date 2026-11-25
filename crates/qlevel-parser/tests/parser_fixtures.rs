//! Integration tests for `qlevel-parser`: Excel hidden-row detection, Arabic BOQ headers,
//! clipboard TSV/HTML parsing, and versioned fallback paths.

use std::path::Path;

use qlevel_domain::{CurrencyCode, DecimalValue, LineItemKind};
use qlevel_parser::{
    extract_master_boq_from_sheet, normalize_arabic_digits, parse_clipboard_table,
    parse_excel_bytes, suggest_unlocked_version_path,
};
use rust_xlsxwriter::Workbook;

#[test]
fn detects_hidden_excel_rows_and_extracts_arabic_master_boq() {
    let mut wb = Workbook::new();
    let ws = wb.add_worksheet();
    ws.set_name("المقايسة").expect("sheet name");

    ws.write_string(0, 0, "رقم البند").expect("h0");
    ws.write_string(0, 1, "بيان الأعمال").expect("h1");
    ws.write_string(0, 2, "الوحدة").expect("h2");
    ws.write_string(0, 3, "الكمية").expect("h3");
    ws.write_string(0, 4, "سعر الوحدة").expect("h4");

    ws.write_string(1, 0, "1.1").expect("r1");
    ws.write_string(1, 1, "توريد وتركيب وحدات تكييف مخفي").expect("r1");
    ws.write_string(1, 2, "عدد").expect("r1");
    ws.write_string(1, 3, "١٢").expect("r1");
    ws.write_number(1, 4, 4500.0).expect("r1");

    // Hidden trap row inside Excel
    ws.set_row_hidden(2).expect("hide row");
    ws.write_string(2, 0, "1.2").expect("r2");
    ws.write_string(2, 1, "بند مخفي").expect("r2");
    ws.write_string(2, 2, "م2").expect("r2");
    ws.write_number(2, 3, 99.0).expect("r2");

    ws.write_string(3, 0, "1.3").expect("r3");
    ws.write_string(3, 1, "مبلغ احتياطي لأعمال الاختبارات").expect("r3");
    ws.write_string(3, 2, "PS").expect("r3");
    ws.write_number(3, 3, 1.0).expect("r3");
    ws.write_number(3, 4, 25000.0).expect("r3");

    let xlsx_bytes = wb.save_to_buffer().expect("save xlsx");
    let doc = parse_excel_bytes("Master_BOQ_MEP.xlsx", &xlsx_bytes).expect("parse excel");

    assert_eq!(doc.sheets.len(), 1);
    assert_eq!(doc.sheets[0].hidden_row_numbers, vec![3]);

    let boq = extract_master_boq_from_sheet("MEP Tender", CurrencyCode::SAR, &doc.sheets[0])
        .expect("extract boq");
    assert_eq!(boq.items.len(), 2);
    assert_eq!(boq.items[0].quantity, DecimalValue::from_i64(12));
    assert_eq!(boq.items[1].kind, LineItemKind::ProvisionalSum);
}

#[test]
fn parses_clipboard_tsv_and_arabic_indic_digits() {
    assert_eq!(normalize_arabic_digits("١٬٢٥٠٫٥٠"), "1,250.50");

    let tsv = "Item\tDescription\tUnit\tQty\tRate\nA-01\tGypsum Board Ceiling\tm2\t340\t95.50\n";
    let doc = parse_clipboard_table("Supplier_A_Clipboard", None, Some(tsv)).expect("tsv");
    assert_eq!(doc.total_tabular_rows(), 2);
}

#[test]
fn suggests_incremented_unlocked_filename() {
    let fallback = suggest_unlocked_version_path(Path::new("Comparison_Report.xlsx"));
    assert_eq!(fallback, Path::new("./Comparison_Report_v2.xlsx"));
}
