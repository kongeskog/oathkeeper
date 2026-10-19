use oathkeeper::*;
use std::{collections::BTreeMap, fs, path::Path};
use tempfile::TempDir;
fn write(root: &Path, path: &str, content: &str) {
    let path = root.join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}
fn setup(root: &Path) {
    write(
        root,
        "oath.toml",
        "[package]\nname=\"example\"\nversion=\"0.1.0\"\nlanguage=\"=0.1.0\"\nentry=\"src/main.oath\"\n",
    );
    write(
        root,
        "src/main.oath",
        "module example;query answer()->Int{scope=true;applies=true;}rule r for answer(){when=true;yield=7;}",
    );
}
#[test]
fn locks_are_explicit_and_portable() {
    let a = TempDir::new().unwrap();
    let b = TempDir::new().unwrap();
    setup(a.path());
    setup(b.path());
    assert!(load_package(a.path()).is_err());
    let x = lock_package(a.path()).unwrap();
    let y = lock_package(b.path()).unwrap();
    assert_eq!(x, y);
    let pa = compile(&load_package(a.path()).unwrap()).unwrap();
    let pb = compile(&load_package(b.path()).unwrap()).unwrap();
    assert_eq!(pa, pb);
    write(a.path(), "src/main.oath", "module example;const changed=1;");
    assert!(load_package(a.path()).is_err());
}
#[test]
fn dependency_paths_are_hints_and_snapshots_are_transitive() {
    let temp = TempDir::new().unwrap();
    setup(&temp.path().join("lib"));
    for (root, dep) in [("consumer_a", "../lib"), ("consumer_b", "../copy")] {
        write(
            &temp.path().join(root),
            "oath.toml",
            &format!(
                "[package]\nname=\"consumer\"\nversion=\"0.1.0\"\nlanguage=\"=0.1.0\"\nentry=\"src/main.oath\"\n[dependencies.lib]\npath=\"{dep}\"\nversion=\"=0.1.0\"\n"
            ),
        );
        write(
            &temp.path().join(root),
            "src/main.oath",
            "module consumer;use example;",
        );
    }
    setup(&temp.path().join("copy"));
    assert_eq!(
        lock_package(&temp.path().join("consumer_a")).unwrap(),
        lock_package(&temp.path().join("consumer_b")).unwrap()
    );
    write(
        &temp.path().join("lib"),
        "src/main.oath",
        "module example;const new=2;",
    );
    assert!(load_package(&temp.path().join("consumer_a")).is_err());
}
#[test]
fn source_integrity_is_separate_from_computation() {
    let temp = TempDir::new().unwrap();
    setup(temp.path());
    let hash = sha256(b"original source\n");
    write(
        temp.path(),
        "src/provenance.oath",
        "module std.provenance;type SourceRef={stable_id:Text,citation:Text,edition_or_version:Text,locations:List<Text>,expected_sha256:Text,publication_metadata:Text,retrieval_metadata:Text};",
    );
    write(
        temp.path(),
        "src/main.oath",
        &format!(
            r#"module example;use std.provenance.SourceRef;
const source=SourceRef{{stable_id:"source.1",citation:"Fiction",edition_or_version:"1",locations:["fixture:policy.txt"],expected_sha256:"{hash}",publication_metadata:"Private",retrieval_metadata:"Fixture"}};
query answer()->Int{{scope=true;applies=true;}}rule r for answer(){{when=true;yield=7;meta=source;}}"#
        ),
    );
    lock_package(temp.path()).unwrap();
    let package = load_package(temp.path()).unwrap();
    let p = compile(&package).unwrap();
    let request = RunRequest {
        version: VERSION.into(),
        query: "example.answer".into(),
        arguments: BTreeMap::new(),
        metadata: BTreeMap::new(),
    };
    let first = run(&p, &request, &ExecutionLimits::default()).unwrap();
    let cache = temp.path().join("cache");
    assert!(matches!(
        &audit_sources(&package, &p, &cache, false).unwrap()[0],
        SourceAudit::Unavailable { .. }
    ));
    write(temp.path(), "policy.txt", "original source\n");
    assert!(matches!(
        &audit_sources(&package, &p, &cache, false).unwrap()[0],
        SourceAudit::Matched { .. }
    ));
    write(temp.path(), "policy.txt", "changed source\n");
    assert!(matches!(
        &audit_sources(&package, &p, &cache, false).unwrap()[0],
        SourceAudit::Mismatched { .. }
    ));
    assert_eq!(
        first,
        run(&p, &request, &ExecutionLimits::default()).unwrap()
    );
    verify(
        &p,
        &request,
        &first.certificate,
        &ExecutionLimits::default(),
    )
    .unwrap();
}
#[test]
fn utf8_lf_checkout_normalization_is_portable() {
    let a = TempDir::new().unwrap();
    let b = TempDir::new().unwrap();
    setup(a.path());
    setup(b.path());
    let src = "module example;\nconst text=\"café\";\n";
    write(a.path(), "src/main.oath", src);
    write(b.path(), "src/main.oath", &src.replace('\n', "\r\n"));
    assert_eq!(
        lock_package(a.path()).unwrap(),
        lock_package(b.path()).unwrap()
    );
}
