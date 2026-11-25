//! Technical Bid Evaluation (`TBE`) auditor enforcing the 7 built-in trap rules
//! plus user-defined `custom_rules` and Approved Vendor List (`AVL`) checks.

use qlevel_domain::{
    DecimalValue, DetectedTrap, LevelingSessionConfig, PlugBasis, TrapCategory, TrapSeverity,
    VendorQuote,
};

struct BuiltinClausePattern {
    category: TrapCategory,
    severity: TrapSeverity,
    title: &'static str,
    keywords: &'static [&'static str],
    default_pct_plug: Option<i64>,
}

const BUILTIN_PATTERNS: &[BuiltinClausePattern] = &[
    BuiltinClausePattern {
        category: TrapCategory::ScopeExclusion,
        severity: TrapSeverity::High,
        title: "Scaffolding & Access Equipment Excluded",
        keywords: &[
            "scaffolding excluded",
            "exclude scaffolding",
            "scaffolding by others",
            "السقالات غير مشمولة",
            "بدون سقالات",
            "السقالات على المقاول العام",
        ],
        default_pct_plug: None,
    },
    BuiltinClausePattern {
        category: TrapCategory::ScopeExclusion,
        severity: TrapSeverity::High,
        title: "Civil & Builders Work Excluded",
        keywords: &[
            "civil work excluded",
            "builders work by others",
            "chasing and plastering excluded",
            "الأعمال المدنية غير مشمولة",
            "التكسير والترميم على المقاول",
        ],
        default_pct_plug: None,
    },
    BuiltinClausePattern {
        category: TrapCategory::CloseoutDocs,
        severity: TrapSeverity::High,
        title: "Third-Party Testing & Commissioning Excluded",
        keywords: &[
            "third party testing excluded",
            "testing and commissioning excluded",
            "t&c excluded",
            "civil defense approval excluded",
            "اختبارات الطرف الثالث غير مشمولة",
            "بدون اختبارات التشغيل",
        ],
        default_pct_plug: Some(3),
    },
    BuiltinClausePattern {
        category: TrapCategory::LogisticsIncoterms,
        severity: TrapSeverity::High,
        title: "Non-DDP Delivery Term Exposure (EXW / FOB / CIF)",
        keywords: &[
            "ex-works",
            "exw ",
            "fob ",
            "customs clearance excluded",
            "تسليم أرض المصنع",
            "غير شامل التخليص الجمركي",
        ],
        default_pct_plug: Some(5),
    },
    BuiltinClausePattern {
        category: TrapCategory::CloseoutDocs,
        severity: TrapSeverity::Medium,
        title: "Shop Drawings / BIM / As-Built Documentation Excluded",
        keywords: &[
            "shop drawings excluded",
            "bim excluded",
            "as-built excluded",
            "بدون مخططات تنفيذية",
            "المخططات التنفيذية غير مشمولة",
        ],
        default_pct_plug: None,
    },
];

/// Evaluates all 7 built-in technical/commercial rules, AVL compliance, and `custom_rules`.
#[must_use]
pub fn evaluate_technical_traps(
    config: &LevelingSessionConfig,
    quotes: &[VendorQuote],
    existing_ai_traps: &[DetectedTrap],
) -> Vec<DetectedTrap> {
    let mut traps: Vec<DetectedTrap> = existing_ai_traps.to_vec();

    let normalized_avl: Vec<String> = config
        .approved_vendors
        .iter()
        .map(|b| b.trim().to_lowercase())
        .filter(|b| !b.is_empty())
        .collect();

    for quote in quotes {
        // 1. Scan raw exclusion clauses against deterministic built-in patterns
        for (clause_idx, clause) in quote.raw_exclusion_clauses.iter().enumerate() {
            let lower_clause = clause.to_lowercase();
            for pat in BUILTIN_PATTERNS {
                if pat.keywords.iter().any(|kw| lower_clause.contains(kw)) {
                    let already_present = traps.iter().any(|t| {
                        t.vendor_id == quote.vendor_id
                            && t.category == pat.category
                            && t.source_snippet == *clause
                    });
                    if !already_present {
                        let plug_basis = pat.default_pct_plug.map(|pct_int| PlugBasis::PctOfQuotedOnly {
                            pct: DecimalValue::from_i64(pct_int),
                        });
                        traps.push(DetectedTrap {
                            id: format!("TRAP-TBE-{}-{clause_idx}", quote.vendor_id),
                            vendor_id: quote.vendor_id.clone(),
                            category: pat.category,
                            severity: pat.severity,
                            related_boq_id: None,
                            title: pat.title.to_owned(),
                            detail: format!("Detected exclusion clause in {} quotation", quote.vendor_name),
                            source_snippet: clause.clone(),
                            source_page: None,
                            plug_basis,
                            computed_impact: None,
                        });
                    }
                }
            }
        }

        // 2. Commercial validity & Incoterms checks
        if let Some(validity) = quote.terms.validity_days {
            if validity < 30 {
                traps.push(DetectedTrap {
                    id: format!("TRAP-VAL-{}", quote.vendor_id),
                    vendor_id: quote.vendor_id.clone(),
                    category: TrapCategory::CommercialRisk,
                    severity: TrapSeverity::Medium,
                    related_boq_id: None,
                    title: "Short Offer Validity Window".to_owned(),
                    detail: format!(
                        "Quotation validity is only {validity} days (standard tender minimum is 60-90 days)"
                    ),
                    source_snippet: format!("Validity: {validity} days"),
                    source_page: None,
                    plug_basis: None,
                    computed_impact: None,
                });
            }
        }

        if let Some(incoterm) = &quote.terms.delivery_incoterms {
            let upper = incoterm.trim().to_uppercase();
            if upper.starts_with("EXW") || upper.starts_with("FOB") {
                let already = traps.iter().any(|t| {
                    t.vendor_id == quote.vendor_id && t.category == TrapCategory::LogisticsIncoterms
                });
                if !already {
                    traps.push(DetectedTrap {
                        id: format!("TRAP-INCO-{}", quote.vendor_id),
                        vendor_id: quote.vendor_id.clone(),
                        category: TrapCategory::LogisticsIncoterms,
                        severity: TrapSeverity::High,
                        related_boq_id: None,
                        title: format!("Delivery Quoted as {upper} Instead of DDP Site"),
                        detail: "Freight, insurance, or customs duties excluded from unit rates".to_owned(),
                        source_snippet: incoterm.clone(),
                        source_page: None,
                        plug_basis: Some(PlugBasis::PctOfQuotedOnly {
                            pct: DecimalValue::from_i64(5),
                        }),
                        computed_impact: None,
                    });
                }
            }
        }

        // 3. Approved Vendor List (AVL) brand audit on proposed line brands
        if !normalized_avl.is_empty() {
            for line in &quote.lines {
                let Some(brand) = &line.proposed_brand else {
                    continue;
                };
                let lower_brand = brand.trim().to_lowercase();
                if lower_brand.is_empty() {
                    continue;
                }
                let approved = normalized_avl
                    .iter()
                    .any(|ok| lower_brand.contains(ok) || ok.contains(&lower_brand));
                if !approved {
                    traps.push(DetectedTrap {
                        id: format!("TRAP-AVL-{}-{}", quote.vendor_id, line.boq_item_id),
                        vendor_id: quote.vendor_id.clone(),
                        category: TrapCategory::AvlRegulatory,
                        severity: TrapSeverity::High,
                        related_boq_id: Some(line.boq_item_id.clone()),
                        title: format!("Non-AVL Brand Proposed: {brand}"),
                        detail: format!(
                            "Proposed brand '{brand}' is not in the project Approved Vendor List ({})",
                            config.approved_vendors.join(", ")
                        ),
                        source_snippet: line.vendor_description.clone(),
                        source_page: line.source_page,
                        plug_basis: None,
                        computed_impact: None,
                    });
                }
            }
        }

        // 4. Custom project rules check against exclusion clauses
        for (rule_idx, custom_rule) in config.custom_rules.iter().enumerate() {
            let trimmed_rule = custom_rule.trim();
            if trimmed_rule.is_empty() {
                continue;
            }
            // Flag if vendor exclusions explicitly mention key tokens of a custom rule negatively
            let rule_tokens: Vec<String> = trimmed_rule
                .to_lowercase()
                .split_whitespace()
                .filter(|w| w.chars().count() >= 5)
                .map(ToOwned::to_owned)
                .collect();

            for clause in &quote.raw_exclusion_clauses {
                let lc = clause.to_lowercase();
                let is_negative = lc.contains("exclud")
                    || lc.contains("not included")
                    || lc.contains("غير مشمول")
                    || lc.contains("بدون");
                if is_negative && rule_tokens.iter().any(|tok| lc.contains(tok)) {
                    traps.push(DetectedTrap {
                        id: format!("TRAP-CUST-{}-{rule_idx}", quote.vendor_id),
                        vendor_id: quote.vendor_id.clone(),
                        category: TrapCategory::TechnicalDeviation,
                        severity: TrapSeverity::High,
                        related_boq_id: None,
                        title: "Custom Project Rule Violation".to_owned(),
                        detail: format!("Conflicts with custom rule: {trimmed_rule}"),
                        source_snippet: clause.clone(),
                        source_page: None,
                        plug_basis: None,
                        computed_impact: None,
                    });
                }
            }
        }
    }

    traps
}
