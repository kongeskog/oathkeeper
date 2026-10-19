use oathkeeper::*;
use std::collections::BTreeMap;

fn program(source: &str) -> Program {
    compile_sources(
        "test",
        &BTreeMap::from([("test.oath".into(), source.into())]),
    )
    .unwrap_or_else(|errors| {
        panic!(
            "{}",
            errors
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n")
        )
    })
}
fn execute(p: &Program, args: BTreeMap<String, Observation>) -> RunBundle {
    let request = RunRequest {
        version: VERSION.into(),
        query: "test.answer".into(),
        arguments: args,
        metadata: BTreeMap::new(),
    };
    let result = run(p, &request, &ExecutionLimits::default()).unwrap();
    verify(
        p,
        &request,
        &result.certificate,
        &ExecutionLimits::default(),
    )
    .unwrap();
    result
}
const EXCEPTION: &str = r#"module test;
query answer(emergency: Bool) -> Int { scope = true; applies = true; }
rule ordinary for answer(emergency) { when = true; yield = 0; }
rule exception for answer(emergency) { when = emergency; yield = 1; }
priority exception_first for answer(emergency) { higher = exception; lower = ordinary; when = true; }
"#;
#[test]
fn exceptions_and_unknowns() {
    let p = program(EXCEPTION);
    for (b, n) in [(false, "0"), (true, "1")] {
        let b = execute(&p, BTreeMap::from([("emergency".into(), boolean(b))]));
        assert!(
            matches!(b.result.outcome,QueryOutcome::Resolved{value:Value::Int{value},..} if value==n)
        );
    }
    let b = execute(
        &p,
        BTreeMap::from([(
            "emergency".into(),
            Observation::missing("emergency", "Fact", "supply a finding"),
        )]),
    );
    assert!(
        matches!(b.result.outcome,QueryOutcome::Undetermined{stage,..} if stage=="rule_condition")
    );
}
#[test]
fn exact_calculation() {
    let p = program(
        "module test; query answer() -> Rational { scope = true; applies = true; } rule calculation for answer() { when = true; yield = 7000.0 - 7000.0 * (7000.0 / 100000.0); }",
    );
    let b = execute(&p, BTreeMap::new());
    assert!(
        matches!(b.result.outcome,QueryOutcome::Resolved{value:Value::Rational{numerator,denominator},..} if numerator=="6510"&&denominator=="1")
    );
}
#[test]
fn generics_records_match_and_collections() {
    let p = program(
        r#"module test;
enum Option<T> { None, Some(T) }
type Box<T> = { value: T };
fn double(x: Int) -> Int { x * 2 }
fn sum(acc: Int, x: Int) -> Int { acc + x }
fn unwrap<T>(x: Option<T>, fallback: T) -> T { match x { Option.Some(v) => v, Option.None => fallback } }
query answer() -> Int { scope = true; applies = true; }
rule calculation for answer() { when = true; yield = unwrap(Option.Some(Box { value: fold(map([1,2,3], double), 0, sum) }), Box { value: 0 }).value; }
"#,
    );
    let b = execute(&p, BTreeMap::new());
    assert!(
        matches!(b.result.outcome,QueryOutcome::Resolved{value:Value::Int{value},..} if value=="12")
    );
}
#[test]
fn rejects_cycles() {
    let errors = compile_sources(
        "test",
        &BTreeMap::from([(
            "test.oath".into(),
            "module test; fn looped(x: Int) -> Int { looped(x) }".into(),
        )]),
    )
    .unwrap_err();
    assert!(errors.iter().any(|e| e.code == "Cycle"));
}
#[test]
fn tampering_is_rejected() {
    let p = program(EXCEPTION);
    let r = RunRequest {
        version: VERSION.into(),
        query: "test.answer".into(),
        arguments: BTreeMap::from([("emergency".into(), boolean(true))]),
        metadata: BTreeMap::new(),
    };
    let b = run(&p, &r, &ExecutionLimits::default()).unwrap();
    let mut c = b.certificate.clone();
    if let TraceNode::Query { trace } = &mut c.nodes[c.root as usize] {
        trace.guards.remove("test.ordinary");
    }
    assert!(verify(&p, &r, &c, &ExecutionLimits::default()).is_err());
    let mut c = b.certificate;
    for n in &mut c.nodes {
        if let TraceNode::Expression { value, .. } = n {
            *value = boolean(false);
            break;
        }
    }
    assert!(verify(&p, &r, &c, &ExecutionLimits::default()).is_err());
}
