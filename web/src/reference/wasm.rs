//! Browser bindings run the same Rust checker and search as the CLI and Python.
use wasm_bindgen::prelude::*;

fn input(text: &str) -> Result<oathkeeper_core::Bundle, JsValue> {
    if text.len() > 4 * 1024 * 1024 {
        return Err(JsValue::from_str("[E_LIMIT] input exceeds 4 MiB"));
    }
    oathkeeper_core::parse(text, "json").map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn verify_json(text: &str) -> Result<String, JsValue> {
    let report =
        oathkeeper_core::verify(&input(text)?).map_err(|e| JsValue::from_str(&e.to_string()))?;
    serde_json::to_string(&report).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn analyze_json(text: &str, sensitivity: bool) -> Result<String, JsValue> {
    let options = oathkeeper_core::AnalysisOptions {
        sensitivity,
        ..Default::default()
    };
    let result = oathkeeper_core::analyze(&input(text)?, &options)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    serde_json::to_string(&result).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn kernel_version() -> String {
    env!("CARGO_PKG_VERSION").into()
}

#[wasm_bindgen]
pub fn assess_backdoor_roth_json(text: &str) -> Result<String, JsValue> {
    oathkeeper_core::tax::assess_json(text).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn assess_policy_json(request_json: &str) -> Result<String, JsValue> {
    oathkeeper_core::policy::policy_json(request_json)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}
