#![no_main]
use libfuzzer_sys::fuzz_target;
fuzz_target!(|data: &[u8]| {
    if data.len() > 16384 { return; }
    if let Ok(text) = std::str::from_utf8(data) {
        for format in ["json", "oath"] {
            if let Ok(bundle) = oathkeeper_core::parse(text, format) {
                let _ = oathkeeper_core::validate_context(&bundle);
                let _ = oathkeeper_core::verify(&bundle);
            }
        }
        let _ = oathkeeper_core::policy::policy_json(text);
    }
});
