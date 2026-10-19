//! Host-owned proof-construction tasks. Candidate JSON cannot replace any premises or policy.
use crate::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub fn digest<T: Serialize>(value: &T) -> Result<String> {
    // Value uses sorted object keys (serde_json without preserve_order), recursively.
    let value = serde_json::to_value(value).map_err(|e| Error(e.to_string()))?;
    let bytes = serde_json::to_vec(&value).map_err(|e| Error(e.to_string()))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}
pub fn context_digest(bundle: &Bundle) -> Result<String> {
    let mut context = bundle.clone();
    context.proof.clear();
    context.query.root.clear();
    digest(&context)
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvaluationPolicy {
    pub allow_oracles: bool,
    pub require_coverage: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrozenTask {
    pub format: String,
    pub context: Bundle,
    pub policy: EvaluationPolicy,
    pub task_sha256: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProofSubmission {
    pub proof: Vec<Step>,
    #[serde(default)]
    pub scope_proofs: Vec<contracts::ConditionProof>,
}
#[derive(Debug, Serialize)]
pub struct Evaluation {
    pub reward: u8,
    pub task_sha256: String,
    pub report: Option<Report>,
    pub scoped_report: Option<contracts::ScopedReport>,
    pub error: Option<String>,
}
fn identity(task: &FrozenTask) -> Result<String> {
    digest(&("oathkeeper.frozen-task/1", &task.context, &task.policy))
}
pub fn freeze_task(bundle: &Bundle, policy: EvaluationPolicy) -> Result<FrozenTask> {
    validate_context(bundle)?;
    if !bundle.proof.is_empty() {
        verify(bundle)?;
    }
    if policy.require_coverage && bundle.query.coverage.is_none() {
        return Err(Error(
            "host task requires a selected coverage contract".into(),
        ));
    }
    let mut context = bundle.clone();
    context.proof.clear();
    let mut task = FrozenTask {
        format: "oathkeeper.frozen-task/1".into(),
        context,
        policy,
        task_sha256: String::new(),
    };
    task.task_sha256 = identity(&task)?;
    Ok(task)
}
/// `expected` must come from the evaluator, not the candidate or its request body.
pub fn evaluate_task(task: &FrozenTask, expected: &str, candidate: &str) -> Result<Evaluation> {
    if task.format != "oathkeeper.frozen-task/1"
        || !task.context.proof.is_empty()
        || task.task_sha256 != expected
        || identity(task)? != expected
    {
        return Err(Error(
            "[E_TASK] frozen task does not match the host's pinned identity".into(),
        ));
    }
    validate_context(&task.context)?;
    if task.policy.require_coverage && task.context.query.coverage.is_none() {
        return Err(Error(
            "[E_TASK] host omitted a required coverage contract".into(),
        ));
    }
    let mut result = Evaluation {
        reward: 0,
        task_sha256: expected.into(),
        report: None,
        scoped_report: None,
        error: None,
    };
    let checked = (|| -> Result<()> {
        if candidate.len() > 4 * 1024 * 1024 {
            return Err(Error("[E_LIMIT] candidate exceeds 4 MiB".into()));
        }
        let submission: ProofSubmission = if candidate.trim_start().starts_with('[') {
            ProofSubmission {
                proof: serde_json::from_str(candidate).map_err(|e| Error(e.to_string()))?,
                scope_proofs: Vec::new(),
            }
        } else {
            serde_json::from_str(candidate).map_err(|e| Error(e.to_string()))?
        };
        let mut bundle = task.context.clone();
        bundle.proof = submission.proof;
        let report = verify(&bundle)?;
        if task.policy.require_coverage {
            let scoped = contracts::verify_scoped(&bundle, &submission.scope_proofs)?;
            if !task.policy.allow_oracles && scoped.conditions.iter().any(|r| r.oracle_count > 0) {
                return Err(Error(
                    "host policy excludes oracle-dependent scope proofs".into(),
                ));
            }
            result.scoped_report = Some(scoped);
        } else if !submission.scope_proofs.is_empty() {
            return Err(Error(
                "unexpected scope proofs for an unscoped host task".into(),
            ));
        }
        if !task.policy.allow_oracles && report.oracle_count > 0 {
            return Err(Error("host policy excludes oracle premises".into()));
        }
        result.report = Some(report);
        Ok(())
    })();
    match checked {
        Ok(()) => result.reward = 1,
        Err(e) => {
            result.error = Some(e.0);
            result.scoped_report = None;
        }
    }
    Ok(result)
}
