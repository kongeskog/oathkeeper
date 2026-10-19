use clap::{Parser, Subcommand, ValueEnum};
use oathkeeper::*;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Parser)]
#[command(
    name = "oath",
    version,
    about = "Run and independently check versioned legal programs"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}
#[derive(Clone, Copy, ValueEnum)]
enum Format {
    Text,
    Json,
}
#[derive(Subcommand)]
enum Command {
    Lock {
        package: PathBuf,
    },
    Check {
        package: PathBuf,
    },
    Run {
        package: PathBuf,
        #[arg(long)]
        request: PathBuf,
        #[arg(long, value_enum, default_value = "text")]
        format: Format,
        #[arg(long)]
        certificate: Option<PathBuf>,
        #[arg(long)]
        limits: Option<PathBuf>,
    },
    Verify {
        package: PathBuf,
        #[arg(long)]
        request: PathBuf,
        #[arg(long)]
        certificate: PathBuf,
        #[arg(long)]
        limits: Option<PathBuf>,
        #[arg(long, value_enum, default_value = "text")]
        format: Format,
    },
    Test {
        package: PathBuf,
        #[arg(long)]
        json: bool,
    },
    Coverage {
        package: PathBuf,
        #[arg(long)]
        json: bool,
    },
    Sources {
        #[command(subcommand)]
        command: Sources,
    },
}
#[derive(Subcommand)]
enum Sources {
    Check {
        package: PathBuf,
        #[arg(long)]
        fetch: bool,
        #[arg(long)]
        cache: Option<PathBuf>,
    },
}
fn read<T: serde::de::DeserializeOwned>(
    path: &Path,
    limits: &ExecutionLimits,
) -> Result<T, String> {
    if fs::metadata(path).map_err(|e| e.to_string())?.len() > limits.input_bytes as u64 {
        return Err("input file exceeds byte limit".into());
    }
    read_json(
        &fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?,
        limits,
    )
}
fn json<T: serde::Serialize>(value: &T) -> Result<(), String> {
    println!(
        "{}",
        serde_json::to_string_pretty(value).map_err(|e| e.to_string())?
    );
    Ok(())
}
fn load(path: &Path) -> Result<(ResolvedPackage, CompiledProgram), String> {
    let p = load_package(path)?;
    let program = compile(&p).map_err(|ds| {
        ds.iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n")
    })?;
    source_references(&program)?;
    Ok((p, program))
}
fn profile(path: Option<PathBuf>) -> Result<ExecutionLimits, String> {
    path.map(|p| read(&p, &ExecutionLimits::default()))
        .transpose()
        .map(|p| p.unwrap_or_default())
}
fn result_text(result: &QueryResult) {
    println!("{}: {}", result.query, outcome_status(&result.outcome));
    match &result.outcome {
        QueryOutcome::Resolved { value, support } => {
            println!("Value: {}", serde_json::to_string(value).unwrap());
            println!("Supporting rules: {}", support.join(", "));
        }
        QueryOutcome::Undetermined { stage, issues } => {
            println!("Unresolved stage: {stage}");
            for i in issues {
                println!("  {} [{}]: {}", i.id, i.kind, i.message)
            }
        }
        QueryOutcome::Inapplicable { reasons } | QueryOutcome::OutsideCoverage { reasons } => {
            for r in reasons {
                println!("  {r}")
            }
        }
        QueryOutcome::Conflicted { alternatives } => {
            for a in alternatives {
                println!(
                    "  {} supported by {}",
                    serde_json::to_string(&a.value).unwrap(),
                    a.support.join(", ")
                )
            }
        }
    }
    if !result.used_premises.is_empty() {
        println!("Referenced premises: {}", result.used_premises.join(", "))
    }
    if !result.metadata.is_empty() {
        println!("Encoding records: {}", result.metadata.join(", "))
    }
}
fn execute(command: Command) -> Result<u8, String> {
    match command {
        Command::Lock { package } => {
            let lock = lock_package(&package)?;
            println!("Locked {} at {}", lock.root_package, lock.root_digest);
        }
        Command::Check { package } => {
            let (_, p) = load(&package)?;
            println!(
                "Checked {} modules, {} queries, {} rules; program {}",
                p.modules.len(),
                p.queries.len(),
                p.rules.len(),
                p.digest()?
            );
        }
        Command::Run {
            package,
            request,
            format,
            certificate,
            limits,
        } => {
            let (_, p) = load(&package)?;
            let limits = profile(limits)?;
            let request = read(&request, &limits)?;
            let bundle = run(&p, &request, &limits).map_err(|e| e.to_string())?;
            if let Some(file) = certificate {
                fs::write(
                    file,
                    serde_json::to_vec_pretty(&bundle.certificate).map_err(|e| e.to_string())?,
                )
                .map_err(|e| e.to_string())?
            }
            match format {
                Format::Json => json(&bundle.result)?,
                Format::Text => result_text(&bundle.result),
            }
        }
        Command::Verify {
            package,
            request,
            certificate,
            limits,
            format,
        } => {
            let (_, p) = load(&package)?;
            let limits = profile(limits)?;
            let request = read(&request, &limits)?;
            let certificate = read(&certificate, &limits)?;
            let checked = verify(&p, &request, &certificate, &limits).map_err(|e| e.to_string())?;
            match format {
                Format::Json => json(&checked)?,
                Format::Text => {
                    println!("Verified computation: {}", checked.certificate_digest);
                    result_text(&checked.result)
                }
            }
        }
        Command::Test {
            package,
            json: as_json,
        } => {
            let (p, program) = load(&package)?;
            let reports = test_package(&p, &program)?;
            let passed = reports.iter().all(|r| r.passed);
            if as_json {
                json(&reports)?
            } else {
                for r in &reports {
                    println!(
                        "{} {}: {}",
                        if r.passed { "PASS" } else { "FAIL" },
                        r.name,
                        r.message
                    )
                }
                println!(
                    "{}/{} passed",
                    reports.iter().filter(|r| r.passed).count(),
                    reports.len()
                )
            }
            if !passed {
                return Ok(1);
            }
        }
        Command::Coverage {
            package,
            json: as_json,
        } => {
            let (_, p) = load(&package)?;
            let records = coverage(&p)?;
            if !as_json {
                println!(
                    "{} declared coverage records; {} included modules. Declarations describe encoded scope and are not a completeness proof.",
                    records.len(),
                    p.modules.len()
                )
            }
            json(&records)?;
        }
        Command::Sources {
            command:
                Sources::Check {
                    package,
                    fetch,
                    cache,
                },
        } => {
            let (p, program) = load(&package)?;
            let cache = cache.unwrap_or_else(|| std::env::temp_dir().join("oathkeeper-sources"));
            let reports = audit_sources(&p, &program, &cache, fetch)?;
            let passed = reports
                .iter()
                .all(|r| matches!(r, SourceAudit::Matched { .. }));
            json(&reports)?;
            if !passed {
                return Ok(1);
            }
        }
    }
    Ok(0)
}
fn main() -> std::process::ExitCode {
    match execute(Cli::parse().command) {
        Ok(code) => std::process::ExitCode::from(code),
        Err(error) => {
            eprintln!("{error}");
            std::process::ExitCode::from(1)
        }
    }
}
