use sha2::{Digest, Sha256};
use std::{env, fs, path::PathBuf};

fn main() {
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("../..");
    let lock_path = root.join("lawlib.lock.json");
    let relative = "us/tax/backdoor-roth-2025.oath";
    let source_path = root.join(".deps/lawlib").join(relative);
    println!("cargo:rerun-if-changed={}", lock_path.display());
    println!("cargo:rerun-if-changed={}", source_path.display());

    // Native, Python and WASM all embed the same immutable legal input. Builds
    // never fetch it implicitly or silently accept changed dependency bytes.
    let lock: serde_json::Value = serde_json::from_slice(
        &fs::read(&lock_path).expect("lawlib.lock.json is required to build the tax module"),
    )
    .expect("invalid lawlib lock JSON");
    assert_eq!(lock["format"], "oathkeeper.lawlib-lock/1");
    let expected = lock["files"][relative]
        .as_str()
        .expect("tax module is not pinned by lawlib.lock.json");
    let source = fs::read(&source_path).expect(
        "lawlib dependency is missing; run `python3 tools/lawlib_package.py sync` before building",
    );
    assert_eq!(
        format!("{:x}", Sha256::digest(&source)),
        expected,
        "tax module differs from lawlib.lock.json; restore the exact pinned dependency"
    );
    let output = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("backdoor-roth-2025.oath");
    fs::write(output, source).expect("failed to stage the verified tax source");
}
