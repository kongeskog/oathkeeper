use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Term {
    Symbol { value: String },
    Integer { value: i64 },
    Quantity { value: i64, unit: String },
    Variable { name: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Sort {
    Symbol,
    Integer,
    Quantity {
        unit: String,
    },
    /// Nominal domain type, declared in the module (language 0.3).
    Domain(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Predicate {
    pub id: String,
    pub arguments: Vec<Sort>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Constant {
    pub id: String,
    pub sort: Sort,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Parameter {
    pub name: String,
    pub sort: Sort,
}

/// An exact proposition name and explicit polarity. No closed-world negation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Claim {
    pub atom: String,
    #[serde(default)]
    pub negative: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub arguments: Vec<Term>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Jurisdiction {
    pub id: String,
    pub name: String,
    /// Applicability edges, not automatic override priorities.
    #[serde(default)]
    pub parents: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub citation: String,
    pub uri: String,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rule {
    pub id: String,
    pub version: String,
    pub jurisdiction: String,
    pub effective_from: String,
    pub effective_until: Option<String>,
    pub recorded_at: String,
    pub source: Source,
    pub premises: Vec<Claim>,
    pub conclusion: Claim,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<Parameter>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authority: Option<AuthorityUse>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time_basis: Option<TimeBasis>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeBasis {
    LawTime,
    AuthorityTime,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorityKind {
    Case,
    Ordinance,
    Statute,
    Constitution,
    AdministrativeGuidance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Publication {
    Published,
    Unpublished,
    Enacted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Authority {
    pub id: String,
    pub kind: AuthorityKind,
    pub jurisdiction: String,
    pub issuing_body: String,
    /// Required for judicial decisions. Ordinance enactment/effective dates
    /// remain distinct from decision dates and belong in source metadata.
    pub decided_at: Option<String>,
    pub publication: Publication,
    pub source: Source,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorityRole {
    Holding,
    Text,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthorityUse {
    pub authority: String,
    pub role: AuthorityRole,
    pub pinpoint: String,
    /// Must be an explicit rule premise, instantiated like all other premises.
    pub scope: Claim,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Force {
    Binding,
    Persuasive,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Treatment {
    Operative,
    Overruled,
    Vacated,
    Repealed,
    Superseded,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewStatus {
    MachineAssistedUnreviewed,
    Reviewed,
}

/// An explicit, trusted assessment of one rule in one forum at one legal time.
/// It is not an automatic citator or a certified determination of binding force.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthorityAssessment {
    pub id: String,
    pub authority: String,
    pub rule: String,
    pub rule_version: String,
    pub forum: String,
    pub as_of: String,
    pub recorded_at: String,
    pub force: Force,
    pub treatment: Treatment,
    pub review_status: ReviewStatus,
    pub source: Source,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fact {
    pub id: String,
    pub claim: Claim,
    pub evidence_sha256: String,
    pub recorded_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub basis: Option<FactBasis>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FactKind {
    RecordFact,
    JudicialFinding,
    ScopeAssumption,
    InterpretationAssumption,
    PartyAllegation,
    StipulatedFact,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FactBasis {
    pub kind: FactKind,
    pub source: Source,
    pub pinpoint: String,
    pub explanation: String,
}

/// A frozen output of a subjective function. The kernel never calls a model.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Attestation {
    pub id: String,
    pub oracle: String,
    pub provider: String,
    pub provider_version: String,
    pub question: String,
    pub claim: Claim,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disposition: Option<ResponseDisposition>,
    pub evidence_sha256: String,
    pub recorded_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OraclePolicy {
    pub oracle: String,
    pub atom: String,
    pub question: String,
    pub evidence_sha256: String,
    pub providers: Vec<OracleProvider>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_confidence: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protocol: Option<OracleProtocol>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub arguments: Vec<Term>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OracleProvider {
    pub id: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseDisposition {
    Asserted,
    Abstained,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OracleRole {
    Observation,
    Classification,
    LegalAssessment,
    InstitutionalDetermination,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OracleAgreement {
    Single,
    Unanimous,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OracleProtocol {
    pub role: OracleRole,
    pub agreement: OracleAgreement,
    pub procedure: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence_meaning: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    Fact,
    Attestation,
}

/// A recorded withdrawal is an additional event; it never proves the opposite claim.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Retraction {
    pub id: String,
    pub target_kind: EvidenceKind,
    pub target: String,
    pub recorded_at: String,
    pub source: Source,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Reason {
    Fact {
        fact: String,
    },
    Oracle {
        attestation: String,
    },
    Apply {
        rule: String,
        version: String,
        premises: Vec<String>,
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        substitution: BTreeMap<String, Term>,
    },
    Compute {
        operator: Computation,
    },
    Compare {
        operator: Comparison,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Computation {
    QuantityAdd,
    QuantitySubtract,
    /// Round amount * min(basis, total) / total to the nearest integer unit, ties up.
    QuantityAllocate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Comparison {
    GreaterThan,
    LessThan,
    Equal,
    QuantityGreaterThan,
    QuantityLessThan,
    QuantityEqual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Step {
    pub id: String,
    pub claim: Claim,
    pub reason: Reason,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Query {
    pub jurisdiction: String,
    /// Legal time, independent of the knowledge cutoff.
    pub as_of: String,
    pub known_at: String,
    pub goal: Claim,
    pub root: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub forum: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authority_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub purpose: Option<CasePurpose>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temporal_rationale: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CasePurpose {
    HistoricalReconstruction,
    NewMatterAssessment,
    SyntheticTest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bundle {
    pub language: String,
    pub package: String,
    pub version: String,
    pub jurisdictions: Vec<Jurisdiction>,
    pub rules: Vec<Rule>,
    pub facts: Vec<Fact>,
    pub attestations: Vec<Attestation>,
    pub oracle_policy: Vec<OraclePolicy>,
    pub query: Query,
    pub proof: Vec<Step>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub retractions: Vec<Retraction>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub authorities: Vec<Authority>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub authority_assessments: Vec<AuthorityAssessment>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub types: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub predicates: Vec<Predicate>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constants: Vec<Constant>,
}

/// Reusable source-backed rules, independent of any particular case facts.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LawModule {
    pub language: String,
    pub package: String,
    pub version: String,
    pub jurisdictions: Vec<Jurisdiction>,
    pub rules: Vec<Rule>,
    pub authorities: Vec<Authority>,
    pub authority_assessments: Vec<AuthorityAssessment>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub types: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub predicates: Vec<Predicate>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constants: Vec<Constant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaseFile {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constants: Vec<Constant>,
    pub facts: Vec<Fact>,
    pub attestations: Vec<Attestation>,
    pub oracle_policy: Vec<OraclePolicy>,
    pub query: Query,
    pub proof: Vec<Step>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub retractions: Vec<Retraction>,
}

impl LawModule {
    pub fn with_case(self, case: CaseFile) -> Bundle {
        let mut constants = self.constants;
        constants.extend(case.constants);
        Bundle {
            language: self.language,
            package: self.package,
            version: self.version,
            jurisdictions: self.jurisdictions,
            rules: self.rules,
            authorities: self.authorities,
            authority_assessments: self.authority_assessments,
            facts: case.facts,
            attestations: case.attestations,
            oracle_policy: case.oracle_policy,
            query: case.query,
            proof: case.proof,
            retractions: case.retractions,
            types: self.types,
            predicates: self.predicates,
            constants,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub language: String,
    pub kernel_version: String,
    pub package: String,
    pub package_version: String,
    pub bundle_sha256: String,
    pub status: String,
    pub goal: Claim,
    pub jurisdiction: String,
    pub as_of: String,
    pub known_at: String,
    pub applicable_jurisdictions: Vec<String>,
    pub rules_used: Vec<(String, String)>,
    pub facts_used: Vec<String>,
    pub oracle_assumptions: Vec<String>,
    pub oracle_count: usize,
    pub assurance: Assurance,
    pub authorities_used: Vec<Authority>,
    pub authority_assessments_used: Vec<AuthorityAssessment>,
    pub fact_assumptions: Vec<Fact>,
    pub unreviewed_authority_assessment_count: usize,
    pub scope_assumption_count: usize,
    pub record_fact_count: usize,
    pub judicial_finding_count: usize,
    pub oracle_requests_used: Vec<OraclePolicy>,
    pub event_at: Option<String>,
    pub authority_at: String,
    pub purpose: Option<CasePurpose>,
}

/// Separate assertions about the trust boundary; no scalar correctness score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assurance {
    pub derivation: String,
    pub legal_fidelity: String,
    pub corpus_completeness: String,
    pub evidence_authenticity: String,
    pub oracle_authentication: String,
    pub counterargument_search: String,
    pub judicial_acceptance: String,
    pub authority_reviews_reported: usize,
    pub interpretation_assumptions: usize,
    pub party_allegations: usize,
    pub stipulated_facts: usize,
    pub limitations: Vec<String>,
}
