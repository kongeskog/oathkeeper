//! Frozen evidence protocols. These check declared agreement, not provider identity.
use crate::kernel::{require, source, timestamp, unique};
use crate::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleAssessment {
    pub request: String,
    pub role: Option<OracleRole>,
    pub agreement: Option<OracleAgreement>,
    pub status: String,
    pub responses: Vec<String>,
    pub accepted_responses: Vec<String>,
    pub claim: Option<Claim>,
    pub confidence_meaning: Option<String>,
    pub authentication: String,
}

pub(crate) fn retracted(bundle: &Bundle, kind: &EvidenceKind, id: &str) -> Result<bool> {
    let cutoff = timestamp(&bundle.query.known_at)?;
    for event in &bundle.retractions {
        if &event.target_kind == kind
            && event.target == id
            && timestamp(&event.recorded_at)? <= cutoff
        {
            return Ok(true);
        }
    }
    Ok(false)
}

pub(crate) fn validate_evidence_protocols(bundle: &Bundle) -> Result<()> {
    if bundle.language != "oathkeeper/0.4" {
        return Ok(());
    }
    unique(&bundle.retractions, |r| &r.id)?;
    for event in &bundle.retractions {
        source(&event.source)?;
        require(
            !event.rationale.trim().is_empty(),
            "retraction requires a rationale",
        )?;
        let original = match event.target_kind {
            EvidenceKind::Fact => bundle
                .facts
                .iter()
                .find(|f| f.id == event.target)
                .map(|f| &f.recorded_at),
            EvidenceKind::Attestation => bundle
                .attestations
                .iter()
                .find(|a| a.id == event.target)
                .map(|a| &a.recorded_at),
        }
        .ok_or_else(|| Error("retraction targets unknown evidence".into()))?;
        require(
            timestamp(&event.recorded_at)? >= timestamp(original)?,
            "retraction predates its evidence",
        )?;
    }
    let cutoff = timestamp(&bundle.query.known_at)?;
    let mut live_responses = BTreeSet::new();
    for policy in &bundle.oracle_policy {
        let protocol = policy
            .protocol
            .as_ref()
            .ok_or_else(|| Error("0.4 oracle policy requires a protocol".into()))?;
        require(
            !protocol.procedure.trim().is_empty(),
            "oracle procedure is required",
        )?;
        let providers: BTreeSet<_> = policy.providers.iter().map(|p| &p.id).collect();
        require(
            providers.len() == policy.providers.len(),
            "oracle providers must have distinct IDs",
        )?;
        if protocol.agreement == OracleAgreement::Single {
            require(
                policy.providers.len() == 1,
                "single agreement requires exactly one provider",
            )?;
        }
        if policy.min_confidence.is_some()
            || bundle
                .attestations
                .iter()
                .any(|a| a.oracle == policy.oracle && a.confidence.is_some())
        {
            require(
                protocol
                    .confidence_meaning
                    .as_ref()
                    .is_some_and(|s| !s.trim().is_empty()),
                "numeric confidence requires an explicit confidence_meaning",
            )?;
        }
    }
    for a in &bundle.attestations {
        require(
            a.disposition.is_some(),
            "0.4 response requires asserted or abstained disposition",
        )?;
        let policy = bundle
            .oracle_policy
            .iter()
            .find(|p| p.oracle == a.oracle)
            .ok_or_else(|| Error("oracle response has no agreed request".into()))?;
        require(
            policy
                .providers
                .iter()
                .any(|p| p.id == a.provider && p.version == a.provider_version),
            "oracle provider or version not agreed",
        )?;
        require(
            a.claim.atom == policy.atom
                && a.claim.arguments == policy.arguments
                && a.question == policy.question
                && a.evidence_sha256 == policy.evidence_sha256,
            "oracle output does not match agreed request",
        )?;
        if timestamp(&a.recorded_at)? <= cutoff
            && !retracted(bundle, &EvidenceKind::Attestation, &a.id)?
        {
            require(live_responses.insert((&a.oracle, &a.provider, &a.provider_version)), "[E_ORACLE_CONFLICT] multiple live responses from one provider; withdraw the old response first")?;
        }
    }
    Ok(())
}

pub fn assess_oracles(bundle: &Bundle) -> Result<Vec<OracleAssessment>> {
    require(
        bundle.language == "oathkeeper/0.4",
        "oracle protocol assessment requires oathkeeper/0.4",
    )?;
    validate_context(bundle)?;
    bundle
        .oracle_policy
        .iter()
        .map(|p| assess_request(bundle, p))
        .collect()
}

pub(crate) fn assess_request(bundle: &Bundle, policy: &OraclePolicy) -> Result<OracleAssessment> {
    let cutoff = timestamp(&bundle.query.known_at)?;
    let mut visible = Vec::new();
    for a in &bundle.attestations {
        if a.oracle == policy.oracle
            && timestamp(&a.recorded_at)? <= cutoff
            && !retracted(bundle, &EvidenceKind::Attestation, &a.id)?
        {
            visible.push(a);
        }
    }
    visible.sort_by(|a, b| a.id.cmp(&b.id));
    let mut result = OracleAssessment {
        request: policy.oracle.clone(),
        role: policy.protocol.as_ref().map(|p| p.role.clone()),
        agreement: policy.protocol.as_ref().map(|p| p.agreement.clone()),
        status: "pending".into(),
        responses: visible.iter().map(|a| a.id.clone()).collect(),
        accepted_responses: Vec::new(),
        claim: None,
        confidence_meaning: policy
            .protocol
            .as_ref()
            .and_then(|p| p.confidence_meaning.clone()),
        authentication: "provider_identity_not_authenticated".into(),
    };
    if visible.is_empty() {
        return Ok(result);
    }
    if visible
        .iter()
        .any(|a| a.disposition == Some(ResponseDisposition::Abstained))
    {
        result.status = "abstained".into();
        return Ok(result);
    }
    if visible.iter().any(|a| a.claim != visible[0].claim) {
        result.status = "disputed".into();
        return Ok(result);
    }
    if visible.len() != policy.providers.len() {
        result.status = "pending".into();
        return Ok(result);
    }
    if policy.min_confidence.is_some_and(|threshold| {
        visible
            .iter()
            .any(|a| a.confidence.is_none_or(|c| c < threshold))
    }) {
        result.status = "threshold_not_met".into();
        return Ok(result);
    }
    result.status = "accepted_under_protocol".into();
    result.accepted_responses = result.responses.clone();
    result.claim = Some(visible[0].claim.clone());
    Ok(result)
}

pub(crate) fn accepted_responses(bundle: &Bundle, policy: &OraclePolicy) -> Result<Vec<String>> {
    Ok(assess_request(bundle, policy)?.accepted_responses)
}
