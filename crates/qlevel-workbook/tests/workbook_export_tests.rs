//! Integration tests for `qlevel-workbook`: verifies `SingleMasterSheet` & `DualTab` generation,
//! zero `_xlfn.` dynamic array functions in XML, `rightToLeft="1"` on Arabic BOQs, and `<= 31` char tab names.

use std::io::{Cursor, Read};

use qlevel_domain::{
    AiConfig, BoqItem, CellStatus, CurrencyCode, DecimalValue, EffortLevel, LeveledLineCell,
    LevelingResult, LevelingSessionConfig, LineItemKind, Locale, MasterBoq, MergedItemMode,
    PeerPlugStrategy, SheetDirection, VatMode, VatPolicy, VendorEvaluationSummary,
    WorkbookLayoutMode,
};
use qlevel_workbook::{
    col_idx_to_excel_letters, export_leveling_workbook_bytes, sanitize_sheet_name,
    validate_excel_2016_formula,
};
use zip::ZipArchive;

fn sample_result(layout_mode: WorkbookLayoutMode) -> LevelingResult {
    let config = LevelingSessionConfig {
        session_id: "SES-WB-01".to_owned(),
        ai: AiConfig {
            provider: None,
            model_id: None,
            effort: EffortLevel::Auto,
        },
        currency: CurrencyCode::SAR,
        vat_policy: VatPolicy {
            project_vat_pct: DecimalValue::from_i64(15),
            default_vendor_mode: VatMode::Exclusive,
        },
        layout_mode,
        merged_item_mode: MergedItemMode::YellowHighlightAuto,
        peer_plug_strategy: PeerPlugStrategy::LowestCompliant,
        ui_locale: Locale::Ar,
        sheet_direction: SheetDirection::Auto,
        approved_vendors: Vec::new(),
        custom_rules: Vec::new(),
    };

    let master_boq = MasterBoq {
        title: "مقايسة أعمال التكييف".to_owned(),
        currency: CurrencyCode::SAR,
        is_synthetic: false,
        items: vec![BoqItem {
            id: "B-1".to_owned(),
            code: "1.1".to_owned(),
            section: Some("تكييف".to_owned()),
            description: "توريد وتركيب وحدة تكييف مخفي".to_owned(),
            unit: "عدد".to_owned(),
            quantity: DecimalValue::from_i64(10),
            kind: LineItemKind::Measured,
            pte_unit_rate: Some(DecimalValue::from_i64(4000)),
        }],
    };

    LevelingResult {
        session_id: "SES-WB-01".to_owned(),
        config,
        master_boq,
        cells: vec![LeveledLineCell {
            boq_item_id: "B-1".to_owned(),
            vendor_id: "V-1".to_owned(),
            status: CellStatus::Quoted,
            net_unit_rate: DecimalValue::from_i64(3900),
            as_submitted_net_total: DecimalValue::from_i64(39000),
            arithmetic_delta: DecimalValue::ZERO,
            scope_plug_amount: DecimalValue::ZERO,
            leveled_net_total: DecimalValue::from_i64(39000),
            highlight_yellow: false,
            audit_note: None,
        }],
        alignment_groups: Vec::new(),
        traps: Vec::new(),
        vendor_summaries: vec![VendorEvaluationSummary {
            vendor_id: "V-1".to_owned(),
            vendor_name: "مؤسسة نجد للتكييف".to_owned(),
            as_submitted_total: DecimalValue::from_i64(39000),
            arithmetic_and_qty_delta: DecimalValue::ZERO,
            discount_adjustment: DecimalValue::ZERO,
            provisional_sum_equalized_total: DecimalValue::ZERO,
            line_scope_plugs_total: DecimalValue::ZERO,
            clause_plugs_total: DecimalValue::ZERO,
            leveled_net_total: DecimalValue::from_i64(39000),
            vat_amount: DecimalValue::from_i64(5850),
            leveled_gross_total: DecimalValue::from_i64(44850),
            variance_vs_lowest_pct: DecimalValue::ZERO,
            quoted_items_count: 1,
            unquoted_items_count: 0,
            merged_items_count: 0,
            traps_count: 0,
            rank: 1,
            is_true_low_bidder: true,
        }],
        true_low_vendor_id: "V-1".to_owned(),
        true_low_vendor_name: "مؤسسة نجد للتكييف".to_owned(),
        true_low_leveled_net: DecimalValue::from_i64(39000),
    }
}

#[test]
fn exports_both_layouts_with_rtl_and_zero_xlfn_formulas() {
    for mode in [
        WorkbookLayoutMode::SingleMasterSheet,
        WorkbookLayoutMode::DualTab,
    ] {
        let bytes = export_leveling_workbook_bytes(&sample_result(mode)).expect("xlsx export");
        let mut archive = ZipArchive::new(Cursor::new(bytes)).expect("valid zip");

        let mut sheet1_xml = String::new();
        archive
            .by_name("xl/worksheets/sheet1.xml")
            .expect("sheet1 exists")
            .read_to_string(&mut sheet1_xml)
            .expect("read sheet1");

        assert!(!sheet1_xml.contains("_xlfn."));
        assert!(sheet1_xml.contains("rightToLeft=\"1\""));
        assert!(sheet1_xml.contains("ROUND("));
        assert!(sheet1_xml.contains("SUM("));
    }
}

#[test]
fn enforces_31_char_sheet_names_and_rejects_modern_dynamic_formulas() {
    assert_eq!(col_idx_to_excel_letters(0), "A");
    assert_eq!(col_idx_to_excel_letters(25), "Z");
    assert_eq!(col_idx_to_excel_letters(26), "AA");

    let sanitized = sanitize_sheet_name("01: Very/Long*Sheet[Name]?Exceeding_Thirty_One_Chars");
    assert!(sanitized.encode_utf16().count() <= 31);
    assert!(!sanitized.contains(':'));
    assert!(!sanitized.contains('/'));

    assert!(validate_excel_2016_formula("=SUM(H10:H50)").is_ok());
    assert!(validate_excel_2016_formula("=_xlfn.XLOOKUP(A2,B:B,C:C)").is_err());
}
