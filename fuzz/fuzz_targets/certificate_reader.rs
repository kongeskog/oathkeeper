#![no_main]
use libfuzzer_sys::fuzz_target;
use oathkeeper::*;
use std::{collections::BTreeMap,sync::OnceLock};
fuzz_target!(|data:&[u8]| {
    let limits=ExecutionLimits{input_bytes:256*1024,trace_nodes:2000,collection_entries:1000,..ExecutionLimits::default()};
    if let Ok(c)=read_json::<Certificate>(data,&limits){
        static PROGRAM:OnceLock<Program>=OnceLock::new();
        let p=PROGRAM.get_or_init(||compile_sources("fuzz",&BTreeMap::from([("fuzz.oath".into(),"module fuzz;query answer()->Int{scope=true;applies=true;}rule r for answer(){when=true;yield=7;}".into())])).unwrap());
        let request=RunRequest{version:VERSION.into(),query:"fuzz.answer".into(),arguments:BTreeMap::new(),metadata:BTreeMap::new()};
        let _=verify(p,&request,&c,&limits);
    }
});
