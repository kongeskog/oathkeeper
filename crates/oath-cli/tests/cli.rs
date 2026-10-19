use std::{fs, path::PathBuf, process::Command};
use tempfile::TempDir;
fn package() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/quiet-hours")
}
#[test]
fn run_and_verify_and_exit_codes() {
    let dir = TempDir::new().unwrap();
    let cert = dir.path().join("run.json");
    let p = package();
    let request = p.join("scenarios/emergency.json");
    let output = Command::new(env!("CARGO_BIN_EXE_oath"))
        .arg("run")
        .arg(&p)
        .args(["--format", "json", "--request"])
        .arg(&request)
        .arg("--certificate")
        .arg(&cert)
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output);
    let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["outcome"]["value"]["variant"], "Permitted");
    assert!(
        Command::new(env!("CARGO_BIN_EXE_oath"))
            .arg("verify")
            .arg(&p)
            .arg("--request")
            .arg(&request)
            .arg("--certificate")
            .arg(&cert)
            .output()
            .unwrap()
            .status
            .success()
    );
    let mut changed: serde_json::Value =
        serde_json::from_slice(&fs::read(&request).unwrap()).unwrap();
    changed["arguments"]["emergency"]["value"]["value"] = serde_json::json!(false);
    let changed_path = dir.path().join("changed.json");
    fs::write(&changed_path, serde_json::to_vec(&changed).unwrap()).unwrap();
    assert_eq!(
        Command::new(env!("CARGO_BIN_EXE_oath"))
            .arg("verify")
            .arg(&p)
            .arg("--request")
            .arg(&changed_path)
            .arg("--certificate")
            .arg(&cert)
            .output()
            .unwrap()
            .status
            .code(),
        Some(1)
    );
    assert_eq!(
        Command::new(env!("CARGO_BIN_EXE_oath"))
            .arg("not-a-command")
            .output()
            .unwrap()
            .status
            .code(),
        Some(2)
    );
    let output = Command::new(env!("CARGO_BIN_EXE_oath"))
        .arg("run")
        .arg(&p)
        .arg("--request")
        .arg(changed_path)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&output.stdout).contains("Prohibited"));
}
