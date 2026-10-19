//! One strict JSON interface shared by CLI, Python and WebAssembly.
use crate::{contracts::*, evaluation::*, receipts::*, *};
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
enum Request {
    Coverage {
        bundle: Bundle,
        #[serde(default)]
        options: AnalysisOptions,
    },
    VerifyScoped {
        bundle: Bundle,
        conditions: Vec<ConditionProof>,
    },
    Freeze {
        bundle: Bundle,
        policy: EvaluationPolicy,
    },
    Evaluate {
        task: FrozenTask,
        expected_sha256: String,
        candidate: Value,
    },
    Journal {
        journal: Journal,
        trust: TrustStore,
    },
    Reviews {
        artifact_sha256: String,
        journal: Journal,
        trust: TrustStore,
        authors: Vec<String>,
    },
    Authenticate {
        bundle: Bundle,
        journal: Journal,
        trust: TrustStore,
        expected_head_sha256: String,
    },
    SigningPayload {
        receipt: Receipt,
    },
    Digest {
        value: Value,
    },
}

pub fn policy_json(input: &str) -> Result<String> {
    if input.len() > 16 * 1024 * 1024 {
        return Err(Error("[E_LIMIT] policy request exceeds 16 MiB".into()));
    }
    let request: Request = serde_json::from_str(input).map_err(|e| Error(e.to_string()))?;
    let output = match request {
        Request::Coverage { bundle, options } => json!(assess_coverage(&bundle, &options)?),
        Request::VerifyScoped { bundle, conditions } => json!(verify_scoped(&bundle, &conditions)?),
        Request::Freeze { bundle, policy } => json!(freeze_task(&bundle, policy)?),
        Request::Evaluate {
            task,
            expected_sha256,
            candidate,
        } => json!(evaluate_task(
            &task,
            &expected_sha256,
            &candidate.to_string()
        )?),
        Request::Journal { journal, trust } => json!(verify_journal(&journal, &trust)?),
        Request::Reviews {
            artifact_sha256,
            journal,
            trust,
            authors,
        } => json!(review_status(&artifact_sha256, &journal, &trust, &authors)?),
        Request::Authenticate {
            bundle,
            journal,
            trust,
            expected_head_sha256,
        } => json!(authenticate_evidence(
            &bundle,
            &journal,
            &trust,
            &expected_head_sha256
        )?),
        Request::SigningPayload { receipt } => {
            json!({"utf8": String::from_utf8(signing_bytes(&receipt)?).map_err(|e| Error(e.to_string()))?})
        }
        Request::Digest { value } => json!({"sha256": digest(&value)?}),
    };
    serde_json::to_string(&output).map_err(|e| Error(e.to_string()))
}
