//! Offline end-to-end golden fixture tests verifying KSA (15% VAT) & Egypt (14% VAT) tenders
//! without requiring live cloud API keys.

use std::fs;
use std::path::PathBuf;

use qlevel_domain::{
    DecimalValue, LevelingSessionConfig, MasterBoq, TrapCategory, VendorQuote,
};
use qlevel_engine::evaluate_quotes_deterministic;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GoldenFixture {
    expected_true_low_vendor_id: String,
    expected_true_low_leveled_net: DecimalValue,
    config: LevelingSessionConfig,
    master_boq: MasterBoq,
    quotes: Vec<VendorQuote>,
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures")
        .join(name)
}

#[test]
fn golden_ksa_mep_fixture_levels_inclusive_vat_and_catches_exclusions() {
    let raw = fs::read_to_string(fixture_path("golden_ksa_mep.json")).expect("read ksa fixture");
    let fx: GoldenFixture = serde_json::from_str(&raw).expect("parse ksa fixture");

    let res = evaluate_quotes_deterministic(&fx.config, &fx.master_boq, fx.quotes, None, &[])
        .expect("level ksa");

    assert_eq!(res.true_low_vendor_id, fx.expected_true_low_vendor_id);
    assert_eq!(res.true_low_leveled_net, fx.expected_true_low_leveled_net);
    assert!(res
        .traps
        .iter()
        .any(|t| t.vendor_id == "V-GULF" && t.category == TrapCategory::AvlRegulatory));
    assert!(res
        .traps
        .iter()
        .any(|t| t.vendor_id == "V-GULF" && t.category == TrapCategory::LogisticsIncoterms));
}

#[test]
fn golden_egy_fitout_fixture_catches_multiplication_error_and_preserves_merged_lump_sum() {
    let raw =
        fs::read_to_string(fixture_path("golden_egy_fitout.json")).expect("read egy fixture");
    let fx: GoldenFixture = serde_json::from_str(&raw).expect("parse egy fixture");

    let res = evaluate_quotes_deterministic(&fx.config, &fx.master_boq, fx.quotes, None, &[])
        .expect("level egy");

    // V-CAIRO printed 476,000 EGP via a 23,000 EGP multiplication error (true net = 499,000 EGP),
    // making V-NILE (488,000 EGP merged lump sum) the True Low Bidder!
    assert_eq!(res.true_low_vendor_id, fx.expected_true_low_vendor_id);
    assert_eq!(res.true_low_leveled_net, fx.expected_true_low_leveled_net);
    assert!(res
        .traps
        .iter()
        .any(|t| t.vendor_id == "V-CAIRO" && t.category == TrapCategory::QtyArithmetic));
}
