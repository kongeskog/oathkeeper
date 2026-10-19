use oathkeeper_core::{
    tax::{assess, assess_json, BackdoorRothInput},
    *,
};
fn input() -> BackdoorRothInput {
    serde_json::from_str(include_str!(
        "../../../lawlib/us/tax/scenarios/synthetic-clean.json"
    ))
    .unwrap()
}
#[test]
fn no_pretax_balance_and_all_paths_are_kernel_checked() {
    let r = assess(&input()).unwrap();
    assert_eq!(r.status, "calculated");
    let a = r.allocation.unwrap();
    assert_eq!(a.nontaxable_conversion_cents, 700_000);
    assert_eq!(a.taxable_conversion_cents, 0);
    assert_eq!(a.remaining_basis_cents, 0);
    let report = verify(&r.certificate.unwrap()).unwrap();
    assert_eq!(report.status, "derivation_verified");
    assert_eq!(report.oracle_count, 0);
    assert!(report
        .authorities_used
        .iter()
        .all(|a| a.kind == AuthorityKind::AdministrativeGuidance));
    assert!(report
        .authority_assessments_used
        .iter()
        .all(|a| a.force == Force::Persuasive
            && a.review_status == ReviewStatus::MachineAssistedUnreviewed));
}
#[test]
fn aggregate_pretax_balances_and_earnings_change_taxable_amount() {
    let mut i = input();
    i.dec31_ira_value_cents = Some(6_300_000);
    let a = assess(&i).unwrap().allocation.unwrap();
    assert_eq!(a.nontaxable_conversion_cents, 70_000);
    assert_eq!(a.taxable_conversion_cents, 630_000);
    assert_eq!(a.remaining_basis_cents, 630_000);
    i.dec31_ira_value_cents = Some(0);
    i.roth_conversion_cents = Some(710_000);
    let a = assess(&i).unwrap().allocation.unwrap();
    assert_eq!(a.taxable_conversion_cents, 10_000);
}
#[test]
fn following_year_contribution_does_not_shelter_prior_year_conversion() {
    let mut i = input();
    i.contribution_made_in_2026_cents = Some(700_000);
    let a = assess(&i).unwrap().allocation.unwrap();
    assert_eq!(a.taxable_conversion_cents, 700_000);
    assert_eq!(a.remaining_basis_cents, 700_000);
    i.conversion_year = 2026;
    let r = assess(&i).unwrap();
    assert_eq!(r.status, "outside_scope");
    assert!(r.certificate.is_none());
}
#[test]
fn unknown_basis_and_balances_never_default_to_zero() {
    for field in [
        "prior_basis_cents",
        "dec31_ira_value_cents",
        "scope_confirmed",
    ] {
        let mut v = serde_json::to_value(input()).unwrap();
        v[field] = serde_json::Value::Null;
        let r: serde_json::Value =
            serde_json::from_str(&assess_json(&v.to_string()).unwrap()).unwrap();
        assert_eq!(r["status"], "needs_information");
        assert!(r["allocation"].is_null());
        assert!(r["certificate"].is_null());
    }
}
#[test]
fn cap_compensation_and_catchup_boundaries() {
    let mut i = input();
    i.other_regular_ira_contributions_cents = Some(1);
    assert_eq!(assess(&i).unwrap().status, "outside_scope");
    i.age_at_year_end = Some(50);
    assert_eq!(assess(&i).unwrap().status, "calculated");
    i.other_regular_ira_contributions_cents = Some(100_000);
    assert_eq!(assess(&i).unwrap().status, "calculated");
    i.other_regular_ira_contributions_cents = Some(100_001);
    assert_eq!(assess(&i).unwrap().status, "outside_scope");
    i.other_regular_ira_contributions_cents = Some(0);
    i.taxable_compensation_cents = Some(699_999);
    assert_eq!(assess(&i).unwrap().status, "outside_scope");
    i.taxable_compensation_cents = Some(700_000);
    assert_eq!(assess(&i).unwrap().status, "calculated");
}
#[test]
fn losses_cap_ratio_and_preserve_basis() {
    let mut i = input();
    i.roth_conversion_cents = Some(600_000);
    let a = assess(&i).unwrap().allocation.unwrap();
    assert_eq!(a.taxable_conversion_cents, 0);
    assert_eq!(a.remaining_basis_cents, 100_000);
}
#[test]
fn prior_basis_is_used_and_rounding_is_explicit() {
    let mut i = input();
    i.nondeductible_contribution_cents = Some(0);
    i.prior_basis_cents = Some(1);
    i.roth_conversion_cents = Some(1);
    i.dec31_ira_value_cents = Some(1);
    let a = assess(&i).unwrap().allocation.unwrap();
    assert_eq!(a.nontaxable_conversion_cents, 1);
    assert_eq!(a.remaining_basis_cents, 0);
    i.dec31_ira_value_cents = Some(2);
    let a = assess(&i).unwrap().allocation.unwrap();
    assert_eq!(a.nontaxable_conversion_cents, 0);
    assert_eq!(a.remaining_basis_cents, 1);
}
#[test]
fn rejects_invalid_units_false_arithmetic_and_fact_forgery() {
    let r = assess(&input()).unwrap();
    let b = r.certificate.unwrap();
    let idx = b
        .proof
        .iter()
        .position(|s| {
            matches!(
                s.reason,
                Reason::Compute {
                    operator: Computation::QuantityAllocate
                }
            )
        })
        .unwrap();
    let mut altered = b.clone();
    altered.proof[idx].claim.arguments[3] = Term::Quantity {
        value: 1,
        unit: "usd_cent".into(),
    };
    assert!(verify(&altered).is_err());
    let mut altered = b.clone();
    altered.proof[idx].claim.arguments[0] = Term::Quantity {
        value: 700_000,
        unit: "usd".into(),
    };
    assert!(verify(&altered).is_err());
    let mut altered = b.clone();
    let mut fact = altered.facts[0].clone();
    fact.id = "forged".into();
    fact.claim = altered.proof[idx].claim.clone();
    altered.facts.push(fact);
    altered.proof[idx].reason = Reason::Fact {
        fact: "forged".into(),
    };
    assert!(verify(&altered).is_err());
    let mut altered = b;
    altered.authorities[0].publication = Publication::Enacted;
    assert!(verify(&altered).is_err());
}
#[test]
fn bounds_scope_and_unknown_fields_fail_closed() {
    let mut i = input();
    i.scope_confirmed = Some(false);
    assert_eq!(assess(&i).unwrap().status, "outside_scope");
    i = input();
    i.nondeductible_contribution_cents = Some(-1);
    assert!(assess(&i).is_err());
    i = input();
    i.prior_basis_cents = Some(i64::MAX);
    assert!(assess(&i).is_err());
    i = input();
    i.contribution_made_in_2026_cents = Some(700_001);
    assert!(assess(&i).is_err());
    i = input();
    i.roth_conversion_cents = Some(0);
    assert_eq!(assess(&i).unwrap().status, "outside_scope");
    let mut v = serde_json::to_value(input()).unwrap();
    v["rules"] = serde_json::json!([]);
    assert!(assess_json(&v.to_string()).is_err());
}
#[test]
fn official_shank_record_reconstructs_basis_and_taxable_distribution() {
    for name in [
        "shank-2018.case.oath",
        "shank-unavailable-returns.case.oath",
    ] {
        let c = compile_file(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../lawlib/us/tax")
                .join(name),
        )
        .unwrap();
        let r = c.verify().unwrap();
        assert_eq!(r.oracle_count, 0);
        assert_eq!(r.purpose, Some(CasePurpose::HistoricalReconstruction));
    }
}
