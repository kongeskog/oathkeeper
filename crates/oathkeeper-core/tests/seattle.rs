use oathkeeper_core::*;
use std::collections::BTreeMap;

const MODULE: &str = include_str!("fixtures/v03/lawlib/us/wa/seattle/parking.module.json");
const EXCESSIVE: &str =
    include_str!("fixtures/v03/lawlib/us/wa/seattle/cases/long-excessive-fines.case.json");

fn example() -> Bundle {
    assemble_case(MODULE, EXCESSIVE).unwrap()
}
fn rejected(change: impl FnOnce(&mut Bundle), message: &str) {
    let mut b = example();
    change(&mut b);
    let error = verify(&b).unwrap_err().to_string();
    assert!(
        error.contains(message),
        "expected {message:?}, got {error:?}"
    );
}

#[test]
fn actual_long_case_has_traceable_statute_and_two_precedents() {
    let report = verify(&example()).unwrap();
    assert_eq!(report.rules_used.len(), 5);
    assert_eq!(report.authorities_used.len(), 3);
    assert_eq!(report.authority_assessments_used.len(), 5);
    assert_eq!(report.unreviewed_authority_assessment_count, 5);
    assert_eq!(report.goal.atom, "constitutional.excessive_fines_violation");
    assert_eq!(report.oracle_count, 0);
    assert!(report
        .fact_assumptions
        .iter()
        .any(|f| f.id == "disproportionality-finding"
            && f.basis.as_ref().unwrap().kind == FactKind::JudicialFinding));
    // A derivation still exposes the court's recorded evaluative findings.
    assert!(!report.fact_assumptions.is_empty());
}

#[test]
fn other_real_long_holdings_verify() {
    for text in [
        include_str!("fixtures/v03/lawlib/us/wa/seattle/cases/long-statutory-framework.case.json"),
        include_str!("fixtures/v03/lawlib/us/wa/seattle/cases/long-ability-to-pay.case.json"),
        include_str!("fixtures/v03/lawlib/us/wa/seattle/cases/long-homestead.case.json"),
        include_str!(
            "fixtures/v03/lawlib/us/wa/seattle/cases/long-no-automatic-impoundment-bar.case.json"
        ),
    ] {
        assert!(verify(&assemble_case(MODULE, text).unwrap()).is_ok());
    }
}

#[test]
fn precedent_requires_scope_source_forum_and_historical_assessment() {
    rejected(
        |b| {
            b.rules[5].premises.remove(0);
        },
        "scope must be an explicit",
    );
    rejected(
        |b| b.rules[5].authority.as_mut().unwrap().pinpoint.clear(),
        "pinpoint",
    );
    rejected(
        |b| b.rules[5].source.sha256 = "a".repeat(64),
        "source does not match",
    );
    rejected(
        |b| b.query.forum = Some("us.ca.courts.state".into()),
        "no authority assessment",
    );
    rejected(
        |b| b.query.as_of = "2026-10-18T00:00:00Z".into(),
        "no authority assessment",
    );
    rejected(
        |b| b.authority_assessments.clear(),
        "no authority assessment",
    );
    rejected(
        |b| b.authorities[1].publication = Publication::Unpublished,
        "unpublished case",
    );
    rejected(
        |b| b.authorities[1].decided_at = Some("2022-01-01T00:00:00Z".into()),
        "not yet decided",
    );
    rejected(
        |b| b.rules[5].authority.as_mut().unwrap().role = AuthorityRole::Text,
        "role does not match",
    );
}

#[test]
fn latest_visible_negative_treatment_blocks_a_holding() {
    // Synthetic treatment mutation: this is NOT a claim that Long was overruled.
    let mut b = example();
    let prior = b
        .authority_assessments
        .iter()
        .find(|a| a.rule == "wa.long.gross-disproportionality")
        .unwrap()
        .clone();
    let mut correction = prior.clone();
    correction.id = "synthetic.correction".into();
    correction.recorded_at = "2026-10-20T00:00:00Z".into();
    correction.treatment = Treatment::Overruled;
    b.authority_assessments.insert(0, correction);
    assert!(verify(&b).is_ok(), "future correction is not yet visible");
    b.query.known_at = "2026-10-21T00:00:00Z".into();
    assert!(verify(&b)
        .unwrap_err()
        .to_string()
        .contains("not operative"));
    b.authority_assessments.reverse();
    assert!(verify(&b)
        .unwrap_err()
        .to_string()
        .contains("not operative"));
}

#[test]
fn equal_time_authority_assessments_are_ambiguous() {
    rejected(
        |b| {
            let mut duplicate = b.authority_assessments[0].clone();
            duplicate.id = "duplicate-assessment".into();
            b.authority_assessments.push(duplicate);
        },
        "ambiguous authority",
    );
}

#[test]
fn superseded_assessment_ties_do_not_make_selection_order_dependent() {
    let mut b = example();
    let mut old = b.authority_assessments[0].clone();
    old.id = "synthetic.old".into();
    old.recorded_at = "2021-08-14T00:00:00Z".into();
    let mut tied = old.clone();
    tied.id = "synthetic.old-tie".into();
    // The actual assessment is newer than both synthetic records.
    b.authority_assessments.insert(0, old);
    b.authority_assessments.insert(1, tied);
    assert!(verify(&b).is_ok());
    b.authority_assessments.reverse();
    assert!(verify(&b).is_ok());
}

#[test]
fn substitutions_cannot_change_the_case_or_smuggle_free_variables() {
    rejected(
        |b| {
            if let Reason::Apply { substitution, .. } = &mut b.proof.last_mut().unwrap().reason {
                substitution.insert(
                    "matter".into(),
                    Term::Symbol {
                        value: "different-person".into(),
                    },
                );
            }
        },
        "conclusion mismatch",
    );
    rejected(
        |b| {
            if let Reason::Apply { substitution, .. } = &mut b.proof.last_mut().unwrap().reason {
                substitution.clear();
            }
        },
        "bind exactly",
    );
    rejected(
        |b| {
            if let Reason::Apply { substitution, .. } = &mut b.proof.last_mut().unwrap().reason {
                substitution.insert("matter".into(), Term::Integer { value: 1 });
            }
        },
        "sort mismatch",
    );
    rejected(
        |b| {
            b.proof[0].claim.arguments[0] = Term::Variable {
                name: "matter".into(),
            }
        },
        "ground terms",
    );
    rejected(
        |b| b.rules[5].parameters.clear(),
        "undeclared rule variable",
    );
}

#[test]
fn evidence_provenance_is_required_and_not_reclassified_silently() {
    rejected(|b| b.facts[0].basis = None, "provenance basis");
    rejected(
        |b| b.facts[0].basis.as_mut().unwrap().source.sha256 = "a".repeat(64),
        "does not match provenance",
    );
    let mut b = example();
    b.facts[0].basis.as_mut().unwrap().kind = FactKind::ScopeAssumption;
    assert_eq!(verify(&b).unwrap().scope_assumption_count, 1);
}

#[test]
fn cases_cannot_override_the_library() {
    let mut case: serde_json::Value = serde_json::from_str(EXCESSIVE).unwrap();
    case["rules"] = serde_json::json!([]);
    assert!(assemble_case(MODULE, &case.to_string()).is_err());
}

#[test]
fn integer_comparisons_check_the_72_hour_boundary_and_polarity() {
    for (hours, negative) in [(71, true), (72, true), (73, false)] {
        let mut b = example();
        let claim = Claim {
            atom: "oathkeeper.integer.gt".into(),
            negative,
            arguments: vec![Term::Integer { value: hours }, Term::Integer { value: 72 }],
        };
        b.proof = vec![Step {
            id: "comparison".into(),
            claim: claim.clone(),
            reason: Reason::Compare {
                operator: Comparison::GreaterThan,
            },
        }];
        b.query.root = "comparison".into();
        b.query.goal = claim;
        assert!(verify(&b).is_ok());
        b.proof[0].claim.negative = !negative;
        b.query.goal = b.proof[0].claim.clone();
        assert!(verify(&b)
            .unwrap_err()
            .to_string()
            .contains("comparison is false"));
    }
}

#[test]
fn ordinary_parking_rule_is_reusable_but_does_not_invent_exception_facts() {
    let b = example();
    let rule = b
        .rules
        .iter()
        .find(|r| r.id == "seattle.parking.72hours.ordinary-branch")
        .unwrap();
    assert_eq!(rule.parameters.len(), 2);
    assert!(rule
        .premises
        .iter()
        .any(|p| p.atom == "oathkeeper.integer.gt"));
    assert!(!b
        .facts
        .iter()
        .any(|f| f.claim.atom == "scope.smc72.ordinary-branch"));
    let substitution = BTreeMap::from([
        (
            "matter".into(),
            Term::Symbol {
                value: "long.98824-2".into(),
            },
        ),
        ("hours".into(), Term::Integer { value: 73 }),
    ]);
    let mut attempted = b.clone();
    attempted.query.goal = Claim {
        atom: "parking.violates_smc72".into(),
        negative: false,
        arguments: vec![Term::Symbol {
            value: "long.98824-2".into(),
        }],
    };
    attempted.query.root = "unsupported".into();
    attempted.proof = vec![Step {
        id: "unsupported".into(),
        claim: attempted.query.goal.clone(),
        reason: Reason::Apply {
            rule: rule.id.clone(),
            version: "1".into(),
            premises: vec![],
            substitution,
        },
    }];
    assert!(verify(&attempted)
        .unwrap_err()
        .to_string()
        .contains("premise count"));
}

#[test]
fn oracle_agreement_binds_subject_arguments_in_addition_to_question() {
    let mut b = example();
    let original = b.facts.remove(0);
    b.attestations.push(Attestation {
        id: "scope-oracle".into(),
        oracle: "scope-oracle".into(),
        provider: "test-provider".into(),
        provider_version: "1".into(),
        question: "Synthetic scope test".into(),
        claim: original.claim.clone(),
        confidence: Some(0.9),
        disposition: None,
        evidence_sha256: original.evidence_sha256.clone(),
        recorded_at: original.recorded_at.clone(),
    });
    b.oracle_policy.push(OraclePolicy {
        oracle: "scope-oracle".into(),
        atom: original.claim.atom.clone(),
        arguments: original.claim.arguments.clone(),
        question: "Synthetic scope test".into(),
        evidence_sha256: original.evidence_sha256,
        providers: vec![OracleProvider {
            id: "test-provider".into(),
            version: "1".into(),
        }],
        min_confidence: Some(0.8),
        protocol: None,
    });
    b.proof[0].reason = Reason::Oracle {
        attestation: "scope-oracle".into(),
    };
    assert_eq!(verify(&b).unwrap().oracle_count, 1);
    b.oracle_policy[0].arguments = vec![Term::Symbol {
        value: "another-case".into(),
    }];
    assert!(verify(&b)
        .unwrap_err()
        .to_string()
        .contains("agreed request"));
}
