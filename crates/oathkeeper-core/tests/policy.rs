use ed25519_dalek::{Signer, SigningKey};
use oathkeeper_core::{contracts::*, evaluation::*, receipts::*, *};
use proptest::prelude::*;
use serde_json::json;
use std::path::Path;

fn historical() -> Bundle {
    compile_file(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../.deps/lawlib/us/wa/seattle/cases/long-ability-to-pay.case.oath"),
    )
    .unwrap()
    .bundle()
    .unwrap()
}
fn scoped() -> Bundle {
    let mut b = historical();
    b.language = "oathkeeper/0.5".into();
    b.query.coverage = Some("test.bounded".into());
    let scope = b
        .facts
        .iter()
        .find(|f| f.claim.atom == "scope.long.federal-excessive-fines")
        .unwrap()
        .claim
        .clone();
    b.coverage = vec![CoverageContract {
        id: "test.bounded".into(),
        question: "Synthetic engineering contract over the recorded matter".into(),
        conclusion_kind: ConclusionKind::Obligation,
        parameters: vec![],
        goal: b.query.goal.clone(),
        jurisdictions: vec![b.query.jurisdiction.clone()],
        forums: vec![b.query.forum.clone().unwrap()],
        law_from: "2016-01-01T00:00:00Z".into(),
        law_until: "2017-01-01T00:00:00Z".into(),
        requirements: vec![CoverageRequirement {
            id: "historical-scope".into(),
            goal: scope,
            explanation: "Explicit interpretation assumption".into(),
        }],
        source_inventory: vec![CoverageSource {
            source: b.authorities[0].source.clone(),
            treatment: "Engineering fixture only".into(),
        }],
        exclusions: vec!["All other matters".into()],
        known_omissions: vec!["Not a reviewed legal coverage claim".into()],
    }];
    b
}
fn condition_proofs(b: &Bundle) -> Vec<ConditionProof> {
    assess_coverage(b, &AnalysisOptions::default())
        .unwrap()
        .conditions
        .into_iter()
        .map(|c| {
            let a = &c.supporting[0];
            ConditionProof {
                id: c.id,
                root: a.proof.last().unwrap().id.clone(),
                proof: a.proof.clone(),
            }
        })
        .collect()
}
#[test]
fn scoped_proofs_bind_each_condition_to_the_same_context() {
    let b = scoped();
    let mut p = condition_proofs(&b);
    assert!(verify_scoped(&b, &p).is_ok());
    assert!(verify_scoped(&b, &[]).is_err());
    p[0].proof[0].claim.negative = true;
    assert!(verify_scoped(&b, &p).is_err());
    let mut foreign = b.clone();
    foreign.query.as_of = "2025-12-31T00:00:00Z".into();
    assert!(verify_scoped(&foreign, &condition_proofs(&b)).is_err());
    assert_eq!(
        assess_coverage(&foreign, &AnalysisOptions::default())
            .unwrap()
            .status,
        "outside_scope"
    );
}
#[test]
fn scope_unknown_opposed_and_disputed_are_distinct() {
    let mut b = scoped();
    let goal = b.coverage[0].requirements[0].goal.clone();
    assert_eq!(
        assess_coverage(&b, &AnalysisOptions::default())
            .unwrap()
            .status,
        "conditions_supported"
    );
    let index = b.facts.iter().position(|f| f.claim == goal).unwrap();
    let mut contrary = b.facts[index].clone();
    contrary.id = "test.contrary".into();
    contrary.claim.negative = true;
    b.facts.push(contrary.clone());
    assert_eq!(
        assess_coverage(&b, &AnalysisOptions::default())
            .unwrap()
            .status,
        "conflicting_scope_evidence"
    );
    b.facts.remove(index);
    assert_eq!(
        assess_coverage(&b, &AnalysisOptions::default())
            .unwrap()
            .status,
        "outside_scope"
    );
    b.facts.retain(|f| f.id != contrary.id);
    assert_eq!(
        assess_coverage(&b, &AnalysisOptions::default())
            .unwrap()
            .status,
        "needs_information"
    );
}
#[test]
fn source_language_contracts_and_unless_are_not_json_only() {
    let source = r#"oathkeeper "0.5";
module test.syntax @ "1";
type Person;
predicate allowed(Person); predicate exempt(Person);
rule demo(person: Person) @ "1" {
 jurisdiction "x"; effective_from "2020-01-01T00:00:00Z"; effective_until null;
 recorded_at "2020-01-01T00:00:00Z";
 source {citation "Synthetic"; uri "https://example.com/synthetic"; sha256 "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";};
 unless exempt($person); ensures allowed($person);
}
"#;
    // The parser understands the exception; missing legal metadata is rejected separately.
    let error = compile_text(source).unwrap_err().to_string();
    assert!(!error.contains("E_PARSE"), "{error}");
}
#[test]
fn exceptions_require_explicit_negative_evidence_and_selected_readings() {
    let mut b = scoped();
    b.query.coverage = None;
    let i = b
        .rules
        .iter()
        .position(|r| r.id == "wa.long.ability-to-pay")
        .unwrap();
    let condition = Claim {
        atom: "parking.smc72_c_excluded".into(),
        negative: false,
        arguments: b.rules[i].conclusion.arguments.clone(),
    };
    b.rules[i].exceptions.push(condition);
    assert!(verify(&b).is_err());
    let options = AnalysisOptions {
        sensitivity: false,
        ..Default::default()
    };
    assert!(analyze(&b, &options).unwrap().supporting.is_empty());
    let mut f = b.facts[0].clone();
    f.id = "synthetic.no-exception".into();
    f.claim = Claim {
        atom: "parking.smc72_c_excluded".into(),
        negative: true,
        arguments: b.query.goal.arguments.clone(),
    };
    b.facts.push(f);
    assert!(!analyze(&b, &options).unwrap().supporting.is_empty());
    let mut contrary = b.facts.last().unwrap().clone();
    contrary.id = "synthetic.exception-present".into();
    contrary.claim.negative = false;
    b.facts.push(contrary);
    let disputed = analyze(&b, &options).unwrap();
    assert_eq!(disputed.exception_checks[0].status, SupportStatus::Both);
    b.facts.pop();
    b.interpretations.push(Interpretation {
        id: "reading.a".into(),
        issue: "test.issue".into(),
        description: "Synthetic reading".into(),
        source: b.rules[i].source.clone(),
        pinpoint: "Engineering fixture".into(),
    });
    b.rules[i].interpretation = Some("reading.a".into());
    assert!(analyze(&b, &options).unwrap().supporting.is_empty());
    b.query.interpretations = vec!["reading.a".into()];
    assert!(!analyze(&b, &options).unwrap().supporting.is_empty());
    let mut alternate = b.interpretations[0].clone();
    alternate.id = "reading.b".into();
    b.interpretations.push(alternate);
    b.query.interpretations.push("reading.b".into());
    assert!(validate_context(&b).is_err());
    b.language = "oathkeeper/0.4".into();
    assert!(validate_context(&b).is_err());
}
#[test]
fn frozen_evaluator_rejects_candidate_contexts_and_recomputed_task_identity() {
    let b = scoped();
    let task = freeze_task(
        &b,
        EvaluationPolicy {
            allow_oracles: true,
            require_coverage: true,
        },
    )
    .unwrap();
    let candidate = json!({"proof":b.proof,"scope_proofs":condition_proofs(&b)}).to_string();
    assert_eq!(
        evaluate_task(&task, &task.task_sha256, &candidate)
            .unwrap()
            .reward,
        1
    );
    assert_eq!(
        evaluate_task(
            &task,
            &task.task_sha256,
            &serde_json::to_string(&b).unwrap()
        )
        .unwrap()
        .reward,
        0
    );
    assert_eq!(
        evaluate_task(
            &task,
            &task.task_sha256,
            &json!({"proof":b.proof}).to_string()
        )
        .unwrap()
        .reward,
        0
    );
    let mut changed = b.clone();
    changed.query.known_at = "2027-01-01T00:00:00Z".into();
    let replacement = freeze_task(&changed, task.policy.clone()).unwrap();
    assert!(evaluate_task(&replacement, &task.task_sha256, &candidate).is_err());
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
fn key() -> (SigningKey, TrustStore) {
    let key = SigningKey::from_bytes(&[7; 32]);
    let trust = TrustStore {
        keys: vec![TrustedKey {
            id: "test.key".into(),
            principal: "test.reviewer".into(),
            public_key: hex(&key.verifying_key().to_bytes()),
            valid_from: "2020-01-01T00:00:00Z".into(),
            valid_until: None,
            revoked_at: None,
        }],
    };
    (key, trust)
}
fn append(journal: &mut Journal, key: &SigningKey, decision: ReviewDecision) {
    let mut event = Receipt {
        id: format!("event.{}", journal.events.len()),
        sequence: journal.events.len(),
        previous_sha256: journal.events.last().map(|e| digest(e).unwrap()),
        signer: "test.key".into(),
        recorded_at: "2026-10-18T00:00:00Z".into(),
        payload: ReceiptPayload::Review {
            artifact_sha256: "a".repeat(64),
            aspect: ReviewAspect::Formalization,
            decision,
            independence_declared: true,
            rationale: "Synthetic cryptographic fixture, not a legal review".into(),
        },
        signature: String::new(),
    };
    event.signature = hex(&key.sign(&signing_bytes(&event).unwrap()).to_bytes());
    journal.events.push(event);
}
#[test]
fn signed_reviews_detect_tampering_revocation_and_stale_approval() {
    let (key, mut trust) = key();
    let mut journal = Journal {
        format: "oathkeeper.journal/1".into(),
        events: vec![],
    };
    append(&mut journal, &key, ReviewDecision::Approved);
    assert_eq!(
        review_status(&"a".repeat(64), &journal, &trust, &[])
            .unwrap()
            .current
            .len(),
        1
    );
    assert_eq!(
        review_status(&"b".repeat(64), &journal, &trust, &[])
            .unwrap()
            .current
            .len(),
        0
    );
    assert!(review_status(&"a".repeat(64), &journal, &trust, &["test.reviewer".into()]).is_err());
    append(&mut journal, &key, ReviewDecision::Withdrawn);
    assert_eq!(
        review_status(&"a".repeat(64), &journal, &trust, &[])
            .unwrap()
            .current[0]
            .decision,
        ReviewDecision::Withdrawn
    );
    let mut tampered = journal.clone();
    tampered.events[0].recorded_at = "2026-10-17T00:00:00Z".into();
    assert!(verify_journal(&tampered, &trust).is_err());
    tampered = journal.clone();
    tampered.events.remove(0);
    assert!(verify_journal(&tampered, &trust).is_err());
    trust.keys[0].revoked_at = Some("2026-10-17T00:00:00Z".into());
    assert!(verify_journal(&journal, &trust).is_err());
    assert!(verify_journal(&journal, &TrustStore { keys: vec![] }).is_err());
}
#[test]
fn evidence_receipts_bind_exact_context_and_host_pinned_head() {
    let b = historical();
    let (key, trust) = key();
    let mut journal = Journal {
        format: "oathkeeper.journal/1".into(),
        events: vec![],
    };
    append(&mut journal, &key, ReviewDecision::Approved);
    journal.events[0].payload = ReceiptPayload::Evidence {
        context_sha256: context_digest(&b).unwrap(),
        target_kind: "fact".into(),
        target: b.facts[0].id.clone(),
        payload_sha256: digest(&b.facts[0]).unwrap(),
    };
    journal.events[0].signature = hex(&key
        .sign(&signing_bytes(&journal.events[0]).unwrap())
        .to_bytes());
    let head = digest(&journal.events[0]).unwrap();
    let result = authenticate_evidence(&b, &journal, &trust, &head).unwrap();
    assert_eq!(result.authenticated.len(), 1);
    assert!(!result.missing.is_empty());
    assert!(authenticate_evidence(&b, &journal, &trust, &"a".repeat(64)).is_err());
    let mut changed = b.clone();
    changed.query.known_at = "2027-01-01T00:00:00Z".into();
    assert!(authenticate_evidence(&changed, &journal, &trust, &head)
        .unwrap()
        .authenticated
        .is_empty());
}

// A deliberately independent forward-closure model for a small propositional fragment.
// It shares no production search, substitution, eligibility or certificate helpers.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(96))]
    #[test]
    fn finite_horn_fragment_matches_independent_reference(facts in prop::array::uniform4(any::<bool>()), edges in prop::array::uniform16(any::<bool>())) {
        let source=json!({"citation":"Synthetic property test","uri":"https://example.com/test","sha256":"a".repeat(64)});
        let mut rules=vec![];
        for i in 0..4 {for j in 0..4 {if edges[i*4+j] {rules.push(json!({"id":format!("r.{i}.{j}"),"version":"1","jurisdiction":"toy","effective_from":"2020-01-01T00:00:00Z","effective_until":null,"recorded_at":"2020-01-01T00:00:00Z","source":source,"premises":[{"atom":format!("n.{i}")}],"conclusion":{"atom":format!("n.{j}")}}));}}}
        let mut input=json!({"language":"oathkeeper/0.1","package":"test.fragment","version":"1","jurisdictions":[{"id":"toy","name":"Synthetic","parents":[]}],"rules":rules,"facts":(0..4).filter(|i|facts[*i]).map(|i|json!({"id":format!("f.{i}"),"claim":{"atom":format!("n.{i}")},"evidence_sha256":"b".repeat(64),"recorded_at":"2020-01-01T00:00:00Z"})).collect::<Vec<_>>(),"attestations":[],"oracle_policy":[],"query":{"jurisdiction":"toy","as_of":"2020-01-01T00:00:00Z","known_at":"2020-01-01T00:00:00Z","goal":{"atom":"n.3"},"root":"root"},"proof":[]});
        input["language"]=json!("oathkeeper/0.5");
        input["predicates"]=json!((0..4).map(|i|json!({"id":format!("n.{i}"),"arguments":[]})).chain(std::iter::once(json!({"id":"test.scope","arguments":[]}))).collect::<Vec<_>>());
        input["authorities"]=json!([{"id":"test.authority","kind":"administrative_guidance","jurisdiction":"toy","issuing_body":"toy.office","decided_at":null,"publication":"published","source":source}]);
        let mut assessments=vec![];
        for rule in input["rules"].as_array_mut().unwrap() {
            rule["authority"]=json!({"authority":"test.authority","role":"text","pinpoint":"Synthetic test","scope":{"atom":"test.scope"}});
            rule["premises"].as_array_mut().unwrap().insert(0,json!({"atom":"test.scope"}));
            rule["time_basis"]=json!("law_time");
            assessments.push(json!({"id":format!("assessment.{}",rule["id"].as_str().unwrap()),"authority":"test.authority","rule":rule["id"],"rule_version":"1","forum":"toy.court","as_of":"2020-01-01T00:00:00Z","recorded_at":"2020-01-01T00:00:00Z","force":"persuasive","treatment":"operative","review_status":"machine_assisted_unreviewed","source":source,"rationale":"Synthetic property test"}));
        }
        input["authority_assessments"]=json!(assessments);
        input["facts"].as_array_mut().unwrap().push(json!({"id":"scope","claim":{"atom":"test.scope"},"evidence_sha256":"a".repeat(64),"recorded_at":"2020-01-01T00:00:00Z"}));
        for fact in input["facts"].as_array_mut().unwrap() {
            fact["evidence_sha256"]=json!("a".repeat(64));
            fact["basis"]=json!({"kind":"scope_assumption","source":source,"pinpoint":"Synthetic fixture","explanation":"Generated engineering input"});
        }
        for (key,value) in [("event_at","2020-01-01"),("authority_at","2020-01-01T00:00:00Z"),("forum","toy.court"),("purpose","synthetic_test"),("temporal_rationale","Synthetic same-instant model")] {input["query"][key]=json!(value);}
        let b=parse(&input.to_string(),"json").unwrap();
        let mut reachable=facts;
        for _ in 0..4 { for i in 0..4 {for j in 0..4 {if reachable[i]&&edges[i*4+j] {reachable[j]=true;}}} }
        let result=analyze(&b,&AnalysisOptions{sensitivity:false,..Default::default()}).unwrap();
        prop_assert_eq!(!result.supporting.is_empty(),reachable[3]);
        prop_assert!(result.opposing.is_empty());
    }
    #[test]
    fn arbitrary_text_does_not_panic(source in ".{0,2048}") {
        for format in ["json","oath"] {if let Ok(bundle)=parse(&source,format) {let _=validate_context(&bundle);}}
    }
    #[test]
    fn checked_arithmetic_rejects_a_one_unit_forgery(a in 0i64..1_000_000,b in 0i64..1_000_000) {
        let mut context=historical();context.query.coverage=None;
        let terms=|sum|vec![Term::Quantity{value:a,unit:"cent".into()},Term::Quantity{value:b,unit:"cent".into()},Term::Quantity{value:sum,unit:"cent".into()}];
        context.query.goal=Claim{atom:"oathkeeper.quantity.add".into(),negative:false,arguments:terms(a+b)};context.query.root="sum".into();
        context.proof=vec![Step{id:"sum".into(),claim:context.query.goal.clone(),reason:Reason::Compute{operator:Computation::QuantityAdd}}];
        prop_assert!(verify(&context).is_ok());
        context.query.goal.arguments=terms(a+b+1);context.proof[0].claim=context.query.goal.clone();
        prop_assert!(verify(&context).is_err());
    }
}
