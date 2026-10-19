use oathkeeper::*;
use std::collections::BTreeMap;
fn setup() -> (Program, RunRequest, Certificate) {
    let source = "module test;query answer(x:Bool)->Rational{scope=true;applies=true;}rule a for answer(x){when=true;yield=2.0*3.0;}rule b for answer(x){when=x;yield=9.0;}priority p for answer(x){higher=b;lower=a;when=true;}";
    let p = compile_sources(
        "test",
        &BTreeMap::from([("test.oath".into(), source.into())]),
    )
    .unwrap();
    let r = RunRequest {
        version: VERSION.into(),
        query: "test.answer".into(),
        arguments: BTreeMap::from([("x".into(), boolean(false))]),
        metadata: BTreeMap::new(),
    };
    let c = run(&p, &r, &ExecutionLimits::default())
        .unwrap()
        .certificate;
    (p, r, c)
}
#[test]
fn adversarial_certificates_fail_independent_checking() {
    let (p, r, c) = setup();
    let mut variants = Vec::new();
    let mut x = c.clone();
    x.result.outcome = QueryOutcome::Resolved {
        value: Value::Rational {
            numerator: "99".into(),
            denominator: "1".into(),
        },
        support: vec!["test.a".into()],
    };
    x.result_digest = digest("result", &x.result).unwrap();
    variants.push(("forged outcome", x));
    let mut x = c.clone();
    if let TraceNode::Query { trace } = &mut x.nodes[c.root as usize] {
        trace.guards.remove("test.b");
    }
    variants.push(("omitted candidate", x));
    let mut x = c.clone();
    if let TraceNode::Query { trace } = &mut x.nodes[c.root as usize] {
        trace.priorities.clear();
    }
    variants.push(("omitted priority", x));
    let mut x = c.clone();
    for n in &mut x.nodes {
        if let TraceNode::Expression { expr, value, .. } = n
            && matches!(p.expressions[*expr as usize].kind, ExprKind::Binary { .. })
        {
            *value = Observation::known(Value::Rational {
                numerator: "7".into(),
                denominator: "1".into(),
            });
            break;
        }
    }
    variants.push(("forged arithmetic", x));
    let mut x = c.clone();
    for n in &mut x.nodes {
        if let TraceNode::Expression { env, .. } = n
            && let Some(Observation::Known { evidence, .. }) = env.values.get_mut("x")
        {
            evidence.push("invented".into());
            break;
        }
    }
    variants.push(("invented premise", x));
    let mut x = c.clone();
    x.program_digest = "0".repeat(64);
    variants.push(("wrong program", x));
    let mut x = c.clone();
    x.request_digest = "0".repeat(64);
    variants.push(("wrong request", x));
    let mut x = c.clone();
    for (i, n) in x.nodes.iter_mut().enumerate() {
        if let TraceNode::Expression { children, .. } = n
            && !children.is_empty()
        {
            children[0] = i as u32;
            break;
        }
    }
    variants.push(("cyclic trace", x));
    let mut x = c.clone();
    if let TraceNode::Query { trace } = &mut x.nodes[c.root as usize] {
        trace.scope = u32::MAX;
    }
    variants.push(("invalid reference", x));
    let mut x = c.clone();
    x.nodes.push(x.nodes[0].clone());
    variants.push(("unreferenced node", x));
    for (name, x) in variants {
        assert!(
            verify(&p, &r, &x, &ExecutionLimits::default()).is_err(),
            "{name}"
        );
    }
    let mut changed = r.clone();
    changed.arguments.insert("x".into(), boolean(true));
    assert!(verify(&p, &changed, &c, &ExecutionLimits::default()).is_err());
}
#[test]
fn omitted_unknown_exception_is_not_a_proof_of_default() {
    let (p, mut r, _) = setup();
    r.arguments.insert(
        "x".into(),
        Observation::missing("exception", "Fact", "unknown"),
    );
    let mut c = run(&p, &r, &ExecutionLimits::default())
        .unwrap()
        .certificate;
    if let TraceNode::Query { trace } = &mut c.nodes[c.root as usize] {
        trace.guards.remove("test.b");
    }
    assert!(verify(&p, &r, &c, &ExecutionLimits::default()).is_err());
}
#[test]
fn forged_budget_cannot_claim_execution_in_fewer_steps() {
    let (p, r, mut c) = setup();
    let limits = ExecutionLimits {
        steps: 1,
        ..ExecutionLimits::default()
    };
    c.limits = limits.clone();
    assert!(verify(&p, &r, &c, &limits).is_err());
}
#[test]
fn malformed_ir_is_rejected_without_panicking() {
    let (mut p, r, c) = setup();
    p.rules.get_mut("test.a").unwrap().query = "unknown".into();
    assert!(run(&p, &r, &ExecutionLimits::default()).is_err());
    assert!(verify(&p, &r, &c, &ExecutionLimits::default()).is_err());
    let (mut p, r, _) = setup();
    p.expressions[0].kind = ExprKind::Unary {
        op: "not".into(),
        value: 0,
    };
    assert!(run(&p, &r, &ExecutionLimits::default()).is_err());
}
#[test]
fn strict_readers_reject_duplicates_unknown_fields_and_excesses() {
    let limits = ExecutionLimits::default();
    for json in [
        r#"{"version":"0.1.0","version":"0.1.0"}"#,
        r#"{"version":"0.1.0","query":"test.answer","arguments":{},"extra":1}"#,
        r#"{"value":1.5}"#,
    ] {
        assert!(read_json::<RunRequest>(json.as_bytes(), &limits).is_err());
    }
    let deep = format!("{}0{}", "[".repeat(200), "]".repeat(200));
    assert!(read_json::<serde_json::Value>(deep.as_bytes(), &limits).is_err());
    let tiny = ExecutionLimits {
        input_bytes: 2,
        ..limits.clone()
    };
    assert!(read_json::<serde_json::Value>(b"[1,2,3]", &tiny).is_err());
    for bad in ["01", "-0", "+1", "1e2"] {
        assert!(integer(bad, &limits).is_err());
    }
    for (n, d) in [("2", "4"), ("1", "0"), ("1", "-2")] {
        assert!(
            number(
                &Value::Rational {
                    numerator: n.into(),
                    denominator: d.into()
                },
                &limits
            )
            .is_err()
        );
    }
}
