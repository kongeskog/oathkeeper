#![no_main]
use libfuzzer_sys::fuzz_target;
use std::collections::BTreeMap;
fuzz_target!(|data:&[u8]| {
    if data.len()>64*1024 {return}
    if let Ok(source)=std::str::from_utf8(data){
        let _=oathkeeper::compile_sources("fuzz",&BTreeMap::from([("fuzz.oath".into(),source.to_string())]));
    }
});
