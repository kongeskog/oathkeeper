//! Explicit coverage and composition policy. Coverage is declared, never inferred from corpus size.
use crate::kernel::{identifier, instantiate, require, source, timestamp, unique};
use crate::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Interpretation {
    pub id: String,
    pub issue: String,
    pub description: String,
    pub source: Source,
    pub pinpoint: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConclusionKind {
    Factual,
    Permission,
    Prohibition,
    Obligation,
    Liability,
    Remedy,
    Calculation,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoverageRequirement {
    pub id: String,
    pub goal: Claim,
    pub explanation: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoverageSource {
    pub source: Source,
    pub treatment: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoverageContract {
    pub id: String,
    pub question: String,
    pub conclusion_kind: ConclusionKind,
    pub parameters: Vec<Parameter>,
    pub goal: Claim,
    pub jurisdictions: Vec<String>,
    pub forums: Vec<String>,
    pub law_from: String,
    pub law_until: String,
    pub requirements: Vec<CoverageRequirement>,
    pub source_inventory: Vec<CoverageSource>,
    pub exclusions: Vec<String>,
    pub known_omissions: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeCondition {
    pub id: String,
    pub goal: Claim,
    pub status: SupportStatus,
    pub search_complete: bool,
    pub supporting: Vec<CheckedArgument>,
    pub opposing: Vec<CheckedArgument>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageAssessment {
    pub status: String,
    pub context_sha256: String,
    pub contract: Option<CoverageContract>,
    pub conditions: Vec<ScopeCondition>,
    pub reasons: Vec<String>,
    pub completeness: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConditionProof {
    pub id: String,
    pub root: String,
    pub proof: Vec<Step>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopedReport {
    pub status: String,
    pub context_sha256: String,
    pub contract: CoverageContract,
    pub derivation: Report,
    pub conditions: Vec<Report>,
    pub limitations: String,
}

pub(crate) fn validate(bundle: &Bundle) -> Result<()> {
    let enabled = bundle.language == "oathkeeper/0.5";
    require(
        enabled
            || (bundle.coverage.is_empty()
                && bundle.interpretations.is_empty()
                && bundle.query.coverage.is_none()
                && bundle.query.interpretations.is_empty()
                && bundle
                    .rules
                    .iter()
                    .all(|r| r.exceptions.is_empty() && r.interpretation.is_none())),
        "[E_VERSION] coverage, exceptions and interpretation selection require oathkeeper/0.5",
    )?;
    require(
        bundle.coverage.len() <= 64 && bundle.interpretations.len() <= 256,
        "[E_LIMIT] policy budget exceeded",
    )?;
    let readings = unique(&bundle.interpretations, |r| &r.id)?;
    for reading in &bundle.interpretations {
        identifier(&reading.issue)?;
        source(&reading.source)?;
        require(
            !reading.description.trim().is_empty() && !reading.pinpoint.trim().is_empty(),
            "interpretation needs rationale and pinpoint",
        )?;
    }
    let mut issues = BTreeSet::new();
    for id in &bundle.query.interpretations {
        let reading = readings
            .get(id.as_str())
            .ok_or_else(|| Error("[E_INTERPRETATION] unknown selection".into()))?;
        require(
            issues.insert(&reading.issue),
            "[E_INTERPRETATION] select at most one reading per issue",
        )?;
    }
    for rule in &bundle.rules {
        if let Some(id) = &rule.interpretation {
            require(
                readings.contains_key(id.as_str()),
                "rule references an unknown interpretation",
            )?;
        }
        require(rule.exceptions.len() <= 64, "[E_LIMIT] too many exceptions")?;
        let mut seen = BTreeSet::new();
        for exception in &rule.exceptions {
            require(
                !exception.negative && seen.insert(exception),
                "exceptions must be unique positive claims",
            )?;
        }
    }
    let contracts = unique(&bundle.coverage, |c| &c.id)?;
    if let Some(id) = &bundle.query.coverage {
        require(
            contracts.contains_key(id.as_str()),
            "[E_COVERAGE] unknown selected coverage contract",
        )?;
    }
    let schema = crate::schema::Schema::new(&bundle.types, &bundle.predicates, &bundle.constants)?;
    for contract in &bundle.coverage {
        require(
            !contract.question.trim().is_empty()
                && !contract.source_inventory.is_empty()
                && !contract.exclusions.is_empty()
                && !contract.known_omissions.is_empty(),
            "coverage requires a question, source inventory, exclusions and explicit omissions",
        )?;
        require(
            contract.requirements.len() <= 24 && contract.parameters.len() <= 32,
            "[E_LIMIT] coverage condition budget exceeded",
        )?;
        require(
            !contract.jurisdictions.is_empty() && !contract.forums.is_empty(),
            "coverage must bound jurisdiction and forum",
        )?;
        require(
            timestamp(&contract.law_from)? < timestamp(&contract.law_until)?,
            "invalid coverage law interval",
        )?;
        for id in &contract.jurisdictions {
            require(
                bundle.jurisdictions.iter().any(|j| &j.id == id),
                "unknown coverage jurisdiction",
            )?;
        }
        let parameters = unique(&contract.parameters, |p| &p.name)?;
        schema.claim(&contract.goal, &parameters)?;
        let goal_variables: BTreeSet<_> = contract
            .goal
            .arguments
            .iter()
            .filter_map(|t| match t {
                Term::Variable { name } => Some(name.as_str()),
                _ => None,
            })
            .collect();
        require(
            parameters.keys().copied().collect::<BTreeSet<_>>() == goal_variables,
            "every coverage parameter must be bound by the goal",
        )?;
        unique(&contract.requirements, |r| &r.id)?;
        for condition in &contract.requirements {
            require(
                condition.goal != contract.goal && !condition.explanation.trim().is_empty(),
                "coverage condition must be explained and distinct from the goal",
            )?;
            schema.claim(&condition.goal, &parameters)?;
        }
        for entry in &contract.source_inventory {
            source(&entry.source)?;
            require(
                !entry.treatment.trim().is_empty(),
                "coverage inventory needs source treatment",
            )?;
            require(
                bundle.authorities.iter().any(|a| {
                    a.source.sha256 == entry.source.sha256 && a.source.uri == entry.source.uri
                }),
                "coverage source must be declared by an authority",
            )?;
        }
    }
    Ok(())
}

pub(crate) fn interpretation_eligible(bundle: &Bundle, rule: &Rule) -> Result<()> {
    if let Some(id) = &rule.interpretation {
        require(
            bundle.query.interpretations.contains(id),
            "[E_INTERPRETATION] rule reading is not selected",
        )?;
    }
    Ok(())
}

fn selected(bundle: &Bundle) -> Result<&CoverageContract> {
    bundle
        .query
        .coverage
        .as_ref()
        .and_then(|id| bundle.coverage.iter().find(|c| &c.id == id))
        .ok_or_else(|| Error("[E_COVERAGE] select an explicit coverage contract".into()))
}
fn bind(bundle: &Bundle, contract: &CoverageContract) -> Result<BTreeMap<String, Term>> {
    let goal = &bundle.query.goal;
    require(
        goal.atom == contract.goal.atom
            && goal.negative == contract.goal.negative
            && goal.arguments.len() == contract.goal.arguments.len(),
        "goal is outside the selected contract",
    )?;
    require(
        contract.jurisdictions.contains(&bundle.query.jurisdiction),
        "jurisdiction is outside the selected contract",
    )?;
    require(
        bundle
            .query
            .forum
            .as_ref()
            .is_some_and(|f| contract.forums.contains(f)),
        "forum is outside the selected contract",
    )?;
    let time = timestamp(&bundle.query.as_of)?;
    require(
        timestamp(&contract.law_from)? <= time && time < timestamp(&contract.law_until)?,
        "law time is outside the selected contract",
    )?;
    let mut bindings = BTreeMap::new();
    for (pattern, actual) in contract.goal.arguments.iter().zip(&goal.arguments) {
        match pattern {
            Term::Variable { name } => {
                if let Some(previous) = bindings.insert(name.clone(), actual.clone()) {
                    require(
                        previous == *actual,
                        "coverage variable has inconsistent bindings",
                    )?;
                }
            }
            _ => require(pattern == actual, "goal arguments are outside the contract")?,
        }
    }
    Ok(bindings)
}

pub fn assess_coverage(bundle: &Bundle, options: &AnalysisOptions) -> Result<CoverageAssessment> {
    validate_context(bundle)?;
    let mut result = CoverageAssessment {
        status: "not_declared".into(),
        context_sha256: crate::evaluation::context_digest(bundle)?,
        contract: None,
        conditions: Vec::new(),
        reasons: Vec::new(),
        completeness:
            "Declared bounded coverage; legal fidelity and completeness are not established.".into(),
    };
    if bundle.query.coverage.is_none() {
        return Ok(result);
    }
    let contract = selected(bundle)?;
    result.contract = Some(contract.clone());
    let bindings = match bind(bundle, contract) {
        Ok(b) => b,
        Err(e) => {
            result.status = "outside_scope".into();
            result.reasons.push(e.0);
            return Ok(result);
        }
    };
    result.status = "conditions_supported".into();
    for requirement in &contract.requirements {
        let mut context = bundle.clone();
        context.proof.clear();
        context.query.coverage = None;
        context.query.goal = instantiate(&requirement.goal, &bindings)?;
        let analysis = crate::analysis::run(
            &context,
            &AnalysisOptions {
                sensitivity: false,
                ..options.clone()
            },
        )?;
        result.conditions.push(ScopeCondition {
            id: requirement.id.clone(),
            goal: context.query.goal,
            status: analysis.status,
            search_complete: analysis.search.complete_for_ground_domain,
            supporting: analysis.supporting,
            opposing: analysis.opposing,
        });
    }
    if result
        .conditions
        .iter()
        .any(|c| c.status == SupportStatus::Both)
    {
        result.status = "conflicting_scope_evidence".into();
    } else if result
        .conditions
        .iter()
        .any(|c| c.status == SupportStatus::Opposed && c.search_complete)
    {
        result.status = "outside_scope".into();
    } else if result
        .conditions
        .iter()
        .any(|c| c.status != SupportStatus::Supported || !c.search_complete)
    {
        result.status = "needs_information".into();
    }
    Ok(result)
}

/// Check the main proof and every coverage condition against the same host context.
/// It checks positive certificates, not absence of counterarguments or corpus completeness.
pub fn verify_scoped(bundle: &Bundle, conditions: &[ConditionProof]) -> Result<ScopedReport> {
    validate_context(bundle)?;
    let contract = selected(bundle)?;
    let bindings = bind(bundle, contract)?;
    let supplied = unique(conditions, |c| &c.id)?;
    require(
        supplied.len() == contract.requirements.len(),
        "[E_COVERAGE] supply exactly every coverage condition",
    )?;
    let mut reports = Vec::new();
    for condition in &contract.requirements {
        let proof = supplied
            .get(condition.id.as_str())
            .ok_or_else(|| Error("[E_COVERAGE] missing condition proof".into()))?;
        let mut context = bundle.clone();
        context.query.coverage = None;
        context.query.goal = instantiate(&condition.goal, &bindings)?;
        context.query.root = proof.root.clone();
        context.proof = proof.proof.clone();
        reports.push(verify(&context)?);
    }
    Ok(ScopedReport {
        status: "derivation_and_scope_conditions_verified".into(),
        context_sha256: crate::evaluation::context_digest(bundle)?, contract: contract.clone(),
        derivation: verify(bundle)?, conditions: reports,
        limitations: "Conditional derivations only. Legal fidelity, completeness, input truth and absence of counterarguments are not certified.".into(),
    })
}
