use oathkeeper_core::*;

fn example() -> Bundle {
    parse(include_str!("../../../examples/hoa.oath"), "oath").unwrap()
}

fn rejected(mut change: impl FnMut(&mut Bundle), message: &str) {
    let mut bundle = example();
    change(&mut bundle);
    let error = verify(&bundle).unwrap_err().to_string();
    assert!(
        error.contains(message),
        "expected {message:?}, got {error:?}"
    );
}

#[test]
fn verifies_hierarchy_and_exposes_assumptions() {
    let report = verify(&example()).unwrap();
    assert_eq!(report.applicable_jurisdictions.len(), 4);
    assert_eq!(report.rules_used.len(), 4);
    assert_eq!(report.facts_used, ["notice"]);
    assert_eq!(report.oracle_assumptions, ["color-assessment-1"]);
    assert_eq!(report.oracle_count, 1);
    assert_eq!(report.status, "derivation_verified");
}

#[test]
fn deterministic_and_transport_independent() {
    let bundle = example();
    let json = serde_json::to_string(&bundle).unwrap();
    let first = verify(&bundle).unwrap();
    assert_eq!(
        first.bundle_sha256, "8749fa9fb7f0858eb1e1bcc35cf837f0b4cf89a3686da086c1bf52842d2924a2",
        "0.1 snapshot identity must survive additive schema extensions"
    );
    let second = verify_text(&json, "json").unwrap();
    assert_eq!(first.bundle_sha256, second.bundle_sha256);
    assert_eq!(
        serde_json::to_string(&first).unwrap(),
        serde_json::to_string(&second).unwrap()
    );
}

#[test]
fn zero_oracle_proof_retains_fact_assumptions() {
    let mut bundle = example();
    bundle.proof.truncate(3);
    bundle.query.root = "state".into();
    bundle.query.goal = bundle.proof[2].claim.clone();
    let report = verify(&bundle).unwrap();
    assert_eq!(report.assurance.legal_fidelity, "not_established_by_kernel");
    assert_eq!(report.oracle_count, 0);
    assert_eq!(report.facts_used.len(), 1);
    assert_eq!(report.status, "derivation_verified");
}

#[test]
fn multiple_distinct_oracles_are_counted_without_a_score() {
    let mut bundle = example();
    let mut a = bundle.attestations[0].clone();
    a.id = "second-attestation".into();
    a.claim.atom = "case.second_opinion".into();
    a.oracle = "second-oracle".into();
    let mut policy = bundle.oracle_policy[0].clone();
    policy.oracle = a.oracle.clone();
    policy.atom = a.claim.atom.clone();
    bundle.oracle_policy.push(policy);
    bundle.attestations.push(a.clone());
    bundle.rules[3].premises.push(a.claim.clone());
    let index = bundle.proof.len() - 1;
    bundle.proof.insert(
        index,
        Step {
            id: "second".into(),
            claim: a.claim,
            reason: Reason::Oracle { attestation: a.id },
        },
    );
    if let Reason::Apply { premises, .. } = &mut bundle.proof[index + 1].reason {
        premises.push("second".into());
    }
    let report = verify(&bundle).unwrap();
    assert_eq!(report.oracle_count, 2);
    assert_eq!(report.assurance.evidence_authenticity, "not_authenticated");
}

#[test]
fn shared_oracle_dependency_is_counted_once() {
    let mut bundle = example();
    bundle.rules[3]
        .premises
        .push(bundle.attestations[0].claim.clone());
    if let Reason::Apply { premises, .. } = &mut bundle.proof[5].reason {
        premises.push("color".into());
    }
    assert_eq!(verify(&bundle).unwrap().oracle_count, 1);
}

#[test]
fn oracle_agreement_is_enforced() {
    rejected(|b| b.oracle_policy.clear(), "no agreement");
    rejected(
        |b| b.attestations[0].provider = "unapproved".into(),
        "not agreed",
    );
    rejected(
        |b| b.attestations[0].provider_version = "new-model".into(),
        "not agreed",
    );
    rejected(
        |b| b.attestations[0].question = "A different question?".into(),
        "agreed request",
    );
    rejected(
        |b| b.attestations[0].evidence_sha256 = "a".repeat(64),
        "agreed request",
    );
    rejected(
        |b| b.oracle_policy[0].atom = "case.different".into(),
        "agreed request",
    );
    rejected(
        |b| b.attestations[0].confidence = Some(0.89),
        "below agreed threshold",
    );
    rejected(|b| b.attestations[0].confidence = Some(f64::NAN), "finite");
    rejected(|b| b.oracle_policy[0].min_confidence = Some(-0.1), "finite");
    rejected(|b| b.attestations[0].confidence = Some(1.01), "finite");
}

#[test]
fn legal_time_and_knowledge_time_are_independent() {
    rejected(
        |b| b.query.as_of = "2024-12-31T23:59:59Z".into(),
        "outside jurisdiction",
    );
    rejected(
        |b| b.rules[0].effective_until = Some(b.query.as_of.clone()),
        "outside jurisdiction",
    );
    rejected(
        |b| b.rules[0].recorded_at = "2025-04-03T00:00:00Z".into(),
        "outside jurisdiction",
    );
    rejected(
        |b| b.facts[0].recorded_at = "2025-04-03T00:00:00Z".into(),
        "knowledge cutoff",
    );
    rejected(
        |b| b.attestations[0].recorded_at = "2025-04-03T00:00:00Z".into(),
        "knowledge cutoff",
    );
    rejected(
        |b| b.rules[0].effective_until = Some(b.rules[0].effective_from.clone()),
        "validity interval",
    );
    let mut bundle = example();
    bundle.query.as_of = "2025-01-01T00:00:00Z".into();
    // Later-acquired evidence can support a retrospective query.
    assert!(verify(&bundle).is_ok());
    bundle.rules[0].effective_from = "2024-12-31T19:00:00-05:00".into();
    assert!(verify(&bundle).is_ok());
}

#[test]
fn jurisdiction_dag_is_validated_and_no_child_override_is_inferred() {
    rejected(
        |b| b.query.jurisdiction = "us".into(),
        "outside jurisdiction",
    );
    rejected(
        |b| {
            let child = b.jurisdictions[3].id.clone();
            b.jurisdictions[0].parents.push(child);
        },
        "jurisdiction cycle",
    );
    rejected(
        |b| b.jurisdictions[0].parents.push("missing".into()),
        "unknown jurisdiction",
    );
    rejected(
        |b| b.jurisdictions.push(b.jurisdictions[0].clone()),
        "duplicate identifier",
    );
    rejected(
        |b| b.rules[0].jurisdiction = "absent".into(),
        "unknown jurisdiction",
    );
    let mut bundle = example();
    bundle.jurisdictions.push(Jurisdiction {
        id: "other-root".into(),
        name: "Another legal system".into(),
        parents: vec![],
    });
    bundle.jurisdictions[3].parents.push("other-root".into());
    assert_eq!(verify(&bundle).unwrap().applicable_jurisdictions.len(), 5);
}

#[test]
fn historical_versions_resolve_without_silent_shadowing() {
    rejected(
        |b| b.rules.push(b.rules[0].clone()),
        "duplicate rule version",
    );
    rejected(
        |b| {
            let mut next = b.rules[0].clone();
            next.version = "2".into();
            b.rules.push(next);
        },
        "ambiguous active versions",
    );
    let mut bundle = example();
    let mut historical = bundle.rules[0].clone();
    historical.version = "historical".into();
    historical.effective_from = "2020-01-01T00:00:00Z".into();
    historical.effective_until = Some("2025-01-01T00:00:00Z".into());
    bundle.rules.push(historical);
    assert!(verify(&bundle).is_ok());
}

#[test]
fn forged_proofs_and_unused_padding_are_rejected() {
    rejected(
        |b| b.proof[0].claim.atom = "forged".into(),
        "fact claim mismatch",
    );
    rejected(
        |b| b.proof[3].claim.negative = true,
        "oracle claim mismatch",
    );
    rejected(
        |b| b.proof[5].claim.atom = "forged".into(),
        "conclusion mismatch",
    );
    rejected(
        |b| b.query.goal.atom = "not-proved".into(),
        "does not establish",
    );
    rejected(|b| b.query.root = "absent".into(), "missing proof root");
    rejected(|b| b.proof.push(b.proof[0].clone()), "duplicate proof step");
    rejected(
        |b| {
            let mut padding = b.proof[0].clone();
            padding.id = "padding".into();
            b.proof.push(padding);
        },
        "outside the goal",
    );
    rejected(
        |b| {
            if let Reason::Apply { premises, .. } = &mut b.proof[1].reason {
                premises[0] = "approval".into();
            }
        },
        "earlier step",
    );
    rejected(
        |b| {
            if let Reason::Apply { premises, .. } = &mut b.proof[5].reason {
                premises.clear();
            }
        },
        "premise count",
    );
    rejected(
        |b| {
            if let Reason::Apply { premises, .. } = &mut b.proof[5].reason {
                premises[0] = "notice".into();
            }
        },
        "premise claim mismatch",
    );
}

#[test]
fn explicit_contradictions_fail_instead_of_exploding() {
    rejected(
        |b| {
            let mut opposite = b.facts[0].clone();
            opposite.id = "opposite".into();
            opposite.claim.negative = true;
            b.proof.push(Step {
                id: "opposite".into(),
                claim: opposite.claim.clone(),
                reason: Reason::Fact {
                    fact: opposite.id.clone(),
                },
            });
            b.facts.push(opposite);
        },
        "conflicting proof claims",
    );
    let mut bundle = example();
    bundle.proof.truncate(1);
    bundle.proof[0].claim.negative = true;
    bundle.facts[0].claim.negative = true;
    bundle.query.goal = bundle.proof[0].claim.clone();
    bundle.query.root = "notice".into();
    assert!(verify(&bundle).is_ok());
}

#[test]
fn strict_inputs_prevent_ambiguous_interpretation() {
    rejected(
        |b| b.language = "oathkeeper/99".into(),
        "unsupported language",
    );
    rejected(|b| b.facts[0].evidence_sha256 = "invalid".into(), "SHA-256");
    rejected(|b| b.query.as_of = "2025-04-01".into(), "RFC3339");
    rejected(
        |b| b.facts[0].claim.atom = "ambiguous atom".into(),
        "invalid identifier",
    );
    let mut value = serde_json::to_value(example()).unwrap();
    value["silently_ignored"] = serde_json::json!(true);
    assert!(parse(&value.to_string(), "json").is_err());
    assert!(parse("{}", "unknown").is_err());
}
