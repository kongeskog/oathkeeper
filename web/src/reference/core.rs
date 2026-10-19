//! Oathkeeper's small, deterministic conditional-proof kernel.
//! Verification checks a derivation against supplied laws, evidence, and policy;
//! it does not authenticate those inputs or establish their legal authority.
mod analysis;
mod arguments;
mod arithmetic;
pub mod compiler;
pub mod contracts;
pub mod evaluation;
mod kernel;
mod model;
mod oracles;
pub mod policy;
pub mod receipts;
mod schema;
pub mod tax;

pub use analysis::{analyze, Analysis, AnalysisOptions, CheckedArgument, SupportStatus};
pub use arguments::{compare_arguments, ArgumentComparison};
pub use compiler::{compile_file, compile_text, Compilation};
pub use kernel::{validate_context, verify};
pub use model::*;
pub use oracles::{assess_oracles, OracleAssessment};
pub use schema::validate_module_schema;

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct Error(pub String);

pub type Result<T> = std::result::Result<T, Error>;

pub fn parse(input: &str, format: &str) -> Result<Bundle> {
    if input.len() > 16 * 1024 * 1024 {
        return Err(Error("[E_LIMIT] serialized input exceeds 16 MiB".into()));
    }
    match format {
        "json" => serde_json::from_str(input).map_err(|e| Error(e.to_string())),
        "oath"
            if input.trim_start().starts_with("oathkeeper")
                || input.trim_start().starts_with("//") =>
        {
            compile_text(input)?.bundle()
        }
        "oath" | "toml" => toml::from_str(input).map_err(|e| Error(e.to_string())),
        _ => Err(Error(format!("unsupported format: {format}"))),
    }
}

pub fn verify_text(input: &str, format: &str) -> Result<Report> {
    verify(&parse(input, format)?)
}

/// Assemble a reusable JSON law module with a separately supplied JSON case.
/// The case cannot replace the module's rules, sources, or authority assessments.
pub fn assemble_case(module: &str, case: &str) -> Result<Bundle> {
    if module.len().saturating_add(case.len()) > 16 * 1024 * 1024 {
        return Err(Error(
            "[E_LIMIT] serialized module and case exceed 16 MiB".into(),
        ));
    }
    let module: LawModule = serde_json::from_str(module).map_err(|e| Error(e.to_string()))?;
    let case: CaseFile = serde_json::from_str(case).map_err(|e| Error(e.to_string()))?;
    Ok(module.with_case(case))
}
