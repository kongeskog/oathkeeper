export type Sort = string | {domain:string} | {quantity:{unit:string}}
export type Term = {kind:'quantity';value:number;unit:string} |  {kind: 'symbol'; value: string} | {kind: 'integer'; value: number} | {kind: 'variable'; name: string}
export interface Claim { atom: string; negative: boolean; arguments?: Term[] }
export interface Source {citation: string; uri: string; sha256: string}
export interface ProofStep {
  id: string
  claim: Claim
  reason: {kind: string; fact?: string; rule?: string; version?: string; premises?: string[]; substitution?: Record<string,Term>}
}
export interface ApplyStep extends ProofStep {
  reason: {kind: 'apply'; rule: string; version: string; premises: string[]; substitution?: Record<string,Term>}
}
export interface Rule {
  id: string
  version: string
  jurisdiction: string
  effective_from: string
  effective_until: string | null
  recorded_at: string
  parameters: {name: string; sort: Sort}[]
  premises: Claim[]
  conclusion: Claim
  source: Source
  authority: {authority: string; role: string; pinpoint: string; scope: Claim}
  [key: string]: unknown
}
export interface Fact {
  id: string; claim: Claim; recorded_at: string
  basis: {kind: string; source: Source; pinpoint: string; explanation: string}
}
export interface CaseRecord {
  slug: string; title: string
  bundle: {proof: ProofStep[]; rules: Rule[]; query: {root: string; goal: Claim; jurisdiction: string; as_of: string; known_at: string; forum: string}; [key: string]: unknown}
  report: {fact_assumptions: Fact[]; oracle_count: number; kernel_version: string; unreviewed_authority_assessment_count: number; [key: string]: unknown}
}
export interface Authority {
  id: string; kind: string; jurisdiction: string; issuing_body: string
  publication: string; decided_at: string | null; source: Source
}
export interface Assessment {
  id: string; authority: string; rule: string; rule_version: string; forum: string
  as_of: string; recorded_at: string; force: string; treatment: string
  review_status: string; source: Source; rationale: string
}
export interface LawModule {
  language: string; package: string; version: string
  jurisdictions: {id:string;name:string;parents:string[]}[]
  rules: Rule[]; authorities: Authority[]; authority_assessments: Assessment[]
}
export interface SourceSnapshot {
  id:string;file:string;sha256:string;uri:string;official_uri:string
  retrieved_at:string;representation:string;source_status:string;rights:string;extractor:string
}
export interface RuleNote {title: string; short: string; source: string; kind: string; body: string; result: string; local: string}
