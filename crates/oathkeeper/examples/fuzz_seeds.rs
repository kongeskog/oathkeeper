use oathkeeper::*;
use std::{collections::BTreeMap, path::PathBuf};
fn main() {
    let root = PathBuf::from(
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| "fuzz/corpus".into()),
    );
    let source = "module fuzz;query answer()->Int{scope=true;applies=true;}rule r for answer(){when=true;yield=7;}";
    let p = compile_sources(
        "fuzz",
        &BTreeMap::from([("fuzz.oath".into(), source.into())]),
    )
    .unwrap();
    let r = RunRequest {
        version: VERSION.into(),
        query: "fuzz.answer".into(),
        arguments: BTreeMap::new(),
        metadata: BTreeMap::new(),
    };
    let limits = ExecutionLimits {
        input_bytes: 256 * 1024,
        trace_nodes: 2000,
        collection_entries: 1000,
        ..ExecutionLimits::default()
    };
    let c = run(&p, &r, &limits).unwrap().certificate;
    for name in ["parser", "certificate_reader"] {
        std::fs::create_dir_all(root.join(name)).unwrap();
    }
    std::fs::write(root.join("parser/valid.oath"), source).unwrap();
    std::fs::write(
        root.join("certificate_reader/valid.json"),
        canonical_json(&c).unwrap(),
    )
    .unwrap();
}
