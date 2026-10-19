//! Bounded, proof-producing 2025 federal IRA allocation pilot.
//! This builder is untrusted: every generated certificate is replayed by the kernel.
use crate::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub const MODULE_SOURCE: &str = include_str!("../../../lawlib/us/tax/backdoor-roth-2025.oath");
pub const SOURCE_RECORDED_AT: &str = "2026-10-18T23:21:21Z";
pub const ROUNDING: &str = "Exact capped pro-rata fraction; round the allocated amount once to cents, ties up. This is an allocation estimate, not a completed Form 8606. IRS decimal-ratio and filing rounding may differ.";
pub const SCOPE: &str = "2025 U.S. federal calendar-year, own ordinary traditional IRA conversion; timely, valid, nondeductible regular contribution. Own taxable compensation only. Other regular IRA contributions are Roth contributions already independently determined eligible, not deductible traditional contributions. All own traditional, traditional SEP and traditional SIMPLE IRA year-end balances across custodians are aggregated; spouse IRAs and employer plans are excluded. No other IRA distributions, tax withholding from the conversion, RMD obligation, outstanding rollovers, rollovers into or out of traditional IRAs, special repayments, qualified charitable/HSA distributions, recharacterizations, excess corrections, inherited IRAs, SIMPLE conversions, annuity valuation adjustments, disaster relief, nonresident/treaty or community-property issues. Following-year contributions were designated for 2025 and made by April 15, 2026. Basis and account values are supplied assumptions, not authenticated records.";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BackdoorRothInput {
    pub tax_year: u16,
    pub conversion_year: u16,
    pub recorded_at: String,
    pub purpose: Option<CasePurpose>,
    pub scope_confirmed: Option<bool>,
    pub age_at_year_end: Option<i64>,
    pub taxable_compensation_cents: Option<i64>,
    pub nondeductible_contribution_cents: Option<i64>,
    pub contribution_made_in_2026_cents: Option<i64>,
    pub other_regular_ira_contributions_cents: Option<i64>,
    pub prior_basis_cents: Option<i64>,
    pub dec31_ira_value_cents: Option<i64>,
    pub roth_conversion_cents: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxIssue {
    pub field: String,
    pub message: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Allocation {
    pub annual_limit_cents: i64,
    pub total_basis_cents: i64,
    pub available_basis_cents: i64,
    pub aggregate_pool_cents: i64,
    pub fraction_numerator_cents: i64,
    pub fraction_denominator_cents: i64,
    pub nontaxable_conversion_cents: i64,
    pub taxable_conversion_cents: i64,
    pub remaining_basis_cents: i64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxAssessment {
    pub status: String,
    pub module_snapshot_sha256: String,
    pub module_version: String,
    pub scope: String,
    pub rounding: String,
    pub review_status: String,
    pub issues: Vec<TaxIssue>,
    pub allocation: Option<Allocation>,
    pub certificate: Option<Bundle>,
    pub verification: Option<Report>,
}
fn issue(field: &str, message: &str) -> TaxIssue {
    TaxIssue {
        field: field.into(),
        message: message.into(),
    }
}
fn quantity(value: i64) -> Term {
    Term::Quantity {
        value,
        unit: "usd_cent".into(),
    }
}
fn person() -> Term {
    Term::Symbol {
        value: "taxpayer.scenario".into(),
    }
}
fn claim(atom: &str, mut args: Vec<Term>) -> Claim {
    args.insert(0, person());
    Claim {
        atom: format!("tax.{atom}"),
        negative: false,
        arguments: args,
    }
}
fn instantiate(claim: &Claim, bindings: &BTreeMap<String, Term>) -> Claim {
    let mut c = claim.clone();
    c.arguments = c
        .arguments
        .into_iter()
        .map(|t| match t {
            Term::Variable { name } => bindings[&name].clone(),
            _ => t,
        })
        .collect();
    c
}
fn apply(bundle: &mut Bundle, name: &str, values: &[(&str, Term)]) -> Result<()> {
    let id = format!("us.tax.2025.{name}");
    let rule = bundle
        .rules
        .iter()
        .find(|r| r.id == id)
        .ok_or_else(|| Error("missing tax rule".into()))?
        .clone();
    let mut bindings: BTreeMap<String, Term> = values
        .iter()
        .map(|(k, v)| ((*k).into(), v.clone()))
        .collect();
    bindings.insert("person".into(), person());
    let mut premises = Vec::new();
    for p in &rule.premises {
        let c = instantiate(p, &bindings);
        if let Some(s) = bundle.proof.iter().find(|s| s.claim == c) {
            premises.push(s.id.clone());
            continue;
        }
        let reason = if let Some(op) = crate::arithmetic::operator(&c) {
            Reason::Compute { operator: op }
        } else {
            let op = match c.atom.as_str() {
                "oathkeeper.integer.lt" => Comparison::LessThan,
                "oathkeeper.quantity.gt" => Comparison::QuantityGreaterThan,
                "oathkeeper.quantity.lt" => Comparison::QuantityLessThan,
                _ => return Err(Error(format!("tax builder lacks premise {}", c.atom))),
            };
            Reason::Compare { operator: op }
        };
        let id = format!("arithmetic.{}", bundle.proof.len());
        bundle.proof.push(Step {
            id: id.clone(),
            claim: c,
            reason,
        });
        premises.push(id);
    }
    bundle.proof.push(Step {
        id: name.into(),
        claim: instantiate(&rule.conclusion, &bindings),
        reason: Reason::Apply {
            rule: id,
            version: rule.version,
            premises,
            substitution: bindings,
        },
    });
    Ok(())
}

/// Missing values stay unknown. Out-of-scope scenarios produce no certificate or amount.
pub fn assess(input: &BackdoorRothInput) -> Result<TaxAssessment> {
    if input.purpose == Some(CasePurpose::HistoricalReconstruction) {
        return Err(Error(
            "Use the separately sourced historical case module for an adjudicated reconstruction"
                .into(),
        ));
    }
    let compiled = compile_text(MODULE_SOURCE)?;
    let mut output = TaxAssessment {
        status: "needs_information".into(),
        module_snapshot_sha256: compiled.snapshot_sha256,
        module_version: compiled.module.version.clone(),
        scope: SCOPE.into(),
        rounding: ROUNDING.into(),
        review_status: "independent_legal_and_formalization_review_pending".into(),
        issues: vec![],
        allocation: None,
        certificate: None,
        verification: None,
    };
    let recorded = chrono::DateTime::parse_from_rfc3339(&input.recorded_at)
        .map_err(|_| Error("invalid recorded_at timestamp".into()))?;
    let source_time =
        chrono::DateTime::parse_from_rfc3339(SOURCE_RECORDED_AT).expect("source timestamp");
    if recorded < source_time {
        return Err(Error("recorded_at precedes this source snapshot; historical knowledge claims are unsupported".into()));
    }
    if input.tax_year != 2025 || input.conversion_year != 2025 {
        output.status = "outside_scope".into();
        output.issues.push(issue("tax_year", "Only tax year 2025 and conversions completed in calendar 2025 are supported. A 2025 contribution made in 2026 does not make a 2026 conversion a 2025 conversion."));
        return Ok(output);
    }
    if input.scope_confirmed == Some(false) {
        output.status = "outside_scope".into();
        output.issues.push(issue(
            "scope_confirmed",
            "Scenario does not meet the stated scope; no allocation was calculated.",
        ));
        return Ok(output);
    }
    if input.scope_confirmed.is_none() {
        output.issues.push(issue(
            "scope_confirmed",
            "Confirm every stated scope condition, or leave this scenario unresolved.",
        ));
    }
    let fields = [
        ("age_at_year_end", input.age_at_year_end),
        (
            "taxable_compensation_cents",
            input.taxable_compensation_cents,
        ),
        (
            "nondeductible_contribution_cents",
            input.nondeductible_contribution_cents,
        ),
        (
            "contribution_made_in_2026_cents",
            input.contribution_made_in_2026_cents,
        ),
        (
            "other_regular_ira_contributions_cents",
            input.other_regular_ira_contributions_cents,
        ),
        ("prior_basis_cents", input.prior_basis_cents),
        ("dec31_ira_value_cents", input.dec31_ira_value_cents),
        ("roth_conversion_cents", input.roth_conversion_cents),
    ];
    for (field, value) in fields {
        match value {
            None => output.issues.push(issue(field, "Required input is unknown. Enter zero only when established by the scenario or records.")),
            Some(v) if !(0..=1_000_000_000_000).contains(&v) => return Err(Error(format!("{field} must be a nonnegative integer, at most 1000000000000"))),
            _ => (),
        }
    }
    if input.age_at_year_end.is_some_and(|v| v > 120) {
        return Err(Error("age_at_year_end must be between 0 and 120".into()));
    }
    if !output.issues.is_empty() {
        return Ok(output);
    }
    let age = input.age_at_year_end.unwrap();
    let compensation = input.taxable_compensation_cents.unwrap();
    let contribution = input.nondeductible_contribution_cents.unwrap();
    let late = input.contribution_made_in_2026_cents.unwrap();
    let other = input.other_regular_ira_contributions_cents.unwrap();
    let prior = input.prior_basis_cents.unwrap();
    let balance = input.dec31_ira_value_cents.unwrap();
    let converted = input.roth_conversion_cents.unwrap();
    let limit = if age >= 50 { 800_000 } else { 700_000 };
    if late > contribution {
        return Err(Error(
            "following-year contribution cannot exceed total 2025 contribution".into(),
        ));
    }
    if converted == 0 {
        output.issues.push(issue(
            "roth_conversion_cents",
            "This pilot requires a positive 2025 Roth conversion.",
        ));
    }
    if contribution + other > limit {
        output.issues.push(issue("nondeductible_contribution_cents", "Combined regular IRA contributions exceed the age-based 2025 limit; excess-contribution treatment is outside scope."));
    }
    if contribution + other > compensation {
        output.issues.push(issue("taxable_compensation_cents", "Combined regular IRA contributions exceed own taxable compensation; spousal compensation and excess treatment are outside scope."));
    }
    if !output.issues.is_empty() {
        output.status = "outside_scope".into();
        return Ok(output);
    }
    let total = contribution + prior;
    let available = total - late;
    let pool = balance + converted;
    let nontaxable = crate::arithmetic::allocate(converted, available, pool)?;
    let taxable = converted - nontaxable;
    let remaining = total - nontaxable;
    let input_hash = format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(input).map_err(|e| Error(e.to_string()))?)
    );
    let source = Source {
        citation: "Caller-supplied scenario; not authenticated tax records".into(),
        uri: format!("urn:oathkeeper:tax-input:{input_hash}"),
        sha256: input_hash.clone(),
    };
    let observations = [
        ("scope2025", None),
        ("age", Some(Term::Integer { value: age })),
        ("compensation", Some(quantity(compensation))),
        ("contribution", Some(quantity(contribution))),
        ("following_year_contribution", Some(quantity(late))),
        ("other_contributions", Some(quantity(other))),
        ("prior_basis", Some(quantity(prior))),
        ("dec31_balance", Some(quantity(balance))),
        ("converted", Some(quantity(converted))),
    ];
    let facts: Vec<Fact> = observations.iter().map(|(name, value)| Fact {
        id: format!("input.{name}"), claim: claim(name, value.iter().cloned().collect()),
        evidence_sha256: input_hash.clone(), recorded_at: input.recorded_at.clone(),
        basis: Some(FactBasis { kind: if *name == "scope2025" { FactKind::ScopeAssumption } else { FactKind::PartyAllegation }, source: source.clone(), pinpoint: format!("Input field {name}"), explanation: if *name == "scope2025" { SCOPE.into() } else { "Caller-supplied amount or age. Correctness, completeness, legal eligibility, and authenticity are not established by arithmetic verification.".into() } }),
    }).collect();
    let proof = facts
        .iter()
        .map(|f| Step {
            id: f.id.clone(),
            claim: f.claim.clone(),
            reason: Reason::Fact { fact: f.id.clone() },
        })
        .collect();
    let case = CaseFile {
        constants: vec![Constant { id: "taxpayer.scenario".into(), sort: Sort::Domain("Taxpayer".into()) }], facts, proof,
        attestations: vec![], oracle_policy: vec![], retractions: vec![],
        query: Query { jurisdiction: "us".into(), as_of: "2025-12-31T00:00:00Z".into(), known_at: input.recorded_at.clone(),
            goal: claim("allocation_estimate", vec![quantity(nontaxable), quantity(taxable), quantity(remaining)]), root: "allocation-summary".into(),
            forum: Some("us.federal.tax".into()), event_at: Some("2025-12-31".into()), authority_at: Some("2025-12-31T00:00:00Z".into()),
            purpose: Some(input.purpose.clone().unwrap_or(CasePurpose::NewMatterAssessment)), temporal_rationale: Some("Calendar-year 2025 aggregate calculation under pinned final 2025 IRS guidance, reconstructed at the supplied knowledge cutoff. December 31 is the aggregation date, not an asserted transaction date. Contribution designation and conversion calendar year are separate inputs.".into()),
        },
    };
    let mut bundle = compiled.module.with_case(case);
    apply(
        &mut bundle,
        if age >= 50 {
            "limit-50plus"
        } else {
            "limit-under50"
        },
        &[("age", Term::Integer { value: age })],
    )?;
    apply(
        &mut bundle,
        "contribution-limit",
        &[
            ("contribution", quantity(contribution)),
            ("other", quantity(other)),
            ("total", quantity(contribution + other)),
            ("limit", quantity(limit)),
            ("compensation", quantity(compensation)),
        ],
    )?;
    apply(
        &mut bundle,
        "total-basis",
        &[
            ("contribution", quantity(contribution)),
            ("prior", quantity(prior)),
            ("total", quantity(total)),
        ],
    )?;
    apply(
        &mut bundle,
        "available-basis",
        &[
            ("contribution", quantity(contribution)),
            ("total", quantity(total)),
            ("late", quantity(late)),
            ("available", quantity(available)),
        ],
    )?;
    apply(
        &mut bundle,
        "aggregate-pool",
        &[
            ("balance", quantity(balance)),
            ("converted", quantity(converted)),
            ("pool", quantity(pool)),
        ],
    )?;
    apply(
        &mut bundle,
        "nontaxable-allocation",
        &[
            ("converted", quantity(converted)),
            ("basis", quantity(available)),
            ("pool", quantity(pool)),
            ("nontaxable", quantity(nontaxable)),
        ],
    )?;
    apply(
        &mut bundle,
        "taxable-allocation",
        &[
            ("converted", quantity(converted)),
            ("nontaxable", quantity(nontaxable)),
            ("taxable", quantity(taxable)),
        ],
    )?;
    apply(
        &mut bundle,
        "carryforward-basis",
        &[
            ("total", quantity(total)),
            ("nontaxable", quantity(nontaxable)),
            ("remaining", quantity(remaining)),
        ],
    )?;
    apply(
        &mut bundle,
        "allocation-summary",
        &[
            ("nontaxable", quantity(nontaxable)),
            ("taxable", quantity(taxable)),
            ("remaining", quantity(remaining)),
        ],
    )?;
    output.verification = Some(verify(&bundle)?);
    output.certificate = Some(bundle);
    output.allocation = Some(Allocation {
        annual_limit_cents: limit,
        total_basis_cents: total,
        available_basis_cents: available,
        aggregate_pool_cents: pool,
        fraction_numerator_cents: available.min(pool),
        fraction_denominator_cents: pool,
        nontaxable_conversion_cents: nontaxable,
        taxable_conversion_cents: taxable,
        remaining_basis_cents: remaining,
    });
    output.status = "calculated".into();
    Ok(output)
}

pub fn assess_json(input: &str) -> Result<String> {
    if input.len() > 64 * 1024 {
        return Err(Error("[E_LIMIT] tax input exceeds 64 KiB".into()));
    }
    let input: BackdoorRothInput = serde_json::from_str(input).map_err(|e| Error(e.to_string()))?;
    serde_json::to_string(&assess(&input)?).map_err(|e| Error(e.to_string()))
}
