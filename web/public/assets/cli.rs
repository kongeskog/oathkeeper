use oathkeeper_core::{assemble_case, compile_file, parse, verify, Bundle};
use std::{env, fs, path::Path, process::ExitCode};

fn load(path: &str) -> Result<Bundle, Box<dyn std::error::Error>> {
    let input = fs::read_to_string(path)?;
    if is_source(&input) {
        Ok(compile_file(path)?.bundle()?)
    } else {
        Ok(parse(
            &input,
            if path.ends_with(".json") {
                "json"
            } else {
                "toml"
            },
        )?)
    }
}
fn is_source(input: &str) -> bool {
    input
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && !line.starts_with("//"))
        .is_some_and(|line| line.starts_with("oathkeeper"))
}
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() == 2 && args[0] == "tax-backdoor-roth" {
        println!(
            "{}",
            oathkeeper_core::tax::assess_json(&fs::read_to_string(&args[1])?)?
        );
        return Ok(());
    }
    if args.len() == 2 && args[0] == "compile" {
        println!(
            "{}",
            serde_json::to_string_pretty(&compile_file(Path::new(&args[1]))?)?
        );
        return Ok(());
    }
    if args.len() == 2 && args[0] == "analyze" {
        let result = oathkeeper_core::analyze(
            &load(&args[1])?,
            &oathkeeper_core::AnalysisOptions::default(),
        )?;
        println!("{}", serde_json::to_string_pretty(&result)?);
        return Ok(());
    }
    let bundle = if args.len() == 3 && matches!(args[0].as_str(), "verify-case" | "normalize-case")
    {
        assemble_case(
            &fs::read_to_string(&args[1])?,
            &fs::read_to_string(&args[2])?,
        )?
    } else if args.len() == 2 && matches!(args[0].as_str(), "verify" | "normalize") {
        if args[0] == "verify" && is_source(&fs::read_to_string(&args[1])?) {
            println!(
                "{}",
                serde_json::to_string_pretty(&compile_file(&args[1])?.verify()?)?
            );
            return Ok(());
        }
        load(&args[1])?
    } else if args.len() == 3 && args[0] == "compare-arguments" {
        println!(
            "{}",
            serde_json::to_string_pretty(&oathkeeper_core::compare_arguments(
                &load(&args[1])?,
                &load(&args[2])?
            )?)?
        );
        return Ok(());
    } else {
        return Err("usage: oathkeeper tax-backdoor-roth <scenario.json>\n       oathkeeper compile <module.oath|case.oath>\n       oathkeeper <analyze|verify|normalize> <case.oath|bundle.json|legacy.toml>\n       oathkeeper <verify-case|normalize-case> <module.json> <case.json>\n       oathkeeper compare-arguments <argument.oath|json> <counterargument.oath|json>".into());
    };
    let report = verify(&bundle)?;
    let output = if args[0].starts_with("verify") {
        serde_json::to_string_pretty(&report)?
    } else {
        serde_json::to_string_pretty(&bundle)?
    };
    println!("{output}");
    Ok(())
}
fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("oathkeeper: {error}");
            ExitCode::FAILURE
        }
    }
}
