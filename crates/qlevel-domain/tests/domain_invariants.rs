//! Integration tests verifying `qlevel-domain` fixed-point math and invariants.

use qlevel_domain::{
    BoqItem, CurrencyCode, DecimalValue, DomainError, LineItemKind, MasterBoq, VatMode, VatPolicy,
};

#[test]
fn decimal_value_serializes_strictly_as_json_string() {
    let rate = DecimalValue::parse_clean("1,250.75").expect("valid decimal");
    let json = serde_json::to_string(&rate).expect("serialize");
    assert_eq!(json, "\"1250.75\"");

    let round_trip: DecimalValue = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(round_trip, rate);
}

#[test]
fn currency_scales_and_statutory_vat_rates_match_mena_rules() {
    assert_eq!(CurrencyCode::SAR.decimal_scale(), 2);
    assert_eq!(
        CurrencyCode::SAR.default_vat_pct(),
        DecimalValue::from_i64(15)
    );
    assert_eq!(CurrencyCode::EGP.decimal_scale(), 2);
    assert_eq!(
        CurrencyCode::EGP.default_vat_pct(),
        DecimalValue::from_i64(14)
    );
    assert_eq!(CurrencyCode::KWD.decimal_scale(), 3);
    assert_eq!(CurrencyCode::BHD.decimal_scale(), 3);
    assert_eq!(CurrencyCode::OMR.decimal_scale(), 3);
}

#[test]
fn inclusive_ksa_vat_stripping_recovers_exact_net_amount() {
    let policy = VatPolicy {
        project_vat_pct: DecimalValue::from_i64(15),
        default_vendor_mode: VatMode::Exclusive,
    };
    let gross = DecimalValue::parse_clean("1150.00").expect("gross");
    let net = policy
        .strip_to_net_exclusive(gross, VatMode::Inclusive, None)
        .expect("strip vat")
        .round_commercial(2);
    assert_eq!(net, DecimalValue::parse_clean("1000.00").expect("net"));
}

#[test]
fn master_boq_rejects_duplicate_ids_and_negative_quantities() {
    let item = BoqItem {
        id: "HVAC-01".to_owned(),
        code: "1.01".to_owned(),
        section: Some("Mechanical".to_owned()),
        description: "Supply and install chilled water fan coil unit 2.5 TR".to_owned(),
        unit: "No".to_owned(),
        quantity: DecimalValue::from_i64(12),
        kind: LineItemKind::Measured,
        pte_unit_rate: Some(DecimalValue::from_i64(4200)),
    };

    let dup_boq = MasterBoq {
        title: "QPAC MEP".to_owned(),
        currency: CurrencyCode::SAR,
        is_synthetic: false,
        items: vec![item.clone(), item],
    };
    assert!(matches!(
        dup_boq.validate(),
        Err(DomainError::DuplicateBoqId { .. })
    ));
}
