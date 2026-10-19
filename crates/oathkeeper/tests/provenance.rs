use oathkeeper::*;
use std::collections::BTreeMap;
#[test]
fn source_validation_does_not_force_unrelated_failing_constants() {
    let p=compile_sources("test",&BTreeMap::from([("test.oath".into(),"module test;const unused=1%0;query answer()->Int{scope=true;applies=true;}rule r for answer(){when=true;yield=7;}".into())])).unwrap();
    assert!(source_references(&p).unwrap().is_empty());
    let r = RunRequest {
        version: VERSION.into(),
        query: "test.answer".into(),
        arguments: BTreeMap::new(),
        metadata: BTreeMap::new(),
    };
    let b = run(&p, &r, &ExecutionLimits::default()).unwrap();
    verify(&p, &r, &b.certificate, &ExecutionLimits::default()).unwrap();
}
#[test]
fn recursive_transparent_aliases_are_rejected() {
    for source in [
        "module test;type A=List<A>;",
        "module test;type A=List<B>;type B=A;",
    ] {
        let errors = compile_sources(
            "test",
            &BTreeMap::from([("test.oath".into(), source.into())]),
        )
        .unwrap_err();
        assert!(errors.iter().any(|d| d.code == "Cycle"));
    }
}
#[test]
fn explicit_generic_variant_construction() {
    let p = compile_sources(
        "test",
        &BTreeMap::from([(
            "test.oath".into(),
            "module test;enum Option<T>{None,Some(T)}const a=Option.None::<Int>();".into(),
        )]),
    )
    .unwrap();
    assert!(
        matches!(&p.constants["test.a"].ty,Ty::Named{name,args} if name=="test.Option"&&args==&vec![Ty::Int])
    );
}

#[test]
fn transparent_aliases_work_in_numeric_and_generic_positions() {
    let source = "module test;type Count=Int;type Items=List<Count>;type Box<T>={value:T};fn inc(x:Count)->Count{x+1}query answer()->Items{scope=true;applies=true;}rule r for answer(){when=true;yield=[inc(Box::<Count>{value:1}.value),2];}";
    let p = compile_sources(
        "test",
        &BTreeMap::from([("test.oath".into(), source.into())]),
    )
    .unwrap();
    let r = RunRequest {
        version: VERSION.into(),
        query: "test.answer".into(),
        arguments: BTreeMap::new(),
        metadata: BTreeMap::new(),
    };
    let b = run(&p, &r, &ExecutionLimits::default()).unwrap();
    verify(&p, &r, &b.certificate, &ExecutionLimits::default()).unwrap();
    assert!(
        matches!(b.result.outcome,QueryOutcome::Resolved{value:Value::List{items},..} if items==vec![int(2),int(2)])
    );
}
