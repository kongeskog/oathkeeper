//! Signed, append-only review/evidence journals. Trusted keys come from the host.
//! A valid signature establishes possession of a key, never truth or expertise.
use crate::evaluation::{context_digest, digest};
use crate::kernel::{identifier, require, timestamp, unique};
use crate::*;
use ed25519_dalek::{Signature, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrustedKey {
    pub id: String,
    pub principal: String,
    pub public_key: String,
    pub valid_from: String,
    pub valid_until: Option<String>,
    pub revoked_at: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrustStore {
    pub keys: Vec<TrustedKey>,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewAspect {
    LegalFidelity,
    Formalization,
    SourceAccuracy,
    Coverage,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewDecision {
    Approved,
    ChangesRequested,
    Withdrawn,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ReceiptPayload {
    Review {
        artifact_sha256: String,
        aspect: ReviewAspect,
        decision: ReviewDecision,
        independence_declared: bool,
        rationale: String,
    },
    Evidence {
        context_sha256: String,
        target_kind: String,
        target: String,
        payload_sha256: String,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Receipt {
    pub id: String,
    pub sequence: usize,
    pub previous_sha256: Option<String>,
    pub signer: String,
    pub recorded_at: String,
    pub payload: ReceiptPayload,
    pub signature: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Journal {
    pub format: String,
    pub events: Vec<Receipt>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalReport {
    pub verified_events: usize,
    pub head_sha256: Option<String>,
    pub authentication: String,
    pub history_completeness: String,
    pub timestamp_assurance: String,
}
#[derive(Debug, Serialize)]
pub struct ReviewState {
    pub principal: String,
    pub aspect: ReviewAspect,
    pub decision: ReviewDecision,
    pub independence_declared: bool,
    pub recorded_at: String,
    pub receipt: String,
}
#[derive(Debug, Serialize)]
pub struct ReviewReport {
    pub artifact_sha256: String,
    pub journal: JournalReport,
    pub current: Vec<ReviewState>,
    pub stale_events: usize,
    pub legal_correctness: String,
}
#[derive(Debug, Serialize)]
pub struct EvidenceAuthentication {
    pub context_sha256: String,
    pub journal: JournalReport,
    pub authenticated: Vec<String>,
    pub missing: Vec<String>,
    pub status: String,
    pub meaning: String,
}
fn hex<const N: usize>(text: &str) -> Result<[u8; N]> {
    require(
        text.len() == N * 2
            && text
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
        "expected lowercase fixed-length hex",
    )?;
    let mut bytes = [0u8; N];
    for (i, b) in bytes.iter_mut().enumerate() {
        *b = u8::from_str_radix(&text[i * 2..i * 2 + 2], 16).map_err(|e| Error(e.to_string()))?;
    }
    Ok(bytes)
}
pub fn signing_bytes(receipt: &Receipt) -> Result<Vec<u8>> {
    let mut value = serde_json::to_value(receipt).map_err(|e| Error(e.to_string()))?;
    value.as_object_mut().unwrap().remove("signature");
    let mut bytes = b"oathkeeper.receipt/1\n".to_vec();
    bytes.extend(serde_json::to_vec(&value).map_err(|e| Error(e.to_string()))?);
    Ok(bytes)
}
pub fn verify_journal(journal: &Journal, trust: &TrustStore) -> Result<JournalReport> {
    require(
        journal.format == "oathkeeper.journal/1",
        "unknown journal format",
    )?;
    require(
        journal.events.len() <= 10000 && trust.keys.len() <= 1000,
        "[E_LIMIT] journal/key budget exceeded",
    )?;
    unique(&journal.events, |e| &e.id)?;
    let keys = unique(&trust.keys, |k| &k.id)?;
    for key in &trust.keys {
        require(
            !key.principal.trim().is_empty(),
            "trusted key requires a principal",
        )?;
        let start = timestamp(&key.valid_from)?;
        for end in [&key.valid_until, &key.revoked_at].into_iter().flatten() {
            require(start <= timestamp(end)?, "invalid key validity interval")?;
        }
        VerifyingKey::from_bytes(&hex::<32>(&key.public_key)?).map_err(|e| Error(e.to_string()))?;
    }
    let mut previous = None;
    let mut time = None;
    for (sequence, event) in journal.events.iter().enumerate() {
        require(
            event.sequence == sequence && event.previous_sha256 == previous,
            "[E_JOURNAL] missing, reordered or forked event",
        )?;
        let at = timestamp(&event.recorded_at)?;
        require(
            time.is_none_or(|t| t <= at),
            "journal times must be nondecreasing",
        )?;
        time = Some(at);
        let key = keys
            .get(event.signer.as_str())
            .ok_or_else(|| Error("[E_SIGNER] signer is not in the host trust store".into()))?;
        require(
            timestamp(&key.valid_from)? <= at,
            "signature predates key validity",
        )?;
        for end in [&key.valid_until, &key.revoked_at].into_iter().flatten() {
            require(
                at < timestamp(end)?,
                "signature is outside key validity or after revocation",
            )?;
        }
        match &event.payload {
            ReceiptPayload::Review {
                artifact_sha256,
                rationale,
                ..
            } => {
                hex::<32>(artifact_sha256)?;
                require(
                    !rationale.trim().is_empty(),
                    "review requires an explanation",
                )?;
            }
            ReceiptPayload::Evidence {
                context_sha256,
                target_kind,
                target,
                payload_sha256,
            } => {
                hex::<32>(context_sha256)?;
                hex::<32>(payload_sha256)?;
                identifier(target)?;
                require(
                    matches!(
                        target_kind.as_str(),
                        "fact" | "attestation" | "oracle_policy" | "retraction"
                    ),
                    "unknown evidence target kind",
                )?;
            }
        }
        let verifying = VerifyingKey::from_bytes(&hex::<32>(&key.public_key)?)
            .map_err(|e| Error(e.to_string()))?;
        let signature = Signature::from_bytes(&hex::<64>(&event.signature)?);
        verifying
            .verify_strict(&signing_bytes(event)?, &signature)
            .map_err(|_| Error("[E_SIGNATURE] receipt signature is invalid".into()))?;
        previous = Some(digest(event)?);
    }
    Ok(JournalReport {
        verified_events: journal.events.len(), head_sha256: previous,
        authentication: "ed25519 signatures checked against caller-supplied trusted keys; principal bindings are host assertions".into(),
        history_completeness: "Chain integrity checked. Pin the expected head externally to detect suffix removal or replay; unpublished events remain unknown.".into(),
        timestamp_assurance: "Signer-declared times; no trusted timestamp service or protection against signer backdating.".into(),
    })
}
pub fn review_status(
    artifact: &str,
    journal: &Journal,
    trust: &TrustStore,
    authors: &[String],
) -> Result<ReviewReport> {
    hex::<32>(artifact)?;
    let checked = verify_journal(journal, trust)?;
    let keys = unique(&trust.keys, |k| &k.id)?;
    let mut current = BTreeMap::new();
    let mut stale = 0;
    for event in &journal.events {
        if let ReceiptPayload::Review {
            artifact_sha256,
            aspect,
            decision,
            independence_declared,
            ..
        } = &event.payload
        {
            if artifact_sha256 != artifact {
                stale += 1;
                continue;
            }
            let principal = &keys[event.signer.as_str()].principal;
            require(
                !*independence_declared || !authors.contains(principal),
                "an author cannot declare an independent review of their own artifact",
            )?;
            current.insert(
                (principal.clone(), aspect.clone()),
                ReviewState {
                    principal: principal.clone(),
                    aspect: aspect.clone(),
                    decision: decision.clone(),
                    independence_declared: *independence_declared,
                    recorded_at: event.recorded_at.clone(),
                    receipt: event.id.clone(),
                },
            );
        }
    }
    Ok(ReviewReport { artifact_sha256: artifact.into(), journal: checked,
        current: current.into_values().collect(), stale_events: stale,
        legal_correctness: "Not established by signatures or review counts. Community review statements are shown individually.".into() })
}
/// Authenticate every supplied evidence object at the query cutoff. No missing receipt is treated as valid.
pub fn authenticate_evidence(
    bundle: &Bundle,
    journal: &Journal,
    trust: &TrustStore,
    expected_head: &str,
) -> Result<EvidenceAuthentication> {
    validate_context(bundle)?;
    let checked = verify_journal(journal, trust)?;
    require(
        checked.head_sha256.as_deref() == Some(expected_head),
        "[E_JOURNAL] journal does not match the host-pinned head",
    )?;
    let context = context_digest(bundle)?;
    let cutoff = timestamp(&bundle.query.known_at)?;
    let keys = unique(&trust.keys, |k| &k.id)?;
    let mut entities = Vec::new();
    for fact in &bundle.facts {
        entities.push(("fact", &fact.id, digest(fact)?, None));
    }
    for a in &bundle.attestations {
        entities.push(("attestation", &a.id, digest(a)?, Some(&a.provider)));
    }
    for policy in &bundle.oracle_policy {
        entities.push(("oracle_policy", &policy.oracle, digest(policy)?, None));
    }
    for event in &bundle.retractions {
        entities.push(("retraction", &event.id, digest(event)?, None));
    }
    let mut authenticated = Vec::new();
    let mut missing = Vec::new();
    for (kind, id, body, provider) in entities {
        let found = journal.events.iter().any(|event| {
            if !timestamp(&event.recorded_at).is_ok_and(|t| t <= cutoff) {
                return false;
            }
            match &event.payload {
                ReceiptPayload::Evidence {
                    context_sha256,
                    target_kind,
                    target,
                    payload_sha256,
                } => {
                    context_sha256 == &context
                        && target_kind == kind
                        && target == id
                        && payload_sha256 == &body
                        && provider.is_none_or(|p| &keys[event.signer.as_str()].principal == p)
                }
                _ => false,
            }
        });
        let name = format!("{kind}:{id}");
        if found {
            authenticated.push(name);
        } else {
            missing.push(name);
        }
    }
    let status = if missing.is_empty() {
        "all_supplied_evidence_authenticated"
    } else {
        "missing_authentication"
    }
    .into();
    Ok(EvidenceAuthentication { context_sha256: context, journal: checked, authenticated, missing, status,
        meaning: "Signatures bind exact evidence objects, request procedures and context to trusted keys. Truth, calibration, identity and legal authority remain separate.".into() })
}
