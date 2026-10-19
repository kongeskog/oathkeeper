use crate::*;
use chrono::{DateTime, FixedOffset};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn require(ok: bool, message: impl Into<String>) -> Result<()> {
    if ok {
        Ok(())
    } else {
        Err(Error(message.into()))
    }
}

pub(crate) fn timestamp(value: &str) -> Result<DateTime<FixedOffset>> {
    DateTime::parse_from_rfc3339(value)
        .map_err(|_| Error(format!("invalid RFC3339 timestamp: {value}")))
}

pub(crate) fn identifier(value: &str) -> Result<()> {
    require(
        !value.is_empty()
            && value.len() <= 256
            && value
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || b"._:/-".contains(&c)),
        format!("invalid identifier: {value}"),
    )
}

fn claim(value: &Claim) -> Result<()> {
    identifier(&value.atom)?;
    for term in &value.arguments {
        match term {
            Term::Symbol { value } => identifier(value)?,
            Term::Variable { name } => identifier(name)?,
            Term::Integer { .. } => (),
            Term::Quantity { unit, .. } => identifier(unit)?,
        }
    }
    Ok(())
}

fn ground(value: &Claim) -> Result<()> {
    claim(value)?;
    require(
        !value
            .arguments
            .iter()
            .any(|t| matches!(t, Term::Variable { .. })),
        "proofs and evidence must contain ground terms",
    )
}

pub(crate) fn source(value: &Source) -> Result<()> {
    require(
        !value.citation.trim().is_empty() && !value.uri.trim().is_empty(),
        "source citation and URI are required",
    )?;
    hash(&value.sha256)
}

pub(crate) fn instantiate(value: &Claim, substitution: &BTreeMap<String, Term>) -> Result<Claim> {
    let mut result = value.clone();
    result.arguments = value
        .arguments
        .iter()
        .map(|term| match term {
            Term::Variable { name } => substitution
                .get(name)
                .cloned()
                .ok_or_else(|| Error(format!("unbound variable: {name}"))),
            _ => Ok(term.clone()),
        })
        .collect::<Result<Vec<_>>>()?;
    ground(&result)?;
    Ok(result)
}

fn substitution(rule: &Rule, values: &BTreeMap<String, Term>) -> Result<()> {
    require(
        rule.parameters.len() == values.len(),
        "substitution must bind exactly the declared parameters",
    )?;
    for parameter in &rule.parameters {
        let value = values
            .get(&parameter.name)
            .ok_or_else(|| Error(format!("missing substitution: {}", parameter.name)))?;
        require(
            matches!(
                (&parameter.sort, value),
                (Sort::Symbol | Sort::Domain(_), Term::Symbol { .. })
                    | (Sort::Integer, Term::Integer { .. })
                    | (Sort::Quantity { .. }, Term::Quantity { .. })
            ),
            "substitution sort mismatch or non-ground substitution",
        )?;
        if let Term::Symbol { value } = value {
            identifier(value)?;
        }
    }
    Ok(())
}

fn hash(value: &str) -> Result<()> {
    require(
        value.len() == 64
            && value
                .bytes()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()),
        "expected lowercase SHA-256 hex",
    )
}

fn probability(value: f64) -> Result<()> {
    require(
        value.is_finite() && (0.0..=1.0).contains(&value),
        "confidence must be finite and in [0, 1]",
    )
}

pub(crate) fn unique<'a, T>(
    items: &'a [T],
    key: impl Fn(&'a T) -> &'a str,
) -> Result<BTreeMap<&'a str, &'a T>> {
    require(
        items.len() <= 10000,
        "[E_LIMIT] declaration table too large",
    )?;
    let mut map = BTreeMap::new();
    for item in items {
        let id = key(item);
        identifier(id)?;
        require(
            map.insert(id, item).is_none(),
            format!("duplicate identifier: {id}"),
        )?;
    }
    Ok(map)
}

fn ancestors<'a>(
    id: &'a str,
    nodes: &BTreeMap<&'a str, &'a Jurisdiction>,
    visiting: &mut BTreeSet<&'a str>,
    done: &mut BTreeSet<&'a str>,
) -> Result<()> {
    require(
        !visiting.contains(id),
        format!("jurisdiction cycle at {id}"),
    )?;
    if done.contains(id) {
        return Ok(());
    }
    let node = nodes
        .get(id)
        .ok_or_else(|| Error(format!("unknown jurisdiction: {id}")))?;
    visiting.insert(id);
    for parent in &node.parents {
        ancestors(parent, nodes, visiting, done)?;
    }
    visiting.remove(id);
    done.insert(id);
    Ok(())
}

#[derive(Clone, Default)]
struct Dependencies {
    steps: BTreeSet<String>,
    facts: BTreeSet<String>,
    oracles: BTreeSet<String>,
    rules: BTreeSet<(String, String)>,
    assessments: BTreeSet<String>,
    authorities: BTreeSet<String>,
}

impl Dependencies {
    fn extend(&mut self, other: &Self) {
        self.steps.extend(other.steps.iter().cloned());
        self.facts.extend(other.facts.iter().cloned());
        self.oracles.extend(other.oracles.iter().cloned());
        self.rules.extend(other.rules.iter().cloned());
        self.assessments.extend(other.assessments.iter().cloned());
        self.authorities.extend(other.authorities.iter().cloned());
    }
}

/// Check all input structure, then every proof step, then the exact goal closure.
fn check(bundle: &Bundle, context_only: bool) -> Result<Option<Report>> {
    require(
        matches!(
            bundle.language.as_str(),
            "oathkeeper/0.1"
                | "oathkeeper/0.2"
                | "oathkeeper/0.3"
                | "oathkeeper/0.4"
                | "oathkeeper/0.5"
        ),
        "unsupported language version",
    )?;
    identifier(&bundle.package)?;
    identifier(&bundle.version)?;
    let legal_time = timestamp(&bundle.query.as_of)?;
    let cutoff = timestamp(&bundle.query.known_at)?;
    let extended = bundle.language != "oathkeeper/0.1";
    let modern = matches!(
        bundle.language.as_str(),
        "oathkeeper/0.4" | "oathkeeper/0.5"
    );
    let typed = matches!(
        bundle.language.as_str(),
        "oathkeeper/0.3" | "oathkeeper/0.4" | "oathkeeper/0.5"
    );
    require(
        bundle.rules.len() <= 2000
            && bundle.facts.len() <= 4000
            && bundle.proof.len() <= 2000
            && bundle.attestations.len() <= 500
            && bundle.retractions.len() <= 4000
            && bundle.jurisdictions.len() <= 128
            && bundle.types.len() <= 1024
            && bundle.predicates.len() <= 4000
            && bundle.constants.len() <= 10000
            && bundle.authorities.len() <= 2000
            && bundle.authority_assessments.len() <= 8000
            && bundle.oracle_policy.len() <= 500,
        "[E_LIMIT] kernel declaration budget exceeded",
    )?;
    let authority_time = timestamp(
        bundle
            .query
            .authority_at
            .as_deref()
            .unwrap_or(&bundle.query.as_of),
    )?;
    if modern {
        let event = bundle
            .query
            .event_at
            .as_deref()
            .ok_or_else(|| Error("0.4 query requires event_at".into()))?;
        require(
            timestamp(event).is_ok()
                || chrono::NaiveDate::parse_from_str(event, "%Y-%m-%d").is_ok(),
            "event_at requires an RFC3339 instant or an ISO calendar date",
        )?;
        require(
            bundle.query.authority_at.is_some()
                && bundle.query.purpose.is_some()
                && bundle
                    .query
                    .temporal_rationale
                    .as_ref()
                    .is_some_and(|r| !r.trim().is_empty()),
            "0.4 query requires authority_at, purpose, and temporal_rationale",
        )?;
    } else {
        require(
            bundle.query.event_at.is_none()
                && bundle.query.authority_at.is_none()
                && bundle.query.purpose.is_none()
                && bundle.query.temporal_rationale.is_none()
                && bundle.retractions.is_empty()
                && bundle.rules.iter().all(|r| r.time_basis.is_none())
                && bundle.oracle_policy.iter().all(|p| p.protocol.is_none())
                && bundle.attestations.iter().all(|a| a.disposition.is_none()),
            "0.4 features require oathkeeper/0.4",
        )?;
        require(
            bundle
                .predicates
                .iter()
                .flat_map(|p| &p.arguments)
                .all(|s| !matches!(s, Sort::Quantity { .. }))
                && bundle
                    .rules
                    .iter()
                    .flat_map(|r| &r.parameters)
                    .all(|p| !matches!(p.sort, Sort::Quantity { .. })),
            "quantities require oathkeeper/0.4",
        )?;
    }
    let schema = if typed {
        Some(crate::schema::Schema::new(
            &bundle.types,
            &bundle.predicates,
            &bundle.constants,
        )?)
    } else {
        require(
            bundle.types.is_empty()
                && bundle.predicates.is_empty()
                && bundle.constants.is_empty()
                && bundle
                    .rules
                    .iter()
                    .flat_map(|r| &r.parameters)
                    .all(|p| !matches!(p.sort, Sort::Domain(_))),
            "domain types and signatures require oathkeeper/0.3",
        )?;
        None
    };
    if let Some(schema) = &schema {
        let empty = BTreeMap::new();
        schema.claim(&bundle.query.goal, &empty)?;
        for rule in &bundle.rules {
            schema.rule(rule)?;
        }
        for fact in &bundle.facts {
            schema.claim(&fact.claim, &empty)?;
        }
        for attestation in &bundle.attestations {
            schema.claim(&attestation.claim, &empty)?;
        }
        for step in if context_only { &[][..] } else { &bundle.proof } {
            schema
                .claim(&step.claim, &empty)
                .map_err(|e| Error(format!("step {}: {e}", step.id)))?;
        }
        for policy in &bundle.oracle_policy {
            schema.claim(
                &Claim {
                    atom: policy.atom.clone(),
                    negative: false,
                    arguments: policy.arguments.clone(),
                },
                &empty,
            )?;
        }
    }
    if extended {
        identifier(
            bundle
                .query
                .forum
                .as_deref()
                .ok_or_else(|| Error("0.2 queries require a forum".into()))?,
        )?;
    } else {
        require(
            bundle.authorities.is_empty()
                && bundle.authority_assessments.is_empty()
                && bundle.query.forum.is_none(),
            "0.2 authority features require oathkeeper/0.2",
        )?;
        require(
            bundle.query.goal.arguments.is_empty()
                && bundle
                    .facts
                    .iter()
                    .all(|f| f.claim.arguments.is_empty() && f.basis.is_none())
                && bundle
                    .attestations
                    .iter()
                    .all(|a| a.claim.arguments.is_empty())
                && bundle.oracle_policy.iter().all(|p| p.arguments.is_empty()),
            "0.2 evidence features require oathkeeper/0.2",
        )?;
    }
    ground(&bundle.query.goal)?;
    identifier(&bundle.query.root)?;
    if !modern {
        for c in bundle
            .rules
            .iter()
            .flat_map(|r| r.premises.iter().chain(std::iter::once(&r.conclusion)))
            .chain(bundle.facts.iter().map(|f| &f.claim))
            .chain(bundle.attestations.iter().map(|a| &a.claim))
            .chain(std::iter::once(&bundle.query.goal))
        {
            require(
                !c.arguments
                    .iter()
                    .any(|t| matches!(t, Term::Quantity { .. })),
                "quantities require oathkeeper/0.4",
            )?;
        }
    }
    let jurisdictions = unique(&bundle.jurisdictions, |j| &j.id)?;
    // Validate disconnected components too: a package may not conceal a cycle.
    for id in jurisdictions.keys() {
        ancestors(
            id,
            &jurisdictions,
            &mut BTreeSet::new(),
            &mut BTreeSet::new(),
        )?;
    }
    let mut applicable = BTreeSet::new();
    ancestors(
        &bundle.query.jurisdiction,
        &jurisdictions,
        &mut BTreeSet::new(),
        &mut applicable,
    )?;
    let facts = unique(&bundle.facts, |f| &f.id)?;
    let attestations = unique(&bundle.attestations, |a| &a.id)?;
    if typed && !modern {
        // One frozen response per assessment obligation. Aggregation and resolving
        // disagreement happen before the host freezes the proof context.
        unique(&bundle.attestations, |a| &a.oracle).map_err(|_| {
            Error("[E_ORACLE_CONFLICT] multiple outputs for one frozen oracle request".into())
        })?;
    }
    let policies = unique(&bundle.oracle_policy, |p| &p.oracle)?;
    let authorities = unique(&bundle.authorities, |a| &a.id)?;
    let assessments = unique(&bundle.authority_assessments, |a| &a.id)?;
    for authority in &bundle.authorities {
        identifier(&authority.issuing_body)?;
        require(
            jurisdictions.contains_key(authority.jurisdiction.as_str()),
            "authority has unknown jurisdiction",
        )?;
        source(&authority.source)?;
        match authority.kind {
            AuthorityKind::Case => {
                timestamp(
                    authority
                        .decided_at
                        .as_deref()
                        .ok_or_else(|| Error("case authority requires decided_at".into()))?,
                )?;
                require(
                    authority.publication != Publication::Enacted,
                    "case cannot have enacted publication status",
                )?;
            }
            AuthorityKind::AdministrativeGuidance => require(
                modern && authority.publication == Publication::Published && authority.decided_at.is_none(),
                "administrative guidance requires language 0.4, published status, and no judicial decision timestamp",
            )?,
            _ => require(
                authority.publication == Publication::Enacted && authority.decided_at.is_none(),
                "legislation requires enacted status and no judicial decision timestamp",
            )?,
        }
    }
    let mut selected_assessments: BTreeMap<
        (&str, &str),
        (&AuthorityAssessment, DateTime<FixedOffset>, usize),
    > = BTreeMap::new();
    for assessment in &bundle.authority_assessments {
        require(
            authorities.contains_key(assessment.authority.as_str()),
            "assessment has unknown authority",
        )?;
        identifier(&assessment.rule)?;
        identifier(&assessment.rule_version)?;
        identifier(&assessment.forum)?;
        source(&assessment.source)?;
        require(
            !assessment.rationale.trim().is_empty(),
            "authority assessment requires a rationale",
        )?;
        let at = timestamp(&assessment.as_of)?;
        let recorded = timestamp(&assessment.recorded_at)?;
        if recorded <= cutoff
            && at == authority_time
            && Some(&assessment.forum) == bundle.query.forum.as_ref()
        {
            let key = (assessment.rule.as_str(), assessment.rule_version.as_str());
            if let Some((selected, prior, count)) = selected_assessments.get_mut(&key) {
                match recorded.cmp(prior) {
                    std::cmp::Ordering::Greater => {
                        *selected = assessment;
                        *prior = recorded;
                        *count = 1;
                    }
                    std::cmp::Ordering::Equal => *count += 1,
                    std::cmp::Ordering::Less => (),
                }
            } else {
                selected_assessments.insert(key, (assessment, recorded, 1));
            }
        }
    }
    require(
        selected_assessments
            .values()
            .all(|(_, _, count)| *count == 1),
        "ambiguous authority assessments at the latest recording time",
    )?;
    for fact in &bundle.facts {
        ground(&fact.claim)?;
        hash(&fact.evidence_sha256)?;
        timestamp(&fact.recorded_at)?;
        if extended {
            let basis = fact
                .basis
                .as_ref()
                .ok_or_else(|| Error("0.2 facts require an explicit provenance basis".into()))?;
            source(&basis.source)?;
            require(
                basis.source.sha256 == fact.evidence_sha256,
                "fact evidence digest does not match provenance source",
            )?;
            require(
                !basis.pinpoint.trim().is_empty() && !basis.explanation.trim().is_empty(),
                "fact provenance requires pinpoint and explanation",
            )?;
        }
    }
    for attestation in &bundle.attestations {
        identifier(&attestation.oracle)?;
        identifier(&attestation.provider)?;
        identifier(&attestation.provider_version)?;
        require(
            !attestation.question.trim().is_empty(),
            "empty oracle question",
        )?;
        ground(&attestation.claim)?;
        if let Some(confidence) = attestation.confidence {
            probability(confidence)?;
        }
        if !modern {
            require(
                attestation.confidence.is_some(),
                "legacy attestation requires confidence",
            )?;
        }
        hash(&attestation.evidence_sha256)?;
        timestamp(&attestation.recorded_at)?;
    }
    for policy in &bundle.oracle_policy {
        identifier(&policy.atom)?;
        ground(&Claim {
            atom: policy.atom.clone(),
            negative: false,
            arguments: policy.arguments.clone(),
        })?;
        hash(&policy.evidence_sha256)?;
        require(
            !policy.question.trim().is_empty(),
            "empty agreed oracle question",
        )?;
        if let Some(confidence) = policy.min_confidence {
            probability(confidence)?;
        }
        if !modern {
            require(
                policy.min_confidence.is_some(),
                "legacy policy requires min_confidence",
            )?;
        }
        require(
            !policy.providers.is_empty(),
            "oracle policy has no providers",
        )?;
        for provider in &policy.providers {
            identifier(&provider.id)?;
            identifier(&provider.version)?;
        }
    }
    let mut rules = BTreeMap::new();
    let mut active_ids = BTreeSet::new();
    let mut active = BTreeSet::new();
    let mut owners = BTreeMap::new();
    for rule in &bundle.rules {
        require(
            rule.premises.len() <= 64 && rule.parameters.len() <= 16,
            "[E_LIMIT] rule size budget exceeded",
        )?;
        if modern {
            require(
                rule.time_basis.is_some(),
                "0.4 rule requires explicit time_basis",
            )?;
        }
        identifier(&rule.id)?;
        identifier(&rule.version)?;
        require(
            jurisdictions.contains_key(rule.jurisdiction.as_str()),
            "rule has unknown jurisdiction",
        )?;
        if let Some(owner) = owners.insert(&rule.id, &rule.jurisdiction) {
            require(
                owner == &rule.jurisdiction,
                "rule identity cannot change jurisdiction",
            )?;
        }
        require(
            rules
                .insert((rule.id.as_str(), rule.version.as_str()), rule)
                .is_none(),
            "duplicate rule version",
        )?;
        require(
            !rule.source.citation.trim().is_empty() && !rule.source.uri.trim().is_empty(),
            "rule source is required",
        )?;
        hash(&rule.source.sha256)?;
        claim(&rule.conclusion)?;
        for premise in &rule.premises {
            claim(premise)?;
        }
        let parameters = unique(&rule.parameters, |p| &p.name)?;
        for value in rule
            .premises
            .iter()
            .chain(std::iter::once(&rule.conclusion))
        {
            for term in &value.arguments {
                if let Term::Variable { name } = term {
                    require(
                        parameters.contains_key(name.as_str()),
                        "undeclared rule variable",
                    )?;
                }
            }
        }
        if !extended {
            require(
                rule.parameters.is_empty()
                    && rule.authority.is_none()
                    && rule
                        .premises
                        .iter()
                        .chain(std::iter::once(&rule.conclusion))
                        .all(|c| c.arguments.is_empty()),
                "parameterized rules require oathkeeper/0.2",
            )?;
        } else {
            let usage = rule
                .authority
                .as_ref()
                .ok_or_else(|| Error("0.2 rules require an authority reference".into()))?;
            let authority = authorities
                .get(usage.authority.as_str())
                .ok_or_else(|| Error("rule has unknown authority".into()))?;
            require(
                !usage.pinpoint.trim().is_empty(),
                "authority use requires a pinpoint",
            )?;
            require(
                rule.premises.contains(&usage.scope),
                "authority scope must be an explicit rule premise",
            )?;
            require(
                rule.source.sha256 == authority.source.sha256
                    && rule.source.uri == authority.source.uri,
                "rule source does not match authority source",
            )?;
            require(
                rule.jurisdiction == authority.jurisdiction,
                "rule and authority jurisdictions must match",
            )?;
            require(
                matches!(
                    (&authority.kind, &usage.role),
                    (AuthorityKind::Case, AuthorityRole::Holding)
                ) || (authority.kind != AuthorityKind::Case && usage.role == AuthorityRole::Text),
                "authority role does not match authority kind",
            )?;
        }
        let start = timestamp(&rule.effective_from)?;
        let end = rule.effective_until.as_deref().map(timestamp).transpose()?;
        if let Some(end) = end {
            require(end > start, "empty or inverted rule validity interval")?;
        }
        let recorded = timestamp(&rule.recorded_at)?;
        let rule_time = if rule.time_basis == Some(TimeBasis::AuthorityTime) {
            authority_time
        } else {
            legal_time
        };
        if applicable.contains(rule.jurisdiction.as_str())
            && start <= rule_time
            && end.is_none_or(|end| rule_time < end)
            && recorded <= cutoff
        {
            require(
                active_ids.insert(&rule.id),
                format!("ambiguous active versions of rule {}", rule.id),
            )?;
            active.insert((rule.id.as_str(), rule.version.as_str()));
        }
    }
    for assessment in &bundle.authority_assessments {
        let rule = rules
            .get(&(assessment.rule.as_str(), assessment.rule_version.as_str()))
            .ok_or_else(|| Error("assessment has unknown rule version".into()))?;
        require(
            rule.authority
                .as_ref()
                .is_some_and(|u| u.authority == assessment.authority),
            "assessment authority does not match rule",
        )?;
    }
    crate::oracles::validate_evidence_protocols(bundle)?;
    crate::contracts::validate(bundle)?;
    if context_only {
        return Ok(None);
    }
    let mut checked: BTreeMap<&str, (&Claim, Dependencies)> = BTreeMap::new();
    let mut claims = BTreeSet::new();
    for step in &bundle.proof {
        (|| -> Result<()> {
            identifier(&step.id)?;
            ground(&step.claim)?;
            if !extended {
                require(
                    step.claim.arguments.is_empty(),
                    "claim arguments require oathkeeper/0.2",
                )?;
            }
            if extended && step.claim.atom.starts_with("oathkeeper.") {
                require(
                    matches!(step.reason, Reason::Compare { .. } | Reason::Compute { .. }),
                    "built-in predicates can only be established by checked comparisons or arithmetic",
                )?;
            }
            require(
                !checked.contains_key(step.id.as_str()),
                "duplicate proof step",
            )?;
            let mut opposite = step.claim.clone();
            opposite.negative = !opposite.negative;
            require(
                !claims.contains(&opposite),
                format!("conflicting proof claims: {}", step.claim.atom),
            )?;
            let mut dependencies = Dependencies::default();
            match &step.reason {
                Reason::Compute { operator } => {
                    require(modern, "checked quantity arithmetic requires oathkeeper/0.4 and kernel 0.4.1")?;
                    crate::arithmetic::compute(&step.claim, operator)?;
                }
                Reason::Compare { operator } => {
                    require(extended, "integer comparisons require oathkeeper/0.2")?;
                    if !modern {
                        require(
                            !matches!(
                                operator,
                                Comparison::QuantityGreaterThan
                                    | Comparison::QuantityLessThan
                                    | Comparison::QuantityEqual
                            ),
                            "quantity comparisons require oathkeeper/0.4",
                        )?;
                    }
                    compare(&step.claim, operator)?;
                }
                Reason::Fact { fact } => {
                    let fact = facts.get(fact.as_str()).ok_or_else(|| {
                        Error(format!("[E_MISSING_EVIDENCE] unknown fact: {fact}"))
                    })?;
                    require(fact.claim == step.claim, "fact claim mismatch")?;
                    require(
                        !crate::oracles::retracted(bundle, &EvidenceKind::Fact, &fact.id)?,
                        "[E_RETRACTED_EVIDENCE] fact was withdrawn at the knowledge cutoff",
                    )?;
                    require(
                        timestamp(&fact.recorded_at)? <= cutoff,
                        "fact recorded after knowledge cutoff",
                    )?;
                    dependencies.facts.insert(fact.id.clone());
                }
                Reason::Oracle { attestation } => {
                    let a = attestations.get(attestation.as_str()).ok_or_else(|| {
                        Error(format!(
                            "[E_MISSING_EVIDENCE] unknown attestation: {attestation}"
                        ))
                    })?;
                    let policy = policies
                        .get(a.oracle.as_str())
                        .ok_or_else(|| Error(format!("no agreement for oracle: {}", a.oracle)))?;
                    require(a.claim == step.claim, "oracle claim mismatch")?;
                    require(
                        timestamp(&a.recorded_at)? <= cutoff,
                        "oracle recorded after knowledge cutoff",
                    )?;
                    require(
                        policy
                            .providers
                            .iter()
                            .any(|p| p.id == a.provider && p.version == a.provider_version),
                        "oracle provider or version not agreed",
                    )?;
                    require(
                        a.claim.atom == policy.atom
                            && a.question == policy.question
                            && a.claim.arguments == policy.arguments
                            && a.evidence_sha256 == policy.evidence_sha256,
                        "oracle output does not match agreed request",
                    )?;
                    if modern {
                        let accepted = crate::oracles::accepted_responses(bundle, policy)?;
                        require(
                            accepted.iter().any(|id| id == &a.id),
                            "[E_ORACLE_UNRESOLVED] oracle protocol has no agreed assertion",
                        )?;
                        dependencies.oracles.extend(accepted);
                    } else {
                        require(
                            a.confidence
                                .zip(policy.min_confidence)
                                .is_some_and(|(a, p)| a >= p),
                            "oracle confidence below agreed threshold",
                        )?;
                        dependencies.oracles.insert(a.id.clone());
                    }
                }
                Reason::Apply {
                    rule,
                    version,
                    premises,
                    substitution: values,
                } => {
                    let key = (rule.as_str(), version.as_str());
                    let rule = rules
                        .get(&key)
                        .ok_or_else(|| Error(format!("unknown rule version: {rule}@{version}")))?;
                    require(
                        active.contains(&key),
                        "rule is outside jurisdiction, legal time, or knowledge cutoff",
                    )?;
                    crate::contracts::interpretation_eligible(bundle, rule)?;
                    substitution(rule, values)?;
                    if let Some(schema) = &schema {
                        schema.substitution(rule, values)?;
                    }
                    require(
                        instantiate(&rule.conclusion, values)? == step.claim,
                        "rule conclusion mismatch",
                    )?;
                    if let Some(usage) = &rule.authority {
                        let authority = authorities[usage.authority.as_str()];
                        let (assessment, _, _) =
                            selected_assessments.get(&key).ok_or_else(|| {
                                Error(
                                    "no authority assessment for this rule, forum, and legal time"
                                        .into(),
                                )
                            })?;
                        require(
                            assessment.treatment == Treatment::Operative,
                            "authority is not operative under the latest visible assessment",
                        )?;
                        require(
                            modern
                                || authority.publication != Publication::Unpublished
                                || assessment.force != Force::Binding,
                            "unpublished case cannot be treated as binding",
                        )?;
                        if let Some(decided) = &authority.decided_at {
                            require(
                                timestamp(decided)? <= authority_time
                                    && timestamp(decided)? <= cutoff,
                                "precedent was not yet decided at the query time",
                            )?;
                        }
                        dependencies.assessments.insert(assessment.id.clone());
                        dependencies.authorities.insert(authority.id.clone());
                    }
                    require(
                        premises.len() == rule.required_claims().len(),
                        "rule premise count mismatch",
                    )?;
                    for (reference, expected) in premises.iter().zip(&rule.required_claims()) {
                        let (actual, prior) = checked.get(reference.as_str()).ok_or_else(|| {
                            Error(format!(
                                "premise must reference an earlier step: {reference}"
                            ))
                        })?;
                        require(
                            **actual == instantiate(expected, values)?,
                            "rule premise claim mismatch",
                        )?;
                        dependencies.extend(prior);
                    }
                    dependencies
                        .rules
                        .insert((rule.id.clone(), rule.version.clone()));
                }
            }
            dependencies.steps.insert(step.id.clone());
            claims.insert(step.claim.clone());
            checked.insert(&step.id, (&step.claim, dependencies));
            Ok(())
        })()
        .map_err(|e| Error(format!("step {}: {e}", step.id)))?;
    }
    let (root_claim, dependencies) = checked.get(bundle.query.root.as_str()).ok_or_else(|| {
        Error(
            "[E_NO_PROOF] missing proof root; query remains unknown without a supplied derivation"
                .into(),
        )
    })?;
    require(
        **root_claim == bundle.query.goal,
        "proof root does not establish query goal",
    )?;
    require(
        dependencies.steps.len() == bundle.proof.len(),
        "proof contains steps outside the goal dependency closure",
    )?;
    let encoded = serde_json::to_vec(bundle).map_err(|e| Error(e.to_string()))?;
    let oracle_count = dependencies.oracles.len();
    Ok(Some(Report {
        language: bundle.language.clone(),
        kernel_version: env!("CARGO_PKG_VERSION").into(),
        package: bundle.package.clone(),
        package_version: bundle.version.clone(),
        bundle_sha256: format!("{:x}", Sha256::digest(encoded)),
        status: "derivation_verified".into(),
        goal: bundle.query.goal.clone(),
        jurisdiction: bundle.query.jurisdiction.clone(),
        as_of: bundle.query.as_of.clone(),
        known_at: bundle.query.known_at.clone(),
        applicable_jurisdictions: applicable.into_iter().map(String::from).collect(),
        rules_used: dependencies.rules.iter().cloned().collect(),
        facts_used: dependencies.facts.iter().cloned().collect(),
        oracle_assumptions: dependencies.oracles.iter().cloned().collect(),
        oracle_count,
        assurance: Assurance {
            derivation: "checked_under_supplied_assumptions".into(),
            legal_fidelity: "not_established_by_kernel".into(),
            corpus_completeness: "not_established".into(),
            evidence_authenticity: "not_authenticated".into(),
            oracle_authentication: "not_authenticated".into(),
            counterargument_search: "not_performed_by_verify".into(),
            judicial_acceptance: "not_determined".into(),
            authority_reviews_reported: dependencies.assessments.iter().filter(|id| assessments[id.as_str()].review_status == ReviewStatus::Reviewed).count(),
            interpretation_assumptions: count_kind(dependencies, &facts, FactKind::InterpretationAssumption),
            party_allegations: count_kind(dependencies, &facts, FactKind::PartyAllegation),
            stipulated_facts: count_kind(dependencies, &facts, FactKind::StipulatedFact),
            limitations: vec!["All derivations are conditional on the supplied formalization and evidence.".into(), "Review labels and provider identities are supplied assertions, not authenticated credentials.".into(), "A successful derivation does not exclude a counterargument or establish a remedy.".into()],
        },
        authorities_used: dependencies
            .authorities
            .iter()
            .map(|id| authorities[id.as_str()].clone())
            .collect(),
        authority_assessments_used: dependencies
            .assessments
            .iter()
            .map(|id| assessments[id.as_str()].clone())
            .collect(),
        fact_assumptions: dependencies
            .facts
            .iter()
            .map(|id| facts[id.as_str()].clone())
            .collect(),
        unreviewed_authority_assessment_count: dependencies
            .assessments
            .iter()
            .filter(|id| {
                assessments[id.as_str()].review_status == ReviewStatus::MachineAssistedUnreviewed
            })
            .count(),
        scope_assumption_count: dependencies
            .facts
            .iter()
            .filter(|id| {
                facts[id.as_str()]
                    .basis
                    .as_ref()
                    .is_some_and(|b| b.kind == FactKind::ScopeAssumption)
            })
            .count(),
        record_fact_count: dependencies
            .facts
            .iter()
            .filter(|id| {
                facts[id.as_str()]
                    .basis
                    .as_ref()
                    .is_some_and(|b| b.kind == FactKind::RecordFact)
            })
            .count(),
        judicial_finding_count: dependencies
            .facts
            .iter()
            .filter(|id| {
                facts[id.as_str()]
                    .basis
                    .as_ref()
                    .is_some_and(|b| b.kind == FactKind::JudicialFinding)
            })
            .count(),
        oracle_requests_used: dependencies
            .oracles
            .iter()
            .map(|id| attestations[id.as_str()].oracle.as_str())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .map(|id| policies[id].clone())
            .collect(),
        event_at: bundle.query.event_at.clone(),
        authority_at: bundle.query.authority_at.clone().unwrap_or_else(|| bundle.query.as_of.clone()),
        purpose: bundle.query.purpose.clone(),
    }))
}

fn count_kind(deps: &Dependencies, facts: &BTreeMap<&str, &Fact>, kind: FactKind) -> usize {
    deps.facts
        .iter()
        .filter(|id| {
            facts[id.as_str()]
                .basis
                .as_ref()
                .is_some_and(|b| b.kind == kind)
        })
        .count()
}

pub fn validate_context(bundle: &Bundle) -> Result<()> {
    check(bundle, true).map(|_| ())
}

pub fn verify(bundle: &Bundle) -> Result<Report> {
    check(bundle, false)?.ok_or_else(|| Error("missing verification report".into()))
}

pub(crate) fn compare(claim: &Claim, operator: &Comparison) -> Result<()> {
    let quantity = matches!(
        operator,
        Comparison::QuantityGreaterThan | Comparison::QuantityLessThan | Comparison::QuantityEqual
    );
    let (left, right) = match claim.arguments.as_slice() {
        [Term::Integer { value: l }, Term::Integer { value: r }] if !quantity => (*l, *r),
        [Term::Quantity { value: l, unit: lu }, Term::Quantity { value: r, unit: ru }]
            if quantity && lu == ru =>
        {
            (*l, *r)
        }
        _ => {
            return Err(Error(
                "comparison requires two numbers of the same kind and unit".into(),
            ))
        }
    };
    let (atom, result) = match operator {
        Comparison::GreaterThan => ("oathkeeper.integer.gt", left > right),
        Comparison::LessThan => ("oathkeeper.integer.lt", left < right),
        Comparison::Equal => ("oathkeeper.integer.eq", left == right),
        Comparison::QuantityGreaterThan => ("oathkeeper.quantity.gt", left > right),
        Comparison::QuantityLessThan => ("oathkeeper.quantity.lt", left < right),
        Comparison::QuantityEqual => ("oathkeeper.quantity.eq", left == right),
    };
    require(
        claim.atom == atom && claim.negative != result,
        "comparison is false or uses the wrong predicate",
    )
}

/// Shared eligibility gate used by search. Verification still checks every application.
pub(crate) fn rule_eligibility(bundle: &Bundle, rule: &Rule) -> Result<()> {
    crate::contracts::interpretation_eligible(bundle, rule)?;
    let modern = matches!(
        bundle.language.as_str(),
        "oathkeeper/0.4" | "oathkeeper/0.5"
    );
    let cutoff = timestamp(&bundle.query.known_at)?;
    let authority_time = timestamp(
        bundle
            .query
            .authority_at
            .as_deref()
            .unwrap_or(&bundle.query.as_of),
    )?;
    let rule_time = if rule.time_basis == Some(TimeBasis::AuthorityTime) {
        authority_time
    } else {
        timestamp(&bundle.query.as_of)?
    };
    let nodes = unique(&bundle.jurisdictions, |j| &j.id)?;
    let mut applicable = BTreeSet::new();
    ancestors(
        &bundle.query.jurisdiction,
        &nodes,
        &mut BTreeSet::new(),
        &mut applicable,
    )?;
    require(
        applicable.contains(rule.jurisdiction.as_str()),
        "outside applicable jurisdiction graph",
    )?;
    require(
        timestamp(&rule.recorded_at)? <= cutoff,
        "rule recorded after knowledge cutoff",
    )?;
    require(
        timestamp(&rule.effective_from)? <= rule_time
            && rule
                .effective_until
                .as_deref()
                .map(timestamp)
                .transpose()?
                .is_none_or(|end| rule_time < end),
        "outside rule effective interval",
    )?;
    if let Some(usage) = &rule.authority {
        let authority = bundle
            .authorities
            .iter()
            .find(|a| a.id == usage.authority)
            .ok_or_else(|| Error("unknown authority".into()))?;
        if let Some(decided) = &authority.decided_at {
            require(
                timestamp(decided)? <= authority_time && timestamp(decided)? <= cutoff,
                "precedent unavailable at authority time or knowledge cutoff",
            )?;
        }
        let mut selected: Option<(&AuthorityAssessment, DateTime<FixedOffset>)> = None;
        for a in &bundle.authority_assessments {
            let recorded = timestamp(&a.recorded_at)?;
            if a.rule == rule.id
                && a.rule_version == rule.version
                && Some(&a.forum) == bundle.query.forum.as_ref()
                && timestamp(&a.as_of)? == authority_time
                && recorded <= cutoff
                && selected.as_ref().is_none_or(|(_, t)| recorded > *t)
            {
                selected = Some((a, recorded));
            }
        }
        let a = selected
            .ok_or_else(|| Error("no assessment for this rule, forum, and authority time".into()))?
            .0;
        require(
            a.treatment == Treatment::Operative,
            "authority is not operative under selected assessment",
        )?;
        require(
            modern
                || authority.publication != Publication::Unpublished
                || a.force != Force::Binding,
            "legacy policy disallows unpublished binding authority",
        )?;
    }
    Ok(())
}
