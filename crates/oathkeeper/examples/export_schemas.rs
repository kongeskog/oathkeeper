use oathkeeper::*;
use schemars::{JsonSchema, schema_for};
use std::path::Path;

fn write<T: JsonSchema>(root: &Path, name: &str) {
    let mut schema = serde_json::to_value(schema_for!(T)).unwrap();
    schema["$id"] = serde_json::json!(format!("urn:oathkeeper:0.1.0:{name}"));
    let mut bytes = serde_json::to_vec_pretty(&schema).unwrap();
    bytes.push(b'\n');
    std::fs::write(root.join(format!("{name}.schema.json")), bytes).unwrap();
}
fn main() {
    let root = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "schemas/v0.1".into());
    let root = Path::new(&root);
    std::fs::create_dir_all(root).unwrap();
    write::<RunRequest>(root, "request");
    write::<QueryResult>(root, "result");
    write::<Certificate>(root, "certificate");
    write::<ExecutionLimits>(root, "limits");
    write::<PackageLock>(root, "lock");
    write::<Manifest>(root, "manifest");
    write::<Fixture>(root, "fixture");
}
