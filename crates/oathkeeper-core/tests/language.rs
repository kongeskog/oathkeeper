use oathkeeper_core::*;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn base() -> PathBuf {
    root().join("crates/oathkeeper-core/tests/fixtures/v03/lawlib/us/wa/seattle")
}
fn compiled(name: &str) -> Compilation {
    compile_file(base().join(format!("cases/{name}.case.oath"))).unwrap()
}
fn example() -> Bundle {
    compiled("long-excessive-fines").bundle().unwrap()
}
fn rejects(bundle: &Bundle, message: &str) {
    let error = verify(bundle).unwrap_err().to_string();
    assert!(
        error.contains(message),
        "expected {message:?}, received {error:?}"
    );
}

#[test]
fn migration_preserves_all_five_historical_derivations_and_assumptions() {
    let module = fs::read_to_string(base().join("parking.module.json")).unwrap();
    for name in [
        "long-excessive-fines",
        "long-homestead",
        "long-ability-to-pay",
        "long-statutory-framework",
        "long-no-automatic-impoundment-bar",
    ] {
        let old = verify(
            &assemble_case(
                &module,
                &fs::read_to_string(base().join(format!("cases/{name}.case.json"))).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let compilation = compiled(name);
        let new = compilation.verify().unwrap();
        for field in [
            "goal",
            "rules_used",
            "facts_used",
            "oracle_assumptions",
            "oracle_count",
            "fact_assumptions",
            "authorities_used",
            "authority_assessments_used",
            "scope_assumption_count",
            "unreviewed_authority_assessment_count",
        ] {
            assert_eq!(json!(old)[field], json!(new)[field], "{name}: {field}");
        }
        assert_eq!(compilation.lock.len(), 5);
        assert_eq!(compilation.module.rules.len(), 11);
        assert!(compilation
            .source_map
            .iter()
            .any(|s| s.kind == "step" && s.line > 1));
        assert!(compilation.source_map.iter().any(|s| s.kind == "premise"
            && s.id
                .starts_with("seattle.parking.72hours.ordinary-branch@2/")));
        // A JSON consumer uses the same independently callable kernel.
        assert_eq!(
            verify_text(
                &serde_json::to_string(&compilation.bundle().unwrap()).unwrap(),
                "json"
            )
            .unwrap()
            .bundle_sha256,
            new.bundle_sha256
        );
    }
}

#[test]
fn signatures_are_enforced_on_direct_json_and_cannot_be_downgraded() {
    let mut b = example();
    b.facts[0].claim.arguments.clear();
    rejects(&b, "E_ARITY");
    let mut b = example();
    b.constants[0].sort = Sort::Domain("Vehicle".into());
    rejects(&b, "E_TYPE");
    let mut b = example();
    b.predicates.clear();
    rejects(&b, "undeclared predicate");
    let mut b = example();
    b.types.clear();
    rejects(&b, "unknown domain type");
    let mut b = example();
    b.language = "oathkeeper/0.2".into();
    rejects(&b, "require oathkeeper/0.3");
    let mut b = example();
    b.predicates.push(Predicate {
        id: "oathkeeper.integer.gt".into(),
        arguments: vec![],
    });
    rejects(&b, "reserved predicate");
    let mut b = example();
    b.constants.push(b.constants[0].clone());
    rejects(&b, "duplicate identifier");
    let mut b = example();
    b.predicates
        .iter_mut()
        .find(|p| p.id == "sanction.qualifying_fine")
        .unwrap()
        .arguments[0] = Sort::Domain("Vehicle".into());
    let raw = serde_json::to_string(&b).unwrap();
    assert!(verify_text(&raw, "json").unwrap_err().0.contains("E_TYPE"));
}

#[test]
fn historical_findings_cannot_satisfy_a_different_matter() {
    let mut b = example();
    b.constants.push(Constant {
        id: "different.matter".into(),
        sort: Sort::Domain("Matter".into()),
    });
    let other = Term::Symbol {
        value: "different.matter".into(),
    };
    let last = b.proof.last_mut().unwrap();
    last.claim.arguments = vec![other.clone()];
    b.query.goal = last.claim.clone();
    if let Reason::Apply { substitution, .. } = &mut last.reason {
        substitution.insert("matter".into(), other);
    }
    rejects(&b, "premise claim mismatch");
}

#[test]
fn oracle_application_is_pending_until_an_actual_response_is_supplied() {
    let compilation = compiled("long-oracle-reassessment");
    assert!(!compilation
        .case
        .as_ref()
        .unwrap()
        .facts
        .iter()
        .any(|f| f.id == "disproportionality-finding"));
    let error = compilation.verify().unwrap_err().to_string();
    assert!(
        error.contains(".oath:")
            && error.contains("E_MISSING_EVIDENCE")
            && error.contains("unknown attestation")
    );
    let mut b = compilation.bundle().unwrap();
    let policy = b.oracle_policy[0].clone();
    // Synthetic unit-test response, not an assertion of a real court/reviewer agreement.
    b.attestations.push(Attestation {
        id: "required.disproportionality-attestation".into(),
        oracle: policy.oracle.clone(),
        provider: policy.providers[0].id.clone(),
        provider_version: policy.providers[0].version.clone(),
        question: policy.question.clone(),
        claim: Claim {
            atom: policy.atom,
            negative: false,
            arguments: policy.arguments,
        },
        confidence: Some(1.0),
        disposition: None,
        evidence_sha256: policy.evidence_sha256,
        recorded_at: b.query.known_at.clone(),
    });
    let report = verify(&b).unwrap();
    assert_eq!(report.oracle_count, 1);
    assert_eq!(report.oracle_requests_used.len(), 1);
    let mut duplicate = b.attestations[0].clone();
    duplicate.id = "another-output".into();
    duplicate.claim.negative = true;
    b.attestations.push(duplicate);
    rejects(&b, "E_ORACLE_CONFLICT");
    b.attestations.pop();
    b.attestations[0].confidence = Some(0.8);
    rejects(&b, "below agreed threshold");
    b.attestations[0].confidence = Some(1.0);
    b.oracle_policy[0].question.push_str(" changed");
    rejects(&b, "agreed request");
}

fn symbol(value: &str) -> Term {
    Term::Symbol {
        value: value.into(),
    }
}
fn claim(atom: &str, negative: bool, args: Vec<Term>) -> Claim {
    Claim {
        atom: atom.into(),
        negative,
        arguments: args,
    }
}
fn fact(id: &str, claim: Claim) -> Fact {
    let source = Source {
        citation: "Synthetic boundary-test premises; not a real parking case".into(),
        uri: "urn:oathkeeper:test:parking".into(),
        sha256: format!("{:x}", Sha256::digest(b"synthetic test context")),
    };
    Fact { id:id.into(), claim, evidence_sha256:source.sha256.clone(), recorded_at:"2026-10-18T21:32:50Z".into(),
        basis:Some(FactBasis { kind:FactKind::ScopeAssumption, source, pinpoint:"unit test".into(), explanation:"Synthetic input for testing the actual source-backed law, not an assertion about Long or any real person.".into() }) }
}
fn fact_step(f: &Fact) -> Step {
    Step {
        id: f.id.clone(),
        claim: f.claim.clone(),
        reason: Reason::Fact { fact: f.id.clone() },
    }
}
fn parking(hours: i64, excluded: bool) -> Bundle {
    let module = compile_file(base().join("parking.oath")).unwrap().module;
    let matter = symbol("test.matter");
    let vehicle = symbol("test.vehicle");
    let mut facts = vec![fact(
        "scope",
        claim("scope.smc72.historical-text", false, vec![matter.clone()]),
    )];
    let mut proof;
    if excluded {
        for (id, atom, args) in [
            (
                "vehicle",
                "parking.vehicle",
                vec![matter.clone(), vehicle.clone()],
            ),
            (
                "municipal",
                "parking.on_municipal_property",
                vec![matter.clone()],
            ),
            (
                "authorized",
                "parking.vehicle_authorized_on_municipal_property",
                vec![matter.clone(), vehicle.clone()],
            ),
        ] {
            facts.push(fact(id, claim(atom, false, args)));
        }
        proof = facts.iter().map(fact_step).collect::<Vec<_>>();
        proof.push(Step {
            id: "exclusion".into(),
            claim: claim("parking.smc72_c_excluded", false, vec![matter.clone()]),
            reason: Reason::Apply {
                rule: "seattle.parking.authorized-vehicle-exclusion".into(),
                version: "1".into(),
                premises: vec![
                    "scope".into(),
                    "vehicle".into(),
                    "municipal".into(),
                    "authorized".into(),
                ],
                substitution: BTreeMap::from([
                    ("matter".into(), matter.clone()),
                    ("vehicle".into(), vehicle),
                ]),
            },
        });
        proof.push(Step {
            id: "result".into(),
            claim: claim("parking.violates_smc72", true, vec![matter.clone()]),
            reason: Reason::Apply {
                rule: "seattle.parking.excluded-no-section-violation".into(),
                version: "1".into(),
                premises: vec!["scope".into(), "exclusion".into()],
                substitution: BTreeMap::from([("matter".into(), matter)]),
            },
        });
    } else {
        facts.push(fact(
            "location",
            claim(
                "parking.on_street_or_municipal_property",
                false,
                vec![matter.clone()],
            ),
        ));
        for (id, atom) in [
            ("shorter", "parking.shorter_posted_limit"),
            ("permission", "parking.otherwise_provided_by_law"),
            ("no-exclusion", "parking.smc72_c_excluded"),
            ("junk", "parking.junk_motor_vehicle"),
            ("abandoned", "parking.abandoned_junk_motor_vehicle"),
        ] {
            facts.push(fact(id, claim(atom, true, vec![matter.clone()])));
        }
        facts.push(fact(
            "duration",
            claim(
                "parking.duration_hours",
                false,
                vec![matter.clone(), Term::Integer { value: hours }],
            ),
        ));
        proof = facts.iter().map(fact_step).collect::<Vec<_>>();
        proof.push(Step {
            id: "boundary".into(),
            claim: claim(
                "oathkeeper.integer.gt",
                false,
                vec![Term::Integer { value: hours }, Term::Integer { value: 72 }],
            ),
            reason: Reason::Compare {
                operator: Comparison::GreaterThan,
            },
        });
        proof.push(Step {
            id: "result".into(),
            claim: claim("parking.violates_smc72", false, vec![matter.clone()]),
            reason: Reason::Apply {
                rule: "seattle.parking.72hours.ordinary-branch".into(),
                version: "2".into(),
                premises: proof.iter().map(|s| s.id.clone()).collect(),
                substitution: BTreeMap::from([
                    ("matter".into(), matter),
                    ("hours".into(), Term::Integer { value: hours }),
                ]),
            },
        });
    }
    module.with_case(CaseFile {
        retractions: vec![],
        constants: vec![
            Constant {
                id: "test.matter".into(),
                sort: Sort::Domain("Matter".into()),
            },
            Constant {
                id: "test.vehicle".into(),
                sort: Sort::Domain("Vehicle".into()),
            },
        ],
        facts,
        attestations: vec![],
        oracle_policy: vec![],
        query: Query {
            event_at: None,
            authority_at: None,
            purpose: None,
            temporal_rationale: None,
            jurisdiction: "us.wa.seattle".into(),
            forum: Some("us.wa.courts.state".into()),
            as_of: "2021-08-13T00:00:00Z".into(),
            known_at: "2026-10-18T21:32:50Z".into(),
            goal: proof.last().unwrap().claim.clone(),
            root: "result".into(),
        },
        proof,
    })
}

#[test]
fn actual_ordinance_has_explicit_applicability_exclusion_and_unknown_cases() {
    let ordinary = parking(73, false);
    assert!(verify(&ordinary).is_ok());
    let excluded = parking(73, true);
    let r = verify(&excluded).unwrap();
    assert!(r.goal.negative);
    assert_eq!(r.rules_used.len(), 2);
    for hours in [71, 72] {
        rejects(&parking(hours, false), "comparison is false");
    }
    let mut unknown = ordinary.clone();
    unknown.facts.retain(|f| f.id != "no-exclusion");
    rejects(&unknown, "E_MISSING_EVIDENCE");
    let mut no_proof = ordinary;
    no_proof.proof.clear();
    rejects(&no_proof, "E_NO_PROOF");
    let mut missing = excluded;
    if let Reason::Apply { premises, .. } = &mut missing.proof.last_mut().unwrap().reason {
        premises.pop();
    }
    rejects(&missing, "premise count mismatch");
}

#[test]
fn imports_do_not_override_legal_time_forum_or_authority_treatment() {
    let mut b = parking(73, false);
    b.query.as_of = "2000-01-01T00:00:00Z".into();
    rejects(&b, "outside jurisdiction, legal time");
    let mut b = parking(73, false);
    b.query.as_of = "2022-01-01T00:00:00Z".into();
    rejects(&b, "no authority assessment");
    let mut b = parking(73, false);
    b.query.known_at = "2021-08-13T00:00:00Z".into();
    rejects(&b, "knowledge cutoff");
    let mut b = parking(73, false);
    b.query.forum = Some("us.other.forum".into());
    rejects(&b, "no authority assessment");
    let mut b = parking(73, false);
    b.query.jurisdiction = "us.wa".into();
    rejects(&b, "outside jurisdiction");
    let mut b = parking(73, false);
    b.authority_assessments
        .iter_mut()
        .find(|a| a.rule == "seattle.parking.72hours.ordinary-branch")
        .unwrap()
        .treatment = Treatment::Repealed;
    rejects(&b, "not operative");
}

#[test]
fn two_supported_arguments_expose_a_conflict_without_selecting_a_winner() {
    let mut positive = parking(73, false);
    let mut negative = parking(73, true);
    let mut shared = positive.facts.clone();
    shared.extend(negative.facts.iter().filter(|f| f.id != "scope").cloned());
    positive.facts = shared.clone();
    negative.facts = shared;
    let comparison = compare_arguments(&positive, &negative).unwrap();
    assert_eq!(comparison.status, "conflicting_arguments");
    assert_eq!(comparison.arguments.len(), 2);
    negative.query.known_at = "2026-10-19T00:00:00Z".into();
    assert!(compare_arguments(&positive, &negative)
        .unwrap_err()
        .0
        .contains("same frozen"));
}

struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "oathkeeper-language-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
    fn write(&self, name: &str, text: &str) {
        fs::write(self.0.join(name), text).unwrap();
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn header(name: &str) -> String {
    format!("oathkeeper \"0.3\";\nmodule {name} @ \"1\";\n")
}
fn import(name: &str, content: &str) -> String {
    format!(
        "import {name:?} sha256 \"{:x}\";\n",
        Sha256::digest(content.as_bytes())
    )
}

#[test]
fn imports_are_transitively_locked_and_diamonds_are_deduplicated() {
    let t = Temp::new();
    let leaf = header("leaf") + "type Matter;\n";
    t.write("leaf.oath", &leaf);
    let a = header("a") + &import("leaf.oath", &leaf);
    t.write("a.oath", &a);
    let b = header("b") + &import("leaf.oath", &leaf);
    t.write("b.oath", &b);
    let entry = header("entry") + &import("a.oath", &a) + &import("b.oath", &b);
    t.write("entry.oath", &entry);
    let first = compile_file(t.0.join("entry.oath")).unwrap();
    assert_eq!(first.lock.len(), 4);
    assert_eq!(first.module.types, vec!["Matter"]);
    assert_eq!(
        first.snapshot_sha256,
        compile_file(t.0.join("entry.oath"))
            .unwrap()
            .snapshot_sha256
    );
    t.write(
        "leaf.oath",
        &(leaf + "// changed bytes under same version\n"),
    );
    assert!(compile_file(t.0.join("entry.oath"))
        .unwrap_err()
        .0
        .contains("E_IMPORT_HASH"));
}

#[test]
fn cycles_package_collisions_and_duplicate_declarations_fail() {
    let t = Temp::new();
    t.write("cycle.oath",&(header("cycle")+"import \"cycle.oath\" sha256 \"0000000000000000000000000000000000000000000000000000000000000000\";"));
    assert!(compile_file(t.0.join("cycle.oath"))
        .unwrap_err()
        .0
        .contains("E_IMPORT_CYCLE"));
    let a = header("same");
    let b = header("same") + "// other bytes\n";
    t.write("a.oath", &a);
    t.write("b.oath", &b);
    t.write(
        "root.oath",
        &(header("root") + &import("a.oath", &a) + &import("b.oath", &b)),
    );
    assert!(compile_file(t.0.join("root.oath"))
        .unwrap_err()
        .0
        .contains("duplicate package identity"));
    assert!(
        compile_text(&(header("test") + "type Matter; type Matter;"))
            .unwrap_err()
            .0
            .contains("E_NAMESPACE")
    );
}

#[test]
fn parser_reports_source_locations_and_rejects_ambiguous_or_unsupported_syntax() {
    for suffix in [
        "predicate broken(Matter;",
        "definition x {}",
        "type Matter; predicate p(Unknown);",
        "rule r(m: Matter) @ \"1\" { ensures p($m); ensures p($m); }",
    ] {
        assert!(
            compile_text(&(header("test") + suffix)).is_err(),
            "{suffix}"
        );
    }
    let error = compile_text(&(header("test") + "predicate broken(Matter;"))
        .unwrap_err()
        .0;
    assert!(error.contains("<input>:3:") && error.contains("E_PARSE"));
    let text = "oathkeeper \"0.3\"; case c @ \"1\"; type Matter;";
    assert!(compile_text(text)
        .unwrap_err()
        .0
        .contains("cannot redefine module"));
}
