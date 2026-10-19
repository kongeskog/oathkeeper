use oathkeeper::*;
use std::collections::BTreeMap;

fn compile_text(source: &str) -> Program {
    compile_sources(
        "test",
        &BTreeMap::from([("test.oath".into(), source.into())]),
    )
    .unwrap_or_else(|e| panic!("{e:#?}"))
}
fn request(args: &[(&str, Observation)]) -> RunRequest {
    RunRequest {
        version: VERSION.into(),
        query: "test.answer".into(),
        arguments: args
            .iter()
            .map(|(n, v)| (n.to_string(), v.clone()))
            .collect(),
        metadata: BTreeMap::new(),
    }
}
fn checked(p: &Program, r: &RunRequest) -> RunBundle {
    let b = run(p, r, &ExecutionLimits::default()).unwrap();
    verify(p, r, &b.certificate, &ExecutionLimits::default()).unwrap();
    b
}
fn missing() -> Observation {
    Observation::missing("fact", "Fact", "unknown fact")
}
fn resolved(b: &RunBundle) -> &Value {
    match &b.result.outcome {
        QueryOutcome::Resolved { value, .. } => value,
        other => panic!("{other:?}"),
    }
}
const PRELUDE: &str = "module test; query answer(x: Bool) -> Int {scope=true; applies=true;}";

#[test]
fn outcome_stages_are_exclusive() {
    for (scope, applies, status) in [
        ("false", "true", "outside"),
        ("true", "false", "inapplicable"),
        ("x", "true", "scope"),
        ("true", "x", "applicability"),
    ] {
        let p = compile_text(&format!(
            "module test; query answer(x: Bool) -> Int {{scope={scope};applies={applies};}} rule r for answer(x) {{when=true;yield=9;}}"
        ));
        let b = checked(&p, &request(&[("x", missing())]));
        match (status, b.result.outcome) {
            ("outside", QueryOutcome::OutsideCoverage { .. })
            | ("inapplicable", QueryOutcome::Inapplicable { .. }) => {}
            (s, QueryOutcome::Undetermined { stage, .. }) => assert_eq!(stage, s),
            (_, other) => panic!("{other:?}"),
        }
    }
}
#[test]
fn known_absence_and_missing_option_differ() {
    let p = compile_text(
        "module test; enum Option<T>{None,Some(T)} query answer(x: Option<Int>)->Bool{scope=true;applies=true;} rule r for answer(x){when=true;yield=match x {Option.None=>true,Option.Some(v)=>false};}",
    );
    let none = Observation::known(Value::Variant {
        name: "test.Option".into(),
        type_args: vec![Ty::Int],
        variant: "None".into(),
        fields: vec![],
    });
    assert_eq!(
        resolved(&checked(&p, &request(&[("x", none)]))),
        &Value::Bool { value: true }
    );
    assert!(matches!(
        checked(&p, &request(&[("x", missing())])).result.outcome,
        QueryOutcome::Undetermined { .. }
    ));
}
#[test]
fn decisive_boolean_values_and_unresolved_branches() {
    for expression in [
        "x and false",
        "false and x",
        "not (x or true)",
        "not (true or x)",
    ] {
        let p = compile_text(&format!(
            "module test;query answer(x:Bool)->Bool{{scope=true;applies=true;}}rule r for answer(x){{when=true;yield={expression};}}"
        ));
        assert_eq!(
            resolved(&checked(&p, &request(&[("x", missing())]))),
            &Value::Bool { value: false }
        );
    }
    let p = compile_text(&format!(
        "{PRELUDE}rule r for answer(x){{when=true;yield=if x {{1}} else {{1}};}}"
    ));
    assert!(matches!(
        checked(&p, &request(&[("x", missing())])).result.outcome,
        QueryOutcome::Undetermined { .. }
    ));
}
#[test]
fn agreeing_claims_combine_evidence() {
    let p = compile_text(&format!("{PRELUDE}rule r for answer(x){{when=x;yield=1;}}"));
    let input = Observation::Disputed {
        issue: "dispute".into(),
        claims: vec![
            Claim {
                value: Value::Bool { value: true },
                evidence: vec!["witness.b".into()],
            },
            Claim {
                value: Value::Bool { value: true },
                evidence: vec!["witness.a".into()],
            },
        ],
    };
    let b = checked(&p, &request(&[("x", input)]));
    assert_eq!(resolved(&b), &Value::Int { value: "1".into() });
    assert_eq!(b.result.used_premises, vec!["witness.a", "witness.b"]);
}
#[test]
fn unrelated_missing_record_field_does_not_block_projection() {
    let p = compile_text(
        "module test;type Facts={used:Int,unused:Bool};query answer(x:Facts)->Int{scope=true;applies=true;}rule r for answer(x){when=true;yield=x.used;}",
    );
    let facts = Observation::known(Value::Record {
        name: "test.Facts".into(),
        type_args: vec![],
        fields: BTreeMap::from([("used".into(), int(7)), ("unused".into(), missing())]),
    });
    assert_eq!(
        resolved(&checked(&p, &request(&[("x", facts)]))),
        &Value::Int { value: "7".into() }
    );
}
#[test]
fn inactive_middle_rule_preserves_transitive_priority() {
    let p = compile_text(&format!(
        "{PRELUDE}
      rule high for answer(x){{when=true;yield=3;}}
      rule middle for answer(x){{when=false;yield=2;}}
      rule low for answer(x){{when=true;yield=1;}}
      priority a for answer(x){{higher=high;lower=middle;when=true;}}
      priority b for answer(x){{higher=middle;lower=low;when=true;}}"
    ));
    let b = checked(&p, &request(&[("x", boolean(false))]));
    assert_eq!(resolved(&b), &Value::Int { value: "3".into() });
}
#[test]
fn unknown_priority_or_defeated_candidate_condition_blocks() {
    for (guard, priority, stage) in [("true", "x", "priority"), ("x", "true", "rule_condition")] {
        let p = compile_text(&format!(
            "{PRELUDE}
          rule high for answer(x){{when=true;yield=3;}}
          rule low for answer(x){{when={guard};yield=1;}}
          priority p for answer(x){{higher=high;lower=low;when={priority};}}"
        ));
        assert!(
            matches!(checked(&p,&request(&[("x",missing())])).result.outcome,QueryOutcome::Undetermined{stage:s,..} if s==stage)
        );
    }
}
#[test]
fn equal_answers_merge_and_different_answers_conflict() {
    for (other, conflict) in [(1, false), (2, true)] {
        let p = compile_text(&format!(
            "{PRELUDE}rule a for answer(x){{when=true;yield=1;}}rule b for answer(x){{when=true;yield={other};}}"
        ));
        match checked(&p, &request(&[("x", boolean(false))]))
            .result
            .outcome
        {
            QueryOutcome::Resolved { support, .. } => {
                assert!(!conflict);
                assert_eq!(support, vec!["test.a", "test.b"]);
            }
            QueryOutcome::Conflicted { alternatives } => {
                assert!(conflict);
                assert_eq!(alternatives.len(), 2);
            }
            x => panic!("{x:?}"),
        }
    }
}
#[test]
fn no_rule_does_not_default_to_false() {
    let p = compile_text(PRELUDE);
    let b = checked(&p, &request(&[("x", boolean(false))]));
    assert!(
        matches!(b.result.outcome,QueryOutcome::Undetermined{issues,..} if issues.iter().any(|i|i.kind=="NoSupportingRule"))
    );
}
#[test]
fn defeated_values_are_not_evaluated() {
    let p = compile_text(&format!(
        "{PRELUDE}rule good for answer(x){{when=true;yield=1;}}rule bad for answer(x){{when=true;yield=1%0;}}priority p for answer(x){{higher=good;lower=bad;when=true;}}"
    ));
    assert_eq!(
        resolved(&checked(&p, &request(&[("x", boolean(false))]))),
        &Value::Int { value: "1".into() }
    );
}
#[test]
fn unresolved_dependency_never_coerces_to_false() {
    for applies in ["false", "true"] {
        let p = compile_text(&format!(
            "module test;query absent()->Bool{{scope=true;applies={applies};}}query answer()->Bool{{scope=true;applies=true;}}rule r for answer(){{when=true;yield=not absent();}}"
        ));
        assert!(matches!(
            checked(&p, &request(&[])).result.outcome,
            QueryOutcome::Undetermined { .. }
        ));
    }
}
#[test]
fn all_rules_in_import_closure_are_registered() {
    let sources=BTreeMap::from([
        ("root.oath".into(),"module test;use extras;query answer()->Int{scope=true;applies=true;}rule a for answer(){when=true;yield=1;}".into()),
        ("extra.oath".into(),"module extras;rule hidden for test.answer(){when=true;yield=2;}".into()),
        ("unrelated.oath".into(),"module unrelated;rule absent for test.answer(){when=true;yield=3;}".into()),
    ]);
    let p = compile_sources("test", &sources).unwrap();
    assert_eq!(p.rules.len(), 2);
    assert!(
        matches!(checked(&p,&request(&[])).result.outcome,QueryOutcome::Conflicted{alternatives} if alternatives.len()==2)
    );
}
#[test]
fn declaration_order_does_not_select_winner() {
    let a = "rule a for answer(x){when=true;yield=1;}";
    let b = "rule b for answer(x){when=true;yield=2;}";
    let r = request(&[("x", boolean(false))]);
    assert_eq!(
        checked(&compile_text(&format!("{PRELUDE}{a}{b}")), &r)
            .result
            .outcome,
        checked(&compile_text(&format!("{PRELUDE}{b}{a}")), &r)
            .result
            .outcome
    );
}
#[test]
fn cycles_and_illegal_types_are_rejected_with_spans() {
    let sources = [
        "module test;fn a()->Int{b()}fn b()->Int{a()}",
        "module test;query a()->Int{scope=true;applies=true;}rule r for a(){when=true;yield=a();}",
        "module test;query a(x:Bool)->Int{scope=true;applies=true;}rule r for a(x){when=true;yield=1;}rule s for a(x){when=true;yield=2;}priority p for a(x){higher=r;lower=s;when=x;}priority q for a(x){higher=s;lower=r;when=not x;}",
        "module test;type USD={amount:Rational};type EUR={amount:Rational};fn wrong(x:USD)->EUR{x}",
        "module test;query a()->Bool{scope=1;applies=true;}",
        "module test;enum E{A,B}fn f(x:E)->Bool{match x{E.A=>true}}",
        "module test;query a()->Int{scope=true;applies=true;}fn f()->Int{a()}",
    ];
    for source in sources {
        let errors = compile_sources(
            "test",
            &BTreeMap::from([("test.oath".into(), source.into())]),
        )
        .unwrap_err();
        assert!(!errors[0].message.is_empty());
        if errors[0].code != "Cycle" {
            assert!(errors[0].span.line > 0);
        }
    }
}
#[test]
fn execution_errors_carry_spans() {
    let p = compile_text(
        "module test;query answer()->Rational{scope=true;applies=true;}rule r for answer(){when=true;yield=1.0/0.0;}",
    );
    let error = run(&p, &request(&[]), &ExecutionLimits::default()).unwrap_err();
    assert_eq!(error.code, "Operation");
    assert!(error.span.unwrap().line > 0);
}
#[test]
fn deterministic_run_and_strict_certificate_roundtrip() {
    let p = compile_text(&format!(
        "{PRELUDE}rule r for answer(x){{when=true;yield=7;}}"
    ));
    let r = request(&[("x", boolean(false))]);
    let a = checked(&p, &r);
    let b = checked(&p, &r);
    assert_eq!(a, b);
    let bytes = canonical_json(&a.certificate).unwrap();
    let decoded: Certificate = read_json(&bytes, &ExecutionLimits::default()).unwrap();
    verify(&p, &r, &decoded, &ExecutionLimits::default()).unwrap();
}
#[test]
fn deterministic_resource_errors_are_not_outcomes() {
    let p = compile_text(
        "module test;query answer()->List<Int>{scope=true;applies=true;}rule r for answer(){when=true;yield=range(0,20);}",
    );
    for limits in [
        ExecutionLimits {
            steps: 2,
            ..ExecutionLimits::default()
        },
        ExecutionLimits {
            trace_nodes: 2,
            ..ExecutionLimits::default()
        },
        ExecutionLimits {
            collection_entries: 2,
            ..ExecutionLimits::default()
        },
        ExecutionLimits {
            integer_bits: 2,
            ..ExecutionLimits::default()
        },
        ExecutionLimits {
            call_depth: 0,
            ..ExecutionLimits::default()
        },
        ExecutionLimits {
            input_bytes: 500,
            ..ExecutionLimits::default()
        },
    ] {
        assert!(run(&p, &request(&[]), &limits).is_err());
    }
}
#[test]
fn long_binary_and_type_inputs_are_bounded() {
    let expr = vec!["1"; 2000].join("+");
    let source = format!("module test;const x={expr};");
    assert!(compile_sources("test", &BTreeMap::from([("test.oath".into(), source)])).is_err());
    let source = format!(
        "module test;type T={}Int{};",
        "List<".repeat(200),
        ">".repeat(200)
    );
    assert!(compile_sources("test", &BTreeMap::from([("test.oath".into(), source)])).is_err());
    assert!(decimal(&format!("0.{}", "9".repeat(100000))).is_err());
}
