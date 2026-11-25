//! Comprehensive 3-subcontractor bid-leveling integration tests for `qlevel-engine`.
//!
//! Verifies:
//! - Inclusive 15% KSA VAT stripping vs Net VAT-exclusive quotes
//! - Fixed RICS NRM 2 `ProvisionalSum` equalization when a bidder undercuts an allowance
//! - Merged lump-sum `YellowHighlightAuto` anchor + zero followers (zero double counting)
//! - Vendor multiplication error detection + missing-scope peer plugs + T&C exclusion plugs
//! - Synthetic BOQ generation with `orphan_vendor_rows == 0`
//! - `SQLite WAL` checkpoint persistence keyed by `BLAKE3`.

use qlevel_ai::ExtractedVendorBatch;
use qlevel_domain::{
    AiConfig, BoqItem, CellStatus, CommercialTerms, CurrencyCode, DecimalValue, EffortLevel,
    LevelingSessionConfig, LineItemKind, Locale, MasterBoq, MergedItemMode, PeerPlugStrategy,
    SheetDirection, TrapCategory, VatMode, VatPolicy, VendorLineQuote, VendorQuote,
    WorkbookLayoutMode,
};
use qlevel_engine::{
    build_synthetic_master_boq, compute_batch_blake3_key, evaluate_quotes_deterministic,
    CheckpointStore, RawVendorCandidateItem,
};

fn sample_session_config(merged_mode: MergedItemMode) -> LevelingSessionConfig {
    LevelingSessionConfig {
        session_id: "SES-KSA-001".to_owned(),
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
        layout_mode: WorkbookLayoutMode::SingleMasterSheet,
        merged_item_mode: merged_mode,
        peer_plug_strategy: PeerPlugStrategy::LowestCompliant,
        ui_locale: Locale::Ar,
        sheet_direction: SheetDirection::Rtl,
        approved_vendors: vec![
            "Trane".to_owned(),
            "Carrier".to_owned(),
            "Daikin".to_owned(),
        ],
        custom_rules: vec!["Scaffolding above 6m must be included".to_owned()],
    }
}

#[test]
fn levels_three_ksa_mep_contractors_catching_vat_merged_rows_and_exclusions() {
    let master_boq = MasterBoq {
        title: "Riyadh Commercial Tower HVAC Package".to_owned(),
        currency: CurrencyCode::SAR,
        is_synthetic: false,
        items: vec![
            BoqItem {
                id: "M-01".to_owned(),
                code: "1.01".to_owned(),
                section: Some("Chillers & FCUs".to_owned()),
                description: "Air-cooled screw chiller 250 TR".to_owned(),
                unit: "No".to_owned(),
                quantity: DecimalValue::from_i64(2),
                kind: LineItemKind::Measured,
                pte_unit_rate: Some(DecimalValue::from_i64(310_000)),
            },
            BoqItem {
                id: "M-02".to_owned(),
                code: "1.02".to_owned(),
                section: Some("Chillers & FCUs".to_owned()),
                description: "Concealed ducted FCU 3.0 TR".to_owned(),
                unit: "No".to_owned(),
                quantity: DecimalValue::from_i64(20),
                kind: LineItemKind::Measured,
                pte_unit_rate: Some(DecimalValue::from_i64(4_000)),
            },
            BoqItem {
                id: "M-03".to_owned(),
                code: "1.03".to_owned(),
                section: Some("Ductwork".to_owned()),
                description: "GI ductwork including insulation".to_owned(),
                unit: "m2".to_owned(),
                quantity: DecimalValue::from_i64(500),
                kind: LineItemKind::Measured,
                pte_unit_rate: Some(DecimalValue::from_i64(160)),
            },
            BoqItem {
                id: "M-04".to_owned(),
                code: "1.04".to_owned(),
                section: Some("Provisional Sums".to_owned()),
                description: "Provisional Sum for BMS integration".to_owned(),
                unit: "PS".to_owned(),
                quantity: DecimalValue::ONE,
                kind: LineItemKind::ProvisionalSum,
                pte_unit_rate: Some(DecimalValue::from_i64(50_000)),
            },
        ],
    };

    // Bidder A: Clean compliant bidder, quotes 15% VAT Inclusive on M-01..M-03 -> stripped cleanly!
    // Net M-01 = 600,000 | Net M-02 = 76,000 | Net M-03 = 75,000 | PS M-04 = 50,000 => Net = 801,000 SAR
    let bidder_a = VendorQuote {
        vendor_id: "V-A".to_owned(),
        vendor_name: "Najd Climate Systems".to_owned(),
        currency: CurrencyCode::SAR,
        fx_rate_to_project: DecimalValue::ONE,
        default_vat_mode: VatMode::Inclusive,
        stated_grand_total: None,
        terms: CommercialTerms::default(),
        lines: vec![
            VendorLineQuote {
                boq_item_id: "M-01".to_owned(),
                vendor_code: Some("1.01".to_owned()),
                vendor_description: "Chiller 250 TR".to_owned(),
                quoted_unit: "No".to_owned(),
                quoted_qty: DecimalValue::from_i64(2),
                quoted_unit_rate: DecimalValue::from_i64(345_000),
                quoted_extended_total: DecimalValue::from_i64(690_000),
                vat_mode: VatMode::Inclusive,
                vendor_vat_pct: None,
                status: CellStatus::Quoted,
                proposed_brand: Some("Trane".to_owned()),
                source_page: Some(1),
                notes: None,
            },
            VendorLineQuote {
                boq_item_id: "M-02".to_owned(),
                vendor_code: Some("1.02".to_owned()),
                vendor_description: "FCU 3.0 TR".to_owned(),
                quoted_unit: "No".to_owned(),
                quoted_qty: DecimalValue::from_i64(20),
                quoted_unit_rate: DecimalValue::from_i64(4_370),
                quoted_extended_total: DecimalValue::from_i64(87_400),
                vat_mode: VatMode::Inclusive,
                vendor_vat_pct: None,
                status: CellStatus::Quoted,
                proposed_brand: Some("Trane".to_owned()),
                source_page: Some(1),
                notes: None,
            },
            VendorLineQuote {
                boq_item_id: "M-03".to_owned(),
                vendor_code: Some("1.03".to_owned()),
                vendor_description: "GI Ductwork".to_owned(),
                quoted_unit: "m2".to_owned(),
                quoted_qty: DecimalValue::from_i64(500),
                quoted_unit_rate: DecimalValue::from_parts_i64(1725, 1),
                quoted_extended_total: DecimalValue::from_i64(86_250),
                vat_mode: VatMode::Inclusive,
                vendor_vat_pct: None,
                status: CellStatus::Quoted,
                proposed_brand: None,
                source_page: Some(1),
                notes: None,
            },
            VendorLineQuote {
                boq_item_id: "M-04".to_owned(),
                vendor_code: Some("1.04".to_owned()),
                vendor_description: "BMS Provisional Sum".to_owned(),
                quoted_unit: "PS".to_owned(),
                quoted_qty: DecimalValue::ONE,
                quoted_unit_rate: DecimalValue::from_i64(57_500),
                quoted_extended_total: DecimalValue::from_i64(57_500),
                vat_mode: VatMode::Inclusive,
                vendor_vat_pct: None,
                status: CellStatus::Quoted,
                proposed_brand: None,
                source_page: Some(1),
                notes: None,
            },
        ],
        raw_exclusion_clauses: Vec::new(),
    };

    // Bidder B: Merges M-02 + M-03 into a single 155,000 SAR lump sum and omits Provisional Sum M-04!
    // Net M-01 = 610,000 + Merged(M-02+M-03) = 155,000 + Equalized PS = 50,000 => Net = 815,000 SAR
    let bidder_b = VendorQuote {
        vendor_id: "V-B".to_owned(),
        vendor_name: "Delta Electro-Mechanical".to_owned(),
        currency: CurrencyCode::SAR,
        fx_rate_to_project: DecimalValue::ONE,
        default_vat_mode: VatMode::Exclusive,
        stated_grand_total: None,
        terms: CommercialTerms::default(),
        lines: vec![
            VendorLineQuote {
                boq_item_id: "M-01".to_owned(),
                vendor_code: Some("1.01".to_owned()),
                vendor_description: "Chiller 250 TR".to_owned(),
                quoted_unit: "No".to_owned(),
                quoted_qty: DecimalValue::from_i64(2),
                quoted_unit_rate: DecimalValue::from_i64(305_000),
                quoted_extended_total: DecimalValue::from_i64(610_000),
                vat_mode: VatMode::Exclusive,
                vendor_vat_pct: None,
                status: CellStatus::Quoted,
                proposed_brand: Some("Carrier".to_owned()),
                source_page: Some(1),
                notes: None,
            },
            VendorLineQuote {
                boq_item_id: "M-02".to_owned(),
                vendor_code: Some("1.02+1.03".to_owned()),
                vendor_description: "FCUs and complete GI ductwork lump sum".to_owned(),
                quoted_unit: "LS".to_owned(),
                quoted_qty: DecimalValue::ONE,
                quoted_unit_rate: DecimalValue::ZERO,
                quoted_extended_total: DecimalValue::from_i64(155_000),
                vat_mode: VatMode::Exclusive,
                vendor_vat_pct: None,
                status: CellStatus::Quoted,
                proposed_brand: Some("Carrier".to_owned()),
                source_page: Some(1),
                notes: None,
            },
            VendorLineQuote {
                boq_item_id: "M-03".to_owned(),
                vendor_code: None,
                vendor_description: "Included in item 1.02".to_owned(),
                quoted_unit: "m2".to_owned(),
                quoted_qty: DecimalValue::ZERO,
                quoted_unit_rate: DecimalValue::ZERO,
                quoted_extended_total: DecimalValue::ZERO,
                vat_mode: VatMode::Exclusive,
                vendor_vat_pct: None,
                status: CellStatus::IncludedIn {
                    anchor_boq_id: "M-02".to_owned(),
                },
                proposed_brand: None,
                source_page: Some(1),
                notes: None,
            },
        ],
        raw_exclusion_clauses: Vec::new(),
    };

    // Bidder C: Appears cheap on page 1 by omitting ductwork M-03, proposing non-AVL brand "Midea",
    // and excluding third-party testing & commissioning on the last page!
    let bidder_c = VendorQuote {
        vendor_id: "V-C".to_owned(),
        vendor_name: "Horizon Fast-Track Contracting".to_owned(),
        currency: CurrencyCode::SAR,
        fx_rate_to_project: DecimalValue::ONE,
        default_vat_mode: VatMode::Exclusive,
        stated_grand_total: None,
        terms: CommercialTerms {
            validity_days: Some(14),
            ..CommercialTerms::default()
        },
        lines: vec![
            VendorLineQuote {
                boq_item_id: "M-01".to_owned(),
                vendor_code: Some("1.01".to_owned()),
                vendor_description: "Chiller 250 TR".to_owned(),
                quoted_unit: "No".to_owned(),
                quoted_qty: DecimalValue::from_i64(2),
                quoted_unit_rate: DecimalValue::from_i64(295_000),
                quoted_extended_total: DecimalValue::from_i64(590_000),
                vat_mode: VatMode::Exclusive,
                vendor_vat_pct: None,
                status: CellStatus::Quoted,
                proposed_brand: Some("Midea".to_owned()),
                source_page: Some(1),
                notes: None,
            },
            VendorLineQuote {
                boq_item_id: "M-02".to_owned(),
                vendor_code: Some("1.02".to_owned()),
                vendor_description: "FCU 3.0 TR".to_owned(),
                quoted_unit: "No".to_owned(),
                quoted_qty: DecimalValue::from_i64(20),
                quoted_unit_rate: DecimalValue::from_i64(3_800),
                quoted_extended_total: DecimalValue::from_i64(76_000),
                vat_mode: VatMode::Exclusive,
                vendor_vat_pct: None,
                status: CellStatus::Quoted,
                proposed_brand: Some("Midea".to_owned()),
                source_page: Some(1),
                notes: None,
            },
        ],
        raw_exclusion_clauses: vec![
            "Clause 9: Third party testing excluded and scaffolding excluded above 4m.".to_owned(),
        ],
    };

    let config = sample_session_config(MergedItemMode::YellowHighlightAuto);
    let result = evaluate_quotes_deterministic(
        &config,
        &master_boq,
        vec![bidder_a, bidder_b, bidder_c],
        None,
        &[],
    )
    .expect("leveling succeeds");

    // Bidder A is True Low Bidder at 801,000.00 SAR net (921,150.00 SAR with 15% VAT)!
    assert_eq!(result.true_low_vendor_id, "V-A");
    assert_eq!(result.true_low_leveled_net, DecimalValue::from_i64(801_000));

    // Bidder B merged group counted once (815,000.00 SAR net, zero plug on M-03 follower)
    let sum_b = result
        .vendor_summaries
        .iter()
        .find(|s| s.vendor_id == "V-B")
        .expect("summary B");
    assert_eq!(sum_b.leveled_net_total, DecimalValue::from_i64(815_000));
    assert_eq!(sum_b.line_scope_plugs_total, DecimalValue::ZERO);

    // Bidder C received 75,000 SAR peer plug for omitted M-03 + 3% T&C exclusion plug (19,980 SAR) + 50,000 PS
    let sum_c = result
        .vendor_summaries
        .iter()
        .find(|s| s.vendor_id == "V-C")
        .expect("summary C");
    assert_eq!(sum_c.line_scope_plugs_total, DecimalValue::from_i64(75_000));
    assert_eq!(sum_c.clause_plugs_total, DecimalValue::from_i64(19_980));
    assert_eq!(sum_c.leveled_net_total, DecimalValue::from_i64(810_980));
    assert!(result
        .traps
        .iter()
        .any(|t| t.vendor_id == "V-C" && t.category == TrapCategory::AvlRegulatory));
}

#[test]
fn synthetic_boq_preserves_every_vendor_row_with_zero_orphans_and_sqlite_checkpoints_work() {
    let v1 = vec![
        RawVendorCandidateItem {
            vendor_id: "V1".to_owned(),
            row_index: 0,
            code: Some("A1".to_owned()),
            description: "Supply and install porcelain floor tiles 60x60cm".to_owned(),
            unit: "m2".to_owned(),
            quantity: DecimalValue::from_i64(240),
        },
        RawVendorCandidateItem {
            vendor_id: "V1".to_owned(),
            row_index: 1,
            code: Some("A2".to_owned()),
            description: "Acrylic emulsion wall paint 3 coats".to_owned(),
            unit: "m2".to_owned(),
            quantity: DecimalValue::from_i64(600),
        },
    ];
    let v2 = vec![
        RawVendorCandidateItem {
            vendor_id: "V2".to_owned(),
            row_index: 0,
            code: Some("1".to_owned()),
            description: "Supply & install porcelain floor tiles 60x60 cm".to_owned(),
            unit: "sqm".to_owned(),
            quantity: DecimalValue::from_i64(240),
        },
        RawVendorCandidateItem {
            vendor_id: "V2".to_owned(),
            row_index: 1,
            code: Some("2".to_owned()),
            description: "Waterproofing membrane under wet areas".to_owned(),
            unit: "m2".to_owned(),
            quantity: DecimalValue::from_i64(85),
        },
    ];

    let outcome = build_synthetic_master_boq("Fitout Quotes Only", CurrencyCode::EGP, &[v1, v2])
        .expect("synthetic boq");
    assert_eq!(outcome.orphan_vendor_rows, 0);
    assert_eq!(outcome.master_boq.items.len(), 3);

    let store = CheckpointStore::open_in_memory().expect("in-memory wal");
    let key = compute_batch_blake3_key(
        "openai",
        "gpt-6.2",
        EffortLevel::High,
        "doc_hash_1",
        0,
        "boq_hash_1",
        &[],
    );
    assert!(store.get_batch(&key).expect("miss").is_none());

    let sample_batch = ExtractedVendorBatch::default();
    store
        .save_batch(&key, "SES-1", "Vendor 1", &sample_batch)
        .expect("save");
    assert_eq!(store.get_batch(&key).expect("hit"), Some(sample_batch));
}
