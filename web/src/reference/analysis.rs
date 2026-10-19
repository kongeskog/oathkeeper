//! Bounded finite-domain proof search, outside the trusted derivation checker.
//! Every returned argument is checked by `verify`. Exhaustion never proves negation.
use crate::kernel::{compare, instantiate, rule_eligibility, timestamp};
use crate::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AnalysisOptions {
    pub max_groundings: usize,
    pub max_nodes: usize,
    pub max_depth: usize,
    pub max_arguments: usize,
    pub sensitivity: bool,
    pub max_sensitivity: usize,
}
impl Default for AnalysisOptions {
    fn default() -> Self {
        Self {
            max_groundings: 10000,
            max_nodes: 20000,
            max_depth: 48,
            max_arguments: 32,
            sensitivity: true,
            max_sensitivity: 24,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SupportStatus {
    Supported,
    Opposed,
    Both,
    Neither,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckedArgument {
    pub proof: Vec<Step>,
    pub report: Report,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementGap {
    pub claim: Claim,
    pub needed_by: Vec<String>,
    pub oracle_requests: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcludedRule {
    pub rule: String,
    pub version: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchSummary {
    pub complete_for_ground_domain: bool,
    pub ground_terms: Vec<Term>,
    pub ground_rules: usize,
    pub nodes_visited: usize,
    pub limits_reached: Vec<String>,
    pub meaning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sensitivity {
    pub kind: String,
    pub id: String,
    pub without: SupportStatus,
    pub search_complete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Analysis {
    pub format: String,
    pub kernel_version: String,
    pub context_sha256: String,
    pub status: SupportStatus,
    pub goal: Claim,
    pub supporting: Vec<CheckedArgument>,
    pub opposing: Vec<CheckedArgument>,
    pub open_requirements: Vec<RequirementGap>,
    pub excluded_rules: Vec<ExcludedRule>,
    pub oracle_requests: Vec<OracleAssessment>,
    pub search: SearchSummary,
    pub sensitivity: Vec<Sensitivity>,
    pub sensitivity_complete: bool,
    pub legal_conclusion: String,
}

#[derive(Clone)]
struct GroundRule {
    rule: Rule,
    substitution: BTreeMap<String, Term>,
    premises: Vec<Claim>,
    conclusion: Claim,
}

struct Search<'a> {
    bundle: &'a Bundle,
    options: &'a AnalysisOptions,
    rules: Vec<GroundRule>,
    leaves: BTreeMap<Claim, Vec<Step>>,
    nodes: usize,
    limits: BTreeSet<String>,
    gaps: BTreeMap<Claim, BTreeSet<String>>,
}

fn signed_opposite(claim: &Claim) -> Claim {
    let mut c = claim.clone();
    c.negative = !c.negative;
    c
}
fn hash<T: Serialize>(value: &T) -> String {
    format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(value).expect("serializable proof data"))
    )
}
fn step(claim: Claim, reason: Reason) -> Step {
    Step {
        id: format!("s{}", hash(&(&claim, &reason))),
        claim,
        reason,
    }
}
fn operator(claim: &Claim) -> Option<Comparison> {
    Some(match claim.atom.as_str() {
        "oathkeeper.integer.gt" => Comparison::GreaterThan,
        "oathkeeper.integer.lt" => Comparison::LessThan,
        "oathkeeper.integer.eq" => Comparison::Equal,
        "oathkeeper.quantity.gt" => Comparison::QuantityGreaterThan,
        "oathkeeper.quantity.lt" => Comparison::QuantityLessThan,
        "oathkeeper.quantity.eq" => Comparison::QuantityEqual,
        _ => return None,
    })
}

fn merge(left: &[Step], right: &[Step]) -> Option<Vec<Step>> {
    let mut claims: BTreeSet<Claim> = left.iter().map(|s| s.claim.clone()).collect();
    let mut ids: BTreeSet<String> = left.iter().map(|s| s.id.clone()).collect();
    let mut result = left.to_vec();
    for s in right {
        if claims.contains(&signed_opposite(&s.claim)) {
            return None;
        }
        claims.insert(s.claim.clone());
        if ids.insert(s.id.clone()) {
            result.push(s.clone());
        }
    }
    Some(result)
}

impl Search<'_> {
    fn prove(&mut self, goal: &Claim, path: &mut BTreeSet<Claim>, depth: usize) -> Vec<Vec<Step>> {
        if self.nodes >= self.options.max_nodes {
            self.limits.insert("nodes".into());
            return Vec::new();
        }
        if depth > self.options.max_depth {
            self.limits.insert("depth".into());
            return Vec::new();
        }
        if !path.insert(goal.clone()) {
            return Vec::new();
        }
        self.nodes += 1;
        let mut found: Vec<Vec<Step>> = self
            .leaves
            .get(goal)
            .into_iter()
            .flatten()
            .map(|s| vec![s.clone()])
            .collect();
        if let Some(operator) = operator(goal) {
            if compare(goal, &operator).is_ok() {
                found.push(vec![step(goal.clone(), Reason::Compare { operator })]);
            }
        }
        if let Some(operator) = crate::arithmetic::operator(goal) {
            if crate::arithmetic::compute(goal, &operator).is_ok() {
                found.push(vec![step(goal.clone(), Reason::Compute { operator })]);
            }
        }
        let matching: Vec<_> = self
            .rules
            .iter()
            .filter(|r| r.conclusion == *goal)
            .cloned()
            .collect();
        for rule in matching {
            if self.nodes >= self.options.max_nodes {
                self.limits.insert("nodes".into());
                break;
            }
            let mut combinations: Vec<(Vec<Step>, Vec<String>)> = vec![(Vec::new(), Vec::new())];
            for premise in &rule.premises {
                let candidates = self.prove(premise, path, depth + 1);
                if candidates.is_empty() {
                    self.gaps
                        .entry(premise.clone())
                        .or_default()
                        .insert(format!("{}@{}", rule.rule.id, rule.rule.version));
                }
                let mut next = Vec::new();
                for (prefix, refs) in &combinations {
                    for candidate in &candidates {
                        if let Some(joined) = merge(prefix, candidate) {
                            let mut references = refs.clone();
                            references.push(candidate.last().unwrap().id.clone());
                            next.push((joined, references));
                            if next.len() >= self.options.max_arguments {
                                self.limits.insert("argument_combinations".into());
                                break;
                            }
                        }
                    }
                    if next.len() >= self.options.max_arguments {
                        break;
                    }
                }
                combinations = next;
            }
            for (mut proof, premises) in combinations {
                if proof.iter().any(|s| s.claim == signed_opposite(goal)) {
                    continue;
                }
                proof.push(step(
                    goal.clone(),
                    Reason::Apply {
                        rule: rule.rule.id.clone(),
                        version: rule.rule.version.clone(),
                        premises,
                        substitution: rule.substitution.clone(),
                    },
                ));
                if proof.len() <= 2000 {
                    found.push(proof);
                } else {
                    self.limits.insert("proof_size".into());
                }
                if found.len() >= self.options.max_arguments {
                    self.limits.insert("arguments".into());
                    break;
                }
            }
            if found.len() >= self.options.max_arguments {
                break;
            }
        }
        if found.len() > self.options.max_arguments {
            found.truncate(self.options.max_arguments);
            self.limits.insert("arguments".into());
        }
        path.remove(goal);
        if found.is_empty() {
            self.gaps.entry(goal.clone()).or_default();
        }
        found
    }

    fn checked(&mut self, goal: &Claim) -> Result<Vec<CheckedArgument>> {
        let candidates = self.prove(goal, &mut BTreeSet::new(), 0);
        let mut results = Vec::new();
        let mut seen = BTreeSet::new();
        for mut proof in candidates {
            if !seen.insert(hash(&proof)) {
                continue;
            }
            let mut candidate = self.bundle.clone();
            candidate.query.goal = goal.clone();
            // Root is a host-owned output identifier, not an evidence identifier.
            let root_id = candidate.query.root.clone();
            if proof
                .iter()
                .take(proof.len().saturating_sub(1))
                .any(|s| s.id == root_id)
            {
                return Err(Error("query root collides with generated proof ID".into()));
            }
            proof.last_mut().unwrap().id = root_id;
            candidate.proof = proof.clone();
            let report = verify(&candidate).map_err(|e| Error(format!("[E_SEARCH_CERTIFICATE] generated candidate failed independent verification: {e}")))?;
            results.push(CheckedArgument { proof, report });
        }
        Ok(results)
    }
}

fn terms(bundle: &Bundle) -> BTreeSet<Term> {
    let mut values: BTreeSet<Term> = bundle
        .constants
        .iter()
        .map(|c| Term::Symbol {
            value: c.id.clone(),
        })
        .collect();
    for c in bundle
        .rules
        .iter()
        .flat_map(|r| r.premises.iter().chain(std::iter::once(&r.conclusion)))
        .chain(bundle.facts.iter().map(|f| &f.claim))
        .chain(bundle.attestations.iter().map(|a| &a.claim))
        .chain(std::iter::once(&bundle.query.goal))
    {
        values.extend(
            c.arguments
                .iter()
                .filter(|t| !matches!(t, Term::Variable { .. }))
                .cloned(),
        );
    }
    for p in &bundle.oracle_policy {
        values.extend(p.arguments.iter().cloned());
    }
    values
}

fn has_sort(term: &Term, sort: &Sort, bundle: &Bundle) -> bool {
    match (term, sort) {
        (Term::Integer { .. }, Sort::Integer) => true,
        (Term::Quantity { unit: a, .. }, Sort::Quantity { unit: b }) => a == b,
        (Term::Symbol { value }, expected) => bundle
            .constants
            .iter()
            .find(|c| c.id == *value)
            .map(|c| &c.sort == expected)
            .unwrap_or(*expected == Sort::Symbol),
        _ => false,
    }
}

fn run(bundle: &Bundle, options: &AnalysisOptions) -> Result<Analysis> {
    validate_context(bundle)?;
    if bundle.language != "oathkeeper/0.4" {
        return Err(Error("argument analysis requires oathkeeper/0.4".into()));
    }
    if options.max_groundings == 0
        || options.max_groundings > 50000
        || options.max_nodes == 0
        || options.max_nodes > 100000
        || options.max_depth == 0
        || options.max_depth > 96
        || options.max_arguments == 0
        || options.max_arguments > 128
        || options.max_sensitivity > 64
    {
        return Err(Error("[E_LIMIT] invalid analysis budget".into()));
    }
    let domain = terms(bundle);
    let mut search = Search {
        bundle,
        options,
        rules: Vec::new(),
        leaves: BTreeMap::new(),
        nodes: 0,
        limits: BTreeSet::new(),
        gaps: BTreeMap::new(),
    };
    let mut excluded_rules = Vec::new();
    for rule in &bundle.rules {
        if let Err(e) = rule_eligibility(bundle, rule) {
            excluded_rules.push(ExcludedRule {
                rule: rule.id.clone(),
                version: rule.version.clone(),
                reason: e.0,
            });
            continue;
        }
        let mut assignments = vec![BTreeMap::new()];
        for parameter in &rule.parameters {
            let mut next = Vec::new();
            for existing in &assignments {
                for term in domain
                    .iter()
                    .filter(|t| has_sort(t, &parameter.sort, bundle))
                {
                    if next.len() >= options.max_groundings {
                        search.limits.insert("groundings".into());
                        break;
                    }
                    let mut assignment = existing.clone();
                    assignment.insert(parameter.name.clone(), term.clone());
                    next.push(assignment);
                }
                if next.len() >= options.max_groundings {
                    break;
                }
            }
            assignments = next;
        }
        for substitution in assignments {
            if search.rules.len() >= options.max_groundings {
                search.limits.insert("groundings".into());
                break;
            }
            search.rules.push(GroundRule {
                rule: rule.clone(),
                premises: rule
                    .premises
                    .iter()
                    .map(|p| instantiate(p, &substitution))
                    .collect::<Result<_>>()?,
                conclusion: instantiate(&rule.conclusion, &substitution)?,
                substitution,
            });
        }
    }
    let cutoff = timestamp(&bundle.query.known_at)?;
    for fact in &bundle.facts {
        if !fact.claim.atom.starts_with("oathkeeper.")
            && timestamp(&fact.recorded_at)? <= cutoff
            && !crate::oracles::retracted(bundle, &EvidenceKind::Fact, &fact.id)?
        {
            search
                .leaves
                .entry(fact.claim.clone())
                .or_default()
                .push(step(
                    fact.claim.clone(),
                    Reason::Fact {
                        fact: fact.id.clone(),
                    },
                ));
        }
    }
    let oracle_requests: Vec<_> = bundle
        .oracle_policy
        .iter()
        .map(|p| crate::oracles::assess_request(bundle, p))
        .collect::<Result<_>>()?;
    for request in &oracle_requests {
        if let (Some(claim), Some(id)) = (&request.claim, request.accepted_responses.first()) {
            if !claim.atom.starts_with("oathkeeper.") {
                search.leaves.entry(claim.clone()).or_default().push(step(
                    claim.clone(),
                    Reason::Oracle {
                        attestation: id.clone(),
                    },
                ));
            }
        }
    }
    let supporting = search.checked(&bundle.query.goal)?;
    let opposing = search.checked(&signed_opposite(&bundle.query.goal))?;
    let status = match (supporting.is_empty(), opposing.is_empty()) {
        (false, true) => SupportStatus::Supported,
        (true, false) => SupportStatus::Opposed,
        (false, false) => SupportStatus::Both,
        (true, true) => SupportStatus::Neither,
    };
    let proved: BTreeSet<Claim> = supporting
        .iter()
        .chain(&opposing)
        .flat_map(|a| a.proof.iter().map(|s| s.claim.clone()))
        .collect();
    let open_requirements = search
        .gaps
        .iter()
        .filter(|(c, _)| !proved.contains(*c))
        .map(|(claim, rules)| RequirementGap {
            claim: claim.clone(),
            needed_by: rules.iter().cloned().collect(),
            oracle_requests: bundle
                .oracle_policy
                .iter()
                .filter(|p| p.atom == claim.atom && p.arguments == claim.arguments)
                .map(|p| p.oracle.clone())
                .collect(),
        })
        .collect();
    let mut context = serde_json::to_value(bundle).map_err(|e| Error(e.to_string()))?;
    context.as_object_mut().unwrap().remove("proof");
    context["query"].as_object_mut().unwrap().remove("root");
    Ok(Analysis {
        format:"oathkeeper-analysis/1".into(), kernel_version:env!("CARGO_PKG_VERSION").into(), context_sha256:hash(&context), status,
        goal:bundle.query.goal.clone(), supporting, opposing, open_requirements, excluded_rules, oracle_requests,
        search:SearchSummary { complete_for_ground_domain:search.limits.is_empty(), ground_terms:domain.into_iter().collect(), ground_rules:search.rules.len(), nodes_visited:search.nodes, limits_reached:search.limits.into_iter().collect(), meaning:"Search is relative to this finite supplied vocabulary and rule set. Failure to find a proof never establishes its negation or corpus completeness.".into() },
        sensitivity:Vec::new(), sensitivity_complete:!options.sensitivity,
        legal_conclusion:"not_determined; arguments are conditional on supplied formalization and evidence".into(),
    })
}

pub fn analyze(bundle: &Bundle, options: &AnalysisOptions) -> Result<Analysis> {
    let mut result = run(bundle, options)?;
    if !options.sensitivity {
        return Ok(result);
    }
    let mut dependencies = BTreeSet::new();
    for a in result.supporting.iter().chain(&result.opposing) {
        dependencies.extend(
            a.report
                .facts_used
                .iter()
                .map(|id| ("fact".to_owned(), id.clone())),
        );
        dependencies.extend(
            a.report
                .oracle_requests_used
                .iter()
                .map(|p| ("oracle_request".to_owned(), p.oracle.clone())),
        );
    }
    result.sensitivity_complete =
        dependencies.len() <= options.max_sensitivity && result.search.complete_for_ground_domain;
    for (kind, id) in dependencies.into_iter().take(options.max_sensitivity) {
        let mut hypothetical = bundle.clone();
        if kind == "fact" {
            hypothetical.facts.retain(|f| f.id != id);
            hypothetical
                .retractions
                .retain(|r| r.target_kind != EvidenceKind::Fact || r.target != id);
        } else {
            let removed: BTreeSet<_> = hypothetical
                .attestations
                .iter()
                .filter(|a| a.oracle == id)
                .map(|a| a.id.clone())
                .collect();
            hypothetical.attestations.retain(|a| a.oracle != id);
            hypothetical.retractions.retain(|r| {
                r.target_kind != EvidenceKind::Attestation || !removed.contains(&r.target)
            });
        }
        hypothetical.proof.clear();
        let without = run(&hypothetical, options)?;
        result.sensitivity_complete &= without.search.complete_for_ground_domain;
        result.sensitivity.push(Sensitivity {
            kind,
            id,
            without: without.status,
            search_complete: without.search.complete_for_ground_domain,
        });
    }
    Ok(result)
}
