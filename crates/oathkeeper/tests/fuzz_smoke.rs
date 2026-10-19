//! Bounded mutation smoke tests; coverage-guided campaigns live under /fuzz.
use oathkeeper::*;
use std::collections::BTreeMap;
fn next(seed: &mut u64) -> u64 {
    *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
    *seed
}
#[test]
fn parser_and_certificate_reader_mutation_smoke() {
    let source=b"module test;query answer(x:Bool)->Int{scope=true;applies=true;}rule r for answer(x){when=x;yield=7;}";
    let p = compile_sources(
        "test",
        &BTreeMap::from([(
            "test.oath".into(),
            String::from_utf8(source.to_vec()).unwrap(),
        )]),
    )
    .unwrap();
    let r = RunRequest {
        version: VERSION.into(),
        query: "test.answer".into(),
        arguments: BTreeMap::from([("x".into(), boolean(true))]),
        metadata: BTreeMap::new(),
    };
    let limits = ExecutionLimits {
        input_bytes: 256 * 1024,
        trace_nodes: 2000,
        collection_entries: 1000,
        ..ExecutionLimits::default()
    };
    let certificate = run(&p, &r, &limits).unwrap().certificate;
    let bytes = canonical_json(&certificate).unwrap();
    let mut seed = 0x4f415448u64;
    for _ in 0..512 {
        let mut mutated = source.to_vec();
        for _ in 0..3 {
            let i = (next(&mut seed) as usize) % mutated.len();
            mutated[i] = (next(&mut seed) % 128) as u8;
        }
        if let Ok(s) = String::from_utf8(mutated)
            && let Err(errors) = compile_sources("test", &BTreeMap::from([("test.oath".into(), s)]))
        {
            assert!(errors.iter().all(|e| e.code != "Internal"), "{errors:?}");
        }
        let mut mutated = bytes.clone();
        for _ in 0..3 {
            let i = (next(&mut seed) as usize) % mutated.len();
            mutated[i] = (next(&mut seed) % 256) as u8;
        }
        if let Ok(c) = read_json::<Certificate>(&mutated, &limits)
            && let Err(error) = verify(&p, &r, &c, &limits)
        {
            assert!(!error.0.contains("worker panic"));
        }
    }
}
