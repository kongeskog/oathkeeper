use oathkeeper_core::*;
use serde_json::{json, Value};
use std::{collections::BTreeMap, path::PathBuf};

fn base() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../lawlib/us/wa/seattle")
}
fn historic() -> Bundle {
    compile_file(base().join("cases/long-excessive-fines.case.oath"))
        .unwrap()
        .bundle()
        .unwrap()
}
fn options() -> AnalysisOptions {
    AnalysisOptions {
        sensitivity: false,
        ..Default::default()
    }
}
fn source() -> Value {
    json!({"citation":"Synthetic adversarial test inputs, not a court case", "uri":"urn:oathkeeper:test:assessment", "sha256":"1".repeat(64)})
}
fn s(name: &str) -> Value {
    json!({"kind":"symbol","value":name})
}
fn c(atom: &str, negative: bool, args: Vec<Value>) -> Value {
    json!({"atom":atom,"negative":negative,"arguments":args})
}
fn fact(id: &str, claim: Value) -> Fact {
    serde_json::from_value(json!({"id":id,"claim":claim,"evidence_sha256":"1".repeat(64),"recorded_at":"2026-10-18T22:30:00Z","basis":{"kind":"scope_assumption","source":source(),"pinpoint":"unit-test input","explanation":"Synthetic assumption used to exercise the real sourced rule, not asserted about Long."}})).unwrap()
}
fn parking(duration: i64) -> Bundle {
    let module = compile_file(base().join("parking.oath")).unwrap().module;
    let mut facts = vec![
        fact(
            "scope",
            c("scope.smc72.historical-text", false, vec![s("test.matter")]),
        ),
        fact(
            "location",
            c(
                "parking.on_street_or_municipal_property",
                false,
                vec![s("test.matter")],
            ),
        ),
        fact(
            "duration",
            c(
                "parking.duration",
                false,
                vec![
                    s("test.matter"),
                    json!({"kind":"quantity","value":duration,"unit":"ms"}),
                ],
            ),
        ),
    ];
    for (id, p) in [
        ("shorter", "parking.shorter_posted_limit"),
        ("other", "parking.otherwise_provided_by_law"),
        ("excluded", "parking.smc72_c_excluded"),
        ("junk", "parking.junk_motor_vehicle"),
        ("abandoned", "parking.abandoned_junk_motor_vehicle"),
    ] {
        facts.push(fact(id, c(p, true, vec![s("test.matter")])));
    }
    let case:CaseFile=serde_json::from_value(json!({
        "constants":[{"id":"test.matter","sort":{"domain":"Matter"}},{"id":"test.vehicle","sort":{"domain":"Vehicle"}}],
        "facts":facts,"attestations":[],"oracle_policy":[],"proof":[],
        "query":{"jurisdiction":"us.wa.seattle","forum":"us.wa.courts.state","as_of":"2016-10-12T00:00:00-07:00","event_at":"2016-10-12","authority_at":"2021-08-13T00:00:00Z","known_at":"2026-10-19T00:00:00Z","purpose":"synthetic_test","temporal_rationale":"Synthetic test using explicitly selected historical law and authority times.","goal":c("parking.violates_smc72",false,vec![s("test.matter")]),"root":"result"}
    })).unwrap();
    module.with_case(case)
}
fn authorize(b: &mut Bundle) {
    b.facts.push(fact(
        "vehicle",
        c(
            "parking.vehicle",
            false,
            vec![s("test.matter"), s("test.vehicle")],
        ),
    ));
    b.facts.push(fact(
        "municipal",
        c(
            "parking.on_municipal_property",
            false,
            vec![s("test.matter")],
        ),
    ));
    b.facts.push(fact(
        "authorized",
        c(
            "parking.vehicle_authorized_on_municipal_property",
            false,
            vec![s("test.matter"), s("test.vehicle")],
        ),
    ));
}

#[test]
fn score_is_removed_and_zero_oracle_derivations_remain_conditional() {
    let report = verify(&historic()).unwrap();
    assert_eq!(report.status, "derivation_verified");
    assert_eq!(report.oracle_count, 0);
    assert!(report.judicial_finding_count > 0);
    assert_eq!(report.assurance.legal_fidelity, "not_established_by_kernel");
    let value = json!(report);
    assert!(value.get("verifiability").is_none());
    assert!(value.get("score_version").is_none());
}

#[test]
fn search_distinguishes_support_opposition_conflict_and_missing_evidence() {
    let b = parking(259200001);
    let result = analyze(&b, &options()).unwrap();
    assert_eq!(result.status, SupportStatus::Supported);
    assert!(result.search.complete_for_ground_domain);
    for a in &result.supporting {
        let mut replay = b.clone();
        replay.proof = a.proof.clone();
        assert_eq!(
            verify(&replay).unwrap().bundle_sha256,
            a.report.bundle_sha256
        );
    }
    let mut b = b.clone();
    b.facts.retain(|f| f.id != "excluded");
    let r = analyze(&b, &options()).unwrap();
    assert_eq!(r.status, SupportStatus::Neither);
    assert!(r
        .open_requirements
        .iter()
        .any(|g| g.claim.atom == "parking.smc72_c_excluded" && g.claim.negative));
    authorize(&mut b);
    assert_eq!(
        analyze(&b, &options()).unwrap().status,
        SupportStatus::Opposed
    );
    b.facts.push(fact(
        "excluded",
        c("parking.smc72_c_excluded", true, vec![s("test.matter")]),
    ));
    let both = analyze(&b, &options()).unwrap();
    assert_eq!(both.status, SupportStatus::Both);
    assert!(!both.supporting.is_empty() && !both.opposing.is_empty());
    assert!(both.legal_conclusion.starts_with("not_determined"));
}

#[test]
fn exact_duration_boundary_and_units_are_kernel_checked() {
    for (duration, expected) in [
        (259199999, SupportStatus::Neither),
        (259200000, SupportStatus::Neither),
        (259200001, SupportStatus::Supported),
        (261000000, SupportStatus::Supported),
    ] {
        assert_eq!(
            analyze(&parking(duration), &options()).unwrap().status,
            expected
        );
    }
    let mut b = parking(259200001);
    if let Term::Quantity { unit, .. } = &mut b.facts[2].claim.arguments[1] {
        *unit = "hours".into();
    }
    assert!(analyze(&b, &options()).unwrap_err().0.contains("E_TYPE"));
    let mut b = parking(259200001);
    b.proof = analyze(&b, &options()).unwrap().supporting[0].proof.clone();
    let comparison = b
        .proof
        .iter_mut()
        .find(|s| matches!(s.reason, Reason::Compare { .. }))
        .unwrap();
    comparison.claim.arguments[0] = Term::Quantity {
        unit: "ms".into(),
        value: 259200000,
    };
    assert!(verify(&b).unwrap_err().0.contains("comparison is false"));
}

#[test]
fn sensitivity_recomputes_alternative_arguments() {
    let mut b = parking(259200001);
    let mut duplicate = b.facts[0].clone();
    duplicate.id = "alternative-scope".into();
    b.facts.push(duplicate);
    let r = analyze(&b, &AnalysisOptions::default()).unwrap();
    assert!(r.sensitivity_complete);
    assert_eq!(
        r.sensitivity
            .iter()
            .find(|s| s.id == "scope")
            .unwrap()
            .without,
        SupportStatus::Supported
    );
    assert_eq!(
        r.sensitivity
            .iter()
            .find(|s| s.id == "duration")
            .unwrap()
            .without,
        SupportStatus::Neither
    );
}

#[test]
fn truncated_search_never_claims_exhaustion_or_a_negative_conclusion() {
    let r = analyze(
        &parking(259200001),
        &AnalysisOptions {
            max_nodes: 1,
            ..options()
        },
    )
    .unwrap();
    assert!(!r.search.complete_for_ground_domain);
    assert!(!r.search.limits_reached.is_empty());
    assert!(r.opposing.is_empty());
    assert!(r.legal_conclusion.starts_with("not_determined"));
}

#[test]
fn retractions_preserve_earlier_replay_and_do_not_prove_negation() {
    let mut b = historic();
    b.retractions.push(Retraction {
        id: "withdraw-finding".into(),
        target_kind: EvidenceKind::Fact,
        target: "disproportionality-finding".into(),
        recorded_at: "2026-10-19T00:00:00Z".into(),
        source: serde_json::from_value(source()).unwrap(),
        rationale: "Synthetic test withdrawal.".into(),
    });
    assert!(verify(&b).is_ok());
    b.query.known_at = "2026-10-19T00:00:00Z".into();
    assert!(verify(&b).unwrap_err().0.contains("E_RETRACTED_EVIDENCE"));
    let r = analyze(&b, &options()).unwrap();
    assert_eq!(r.status, SupportStatus::Neither);
    b.retractions[0].recorded_at = "2000-01-01T00:00:00Z".into();
    assert!(analyze(&b, &options()).unwrap_err().0.contains("predates"));
}

#[test]
fn law_time_and_authority_time_are_independent_and_explicit() {
    let mut b = historic();
    assert!(b.query.as_of.starts_with("2016"));
    assert!(verify(&b).unwrap().authority_at.starts_with("2021"));
    b.query.authority_at = Some("2015-01-01T00:00:00Z".into());
    assert!(verify(&b).is_err());
    b = historic();
    b.query.event_at = Some("2016-99-01".into());
    assert!(validate_context(&b).is_err());
    b = historic();
    b.rules[0].time_basis = None;
    assert!(validate_context(&b).unwrap_err().0.contains("time_basis"));
}

fn oracle_context() -> Bundle {
    let mut b = historic();
    b.facts.retain(|f| f.id != "disproportionality-finding");
    b.proof.clear();
    let claim = c(
        "sanction.grossly_disproportionate",
        false,
        vec![s("long.98824-2")],
    );
    b.oracle_policy=vec![serde_json::from_value(json!({"oracle":"review","atom":"sanction.grossly_disproportionate","arguments":[s("long.98824-2")],"question":"Synthetic protocol exercise over a real record; not a real opinion.","evidence_sha256":"1".repeat(64),"providers":[{"id":"test.a","version":"1"},{"id":"test.b","version":"1"}],"protocol":{"role":"legal_assessment","agreement":"unanimous","procedure":"Both named test providers must assert the same polarity; either may abstain."}})).unwrap()];
    b.attestations=vec![serde_json::from_value(json!({"id":"a","oracle":"review","provider":"test.a","provider_version":"1","question":b.oracle_policy[0].question,"claim":claim,"evidence_sha256":"1".repeat(64),"recorded_at":b.query.known_at,"disposition":"asserted"})).unwrap()];
    b
}

#[test]
fn oracle_unanimity_disagreement_abstention_and_missing_response_are_distinct() {
    let mut b = oracle_context();
    assert_eq!(
        analyze(&b, &options()).unwrap().oracle_requests[0].status,
        "pending"
    );
    let mut second = b.attestations[0].clone();
    second.id = "b".into();
    second.provider = "test.b".into();
    b.attestations.push(second);
    let r = analyze(&b, &options()).unwrap();
    assert_eq!(r.status, SupportStatus::Supported);
    assert_eq!(r.supporting[0].report.oracle_count, 2);
    assert_eq!(r.supporting[0].report.oracle_requests_used.len(), 1);
    b.attestations[1].claim.negative = true;
    let r = analyze(&b, &options()).unwrap();
    assert_eq!(r.status, SupportStatus::Neither);
    assert_eq!(r.oracle_requests[0].status, "disputed");
    b.attestations[1].disposition = Some(ResponseDisposition::Abstained);
    assert_eq!(
        analyze(&b, &options()).unwrap().oracle_requests[0].status,
        "abstained"
    );
    let mut duplicate = b.attestations[0].clone();
    duplicate.id = "a2".into();
    b.attestations.push(duplicate);
    assert!(analyze(&b, &options())
        .unwrap_err()
        .0
        .contains("E_ORACLE_CONFLICT"));
}

#[test]
fn confidence_requires_declared_meaning_and_cannot_replace_agreement() {
    let mut b = oracle_context();
    b.oracle_policy[0].min_confidence = Some(0.8);
    assert!(validate_context(&b)
        .unwrap_err()
        .0
        .contains("confidence_meaning"));
    b.oracle_policy[0]
        .protocol
        .as_mut()
        .unwrap()
        .confidence_meaning =
        Some("Uncalibrated synthetic test number, not a legal standard of proof.".into());
    let mut second = b.attestations[0].clone();
    second.id = "b".into();
    second.provider = "test.b".into();
    b.attestations.push(second);
    b.attestations[0].confidence = Some(1.0);
    assert_eq!(
        analyze(&b, &options()).unwrap().oracle_requests[0].status,
        "threshold_not_met"
    );
    b.attestations[1].confidence = Some(0.9);
    assert_eq!(
        analyze(&b, &options()).unwrap().status,
        SupportStatus::Supported
    );
    b.attestations[1].provider = "unagreed".into();
    assert!(analyze(&b, &options())
        .unwrap_err()
        .0
        .contains("not agreed"));
}

#[test]
fn real_record_pilot_exposes_missing_exceptions_and_has_no_fabricated_verdict() {
    let b = compile_file(base().join("cases/long-parking-evidence-review.case.oath"))
        .unwrap()
        .bundle()
        .unwrap();
    assert!(!b
        .facts
        .iter()
        .any(|f| f.claim.atom == "parking.violates_smc72"));
    let r = analyze(&b, &options()).unwrap();
    assert_eq!(r.status, SupportStatus::Neither);
    assert!(r
        .open_requirements
        .iter()
        .any(|g| g.claim.atom == "parking.otherwise_provided_by_law" && g.claim.negative));
    assert!(r.search.complete_for_ground_domain);
}

#[test]
fn compiler_rejects_bad_legal_metadata_and_accepts_quantity_surface_syntax() {
    let text = std::fs::read_to_string(base().join("parking.oath")).unwrap();
    assert!(text.contains("259200000[ms]"));
    let mut b = parking(259200001);
    b.rules[0].effective_from = "yesterday".into();
    assert!(validate_context(&b).unwrap_err().0.contains("RFC3339"));
    let mut b = parking(259200001);
    b.language = "oathkeeper/0.3".into();
    assert!(validate_context(&b).unwrap_err().0.contains("0.4 features"));
    // Even a valid built-in comparison cannot be injected as an unverified fact.
    let mut b = parking(259200001);
    let goal = Claim {
        atom: "oathkeeper.quantity.gt".into(),
        negative: false,
        arguments: vec![
            Term::Quantity {
                value: 1,
                unit: "ms".into(),
            },
            Term::Quantity {
                value: 0,
                unit: "ms".into(),
            },
        ],
    };
    b.facts.push(fact("fake-comparison", json!(goal)));
    b.query.goal = goal.clone();
    b.proof = vec![Step {
        id: b.query.root.clone(),
        claim: goal,
        reason: Reason::Fact {
            fact: "fake-comparison".into(),
        },
    }];
    assert!(verify(&b).unwrap_err().0.contains("checked comparisons"));
}

#[test]
fn search_rejects_invalid_context_instead_of_reporting_unknown() {
    let mut b = parking(259200001);
    b.constants.clear();
    assert!(analyze(&b, &options())
        .unwrap_err()
        .0
        .contains("undeclared constant"));
    let mut b = parking(259200001);
    b.proof = vec![Step {
        id: "irrelevant-bad-candidate".into(),
        claim: b.query.goal.clone(),
        reason: Reason::Apply {
            rule: "invented".into(),
            version: "1".into(),
            premises: vec![],
            substitution: BTreeMap::new(),
        },
    }];
    // Search constructs its own certificates; an old supplied proof is not a premise.
    assert_eq!(
        analyze(&b, &options()).unwrap().status,
        SupportStatus::Supported
    );
    assert!(verify(&b).is_err());
}

#[test]
fn resource_limits_and_reserved_quantity_type_are_rejected_before_search() {
    let mut b = parking(259200001);
    b.jurisdictions = vec![b.jurisdictions[0].clone(); 129];
    assert!(validate_context(&b)
        .unwrap_err()
        .to_string()
        .contains("E_LIMIT"));
    let mut b = parking(259200001);
    b.predicates[0].arguments = vec![Sort::Integer; 33];
    assert!(validate_context(&b)
        .unwrap_err()
        .to_string()
        .contains("E_LIMIT"));
    let mut b = parking(259200001);
    b.types.push("quantity".into());
    assert!(validate_context(&b)
        .unwrap_err()
        .to_string()
        .contains("reserved type"));
    assert!(parse(&" ".repeat(16 * 1024 * 1024 + 1), "json")
        .unwrap_err()
        .to_string()
        .contains("E_LIMIT"));
}
