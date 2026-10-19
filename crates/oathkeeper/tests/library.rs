use oathkeeper::*;
use proptest::prelude::*;
use std::{collections::BTreeMap, path::PathBuf, sync::OnceLock};

fn library_program() -> &'static Program {
    static PROGRAM: OnceLock<Program> = OnceLock::new();
    PROGRAM.get_or_init(|| {
        let root=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib/src");
        let mut sources=BTreeMap::new();
        for file in std::fs::read_dir(root).unwrap(){let file=file.unwrap().path();if file.extension().is_some_and(|s|s=="oath"){sources.insert(format!("std/{}",file.file_name().unwrap().to_str().unwrap()),std::fs::read_to_string(file).unwrap());}}
        sources.insert("test.oath".into(),r#"
module test;
use std.date.{date,from_ymd,to_ymd,add_months,parse_iso,days_between,OverflowPolicy,DateError};
use std.numeric.{round,RoundingMode};
use std.collections.{set,map_from_entries,Entry,get};
use std.types.{Result,Option};
query calendar()->Bool{scope=true;applies=true;}
rule checks for calendar(){when=true;yield=
 date(1970,1,1).day == 0 and date(1969,12,31).day == -1
 and days_between(date(2028,2,28),date(2028,3,1)) == 2
 and from_ymd(1900,2,29) == Result.Err(DateError.InvalidDate)
 and from_ymd(2000,2,29) == Result.Ok(date(2000,2,29))
 and add_months(date(2028,1,31),1,OverflowPolicy.Reject) == Result.Err(DateError.InvalidDate)
 and add_months(date(2028,1,31),1,OverflowPolicy.Clamp) == Result.Ok(date(2028,2,29))
 and add_months(date(2028,3,31),-1,OverflowPolicy.Clamp) == Result.Ok(date(2028,2,29));}
query parsing()->Bool{scope=true;applies=true;}
rule checks_parse for parsing(){when=true;yield=
 parse_iso("2028-02-29") == Result.Ok(date(2028,2,29))
 and parse_iso("2027-02-29") == Result.Err(DateError.InvalidDate)
 and parse_iso("2028/02/29") == Result.Err(DateError.InvalidFormat)
 and parse_iso("202a-02-29") == Result.Err(DateError.InvalidFormat);}
query collections()->Bool{scope=true;applies=true;}
rule checks_set for collections(){when=true;yield=
 set([3,1,2,1]).items == [1,2,3]
 and get(map_from_entries([Entry {key:"a",value:1},Entry{key:"a",value:2}]),"a") == Option.Some(2);}
query roundtrip(year:Int,month:Int,day:Int)->Bool{scope=true;applies=true;}
rule identity for roundtrip(year,month,day){when=true;yield={
 let civil=to_ymd(date(year,month,day));
 civil.year==year and civil.month==month and civil.day==day
};}
query rounding(value:Rational,quantum:Rational,mode:RoundingMode)->Rational{scope=true;applies=true;}
rule rounded for rounding(value,quantum,mode){when=true;yield=round(value,quantum,mode);}
"#.into());
        compile_sources("test",&sources).unwrap_or_else(|e|panic!("{e:#?}"))
    })
}
fn call(name: &str, args: BTreeMap<String, Observation>) -> Value {
    let request = RunRequest {
        version: VERSION.into(),
        query: format!("test.{name}"),
        arguments: args,
        metadata: BTreeMap::new(),
    };
    let p = library_program();
    let bundle = run(p, &request, &ExecutionLimits::default()).unwrap();
    verify(
        p,
        &request,
        &bundle.certificate,
        &ExecutionLimits::default(),
    )
    .unwrap();
    match bundle.result.outcome {
        QueryOutcome::Resolved { value, .. } => value,
        other => panic!("{other:?}"),
    }
}
fn rat(n: i64, d: i64) -> Observation {
    let l = ExecutionLimits::default();
    binary("/", &int(n), &int(d), &l).unwrap()
}
#[test]
fn calendar_boundaries() {
    assert_eq!(
        call("calendar", BTreeMap::new()),
        Value::Bool { value: true }
    );
}
#[test]
fn calendar_parsing() {
    assert_eq!(
        call("parsing", BTreeMap::new()),
        Value::Bool { value: true }
    );
}
#[test]
fn deterministic_set_and_map() {
    assert_eq!(
        call("collections", BTreeMap::new()),
        Value::Bool { value: true }
    );
}
#[test]
fn explicit_rounding_modes() {
    for (n, d, mode, expected) in [
        (25, 10, "HalfEven", 2),
        (35, 10, "HalfEven", 4),
        (-25, 10, "HalfEven", -2),
        (-25, 10, "HalfAwayFromZero", -3),
        (-21, 10, "Floor", -3),
        (-21, 10, "Ceiling", -2),
        (-21, 10, "TowardZero", -2),
    ] {
        let value = call(
            "rounding",
            BTreeMap::from([
                ("value".into(), rat(n, d)),
                ("quantum".into(), rat(1, 1)),
                (
                    "mode".into(),
                    Observation::known(Value::Variant {
                        name: "std.numeric.RoundingMode".into(),
                        type_args: vec![],
                        variant: mode.into(),
                        fields: vec![],
                    }),
                ),
            ]),
        );
        assert_eq!(
            value,
            Value::Rational {
                numerator: expected.to_string(),
                denominator: "1".into()
            }
        );
    }
}
proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]
    #[test] fn civil_dates_roundtrip(year in -2000i64..4000,month in 1i64..13,day in 1i64..29){
        prop_assert_eq!(call("roundtrip",BTreeMap::from([("year".into(),int(year)),("month".into(),int(month)),("day".into(),int(day))])),Value::Bool{value:true});
    }
    #[test] fn exact_rational_arithmetic_and_normalization(a in -100000i64..100000,b in 1i64..10000,c in -10000i64..10000,d in 1i64..10000){
        let l=ExecutionLimits::default();let x=rat(a,b);let y=rat(c,d);
        let sum=binary("+",&x,&y,&l).unwrap();
        prop_assert_eq!(binary("-",&sum,&y,&l).unwrap().value(),x.value());
        let v=x.value().unwrap();prop_assert_eq!(rational(number(&v,&l).unwrap(),&l).unwrap(),v);
    }
    #[test] fn canonical_json_is_stable_for_exact_values(n in any::<i64>()){
        let value=int(n);let bytes=canonical_json(&value).unwrap();
        let reread:Observation=read_json(&bytes,&ExecutionLimits::default()).unwrap();
        prop_assert_eq!(&value,&reread);
        prop_assert_eq!(bytes,canonical_json(&reread).unwrap());
    }
}
#[test]
fn jcs_uses_utf16_key_order_and_separates_hash_domains() {
    let value = serde_json::json!({"\u{e000}":1,"\u{1f600}":2});
    assert_eq!(
        String::from_utf8(canonical_json(&value).unwrap()).unwrap(),
        "{\"😀\":2,\"\":1}"
    );
    assert_ne!(
        digest("request", &value).unwrap(),
        digest("result", &value).unwrap()
    );
}
