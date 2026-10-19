import moduleJson from '@generated/data/module.json'
import caseJson from '@generated/data/cases.json'
import sourceJson from '@generated/data/sources.lock.json'
import coverageJson from '@generated/data/coverage.json'
import { ruleNotes } from '@/catalog/guide-content'
import type { Authority, CaseRecord, Claim, LawModule, Rule, SourceSnapshot, Sort } from '@/types'

export const lawModule=moduleJson as unknown as LawModule
export const records=caseJson as unknown as CaseRecord[]
export const snapshots=sourceJson.sources as SourceSnapshot[]
export const coverage=coverageJson
export const rulePath=(id:string) => `/lawlib/rules/${encodeURIComponent(id)}`
export const authorityPath=(id:string) => `/lawlib/authorities/${encodeURIComponent(id)}`
export const casePath=(slug:string) => `/cases/${slug}`
export const jurisdictionName=(id:string) => lawModule.jurisdictions.find(j=>j.id===id)?.name ?? id
export const authorityName=(a:Authority) => a.id.includes('timbs') ? 'Timbs v. Indiana' : a.id.includes('long') ? 'City of Seattle v. Long' : 'Seattle Ordinance 120573'
export const sourceAsset=(source:SourceSnapshot) => `/assets/${source.file.split('/').at(-1)}`
export const ruleAuthority=(rule:Rule) => lawModule.authorities.find(a=>a.id===rule.authority.authority)!
export const sourceForHash=(sha:string) => snapshots.find(s=>s.sha256===sha)
export const ruleTitle=(rule:Rule) => ruleNotes[rule.id]?.short ?? ({'seattle.parking.duration-threshold':'Elapsed duration exceeds 72 hours','seattle.parking.junk-vehicle-exclusion':'Junk vehicle exclusion','seattle.parking.abandoned-junk-exclusion':'Abandoned junk vehicle exclusion'}[rule.id] ?? '72-hour parking: ordinary branch')
export const ruleSummary=(rule:Rule) => ruleNotes[rule.id]?.body ?? ({'seattle.parking.duration-threshold':'Establishes that an explicitly measured duration is greater than 259,200,000 milliseconds. Exactly 72 hours does not pass; the remaining violation premises must still be established.','seattle.parking.junk-vehicle-exclusion':'The cited section excludes junk motor vehicles from its scope. This rule requires explicit historical applicability and junk-vehicle evidence; it makes no determination under other laws.','seattle.parking.abandoned-junk-exclusion':'The cited section excludes abandoned junk motor vehicles. This rule requires an explicit positive premise and does not infer that status from missing information.'}[rule.id] ?? 'The ordinary 72-hour parking branch requires established applicability, location, duration greater than 72 hours, and explicit negative exception premises. Exceptions are not inferred from missing facts.')
export function claimText(claim:Claim) {
  const terms=(claim.arguments??[]).map(term=>term.kind==='variable' ? `$${term.name}` : term.kind==='quantity'?`${term.value}[${term.unit}]`:String(term.value))
  return `${claim.negative?'¬ ':''}${claim.atom}${terms.length ? `(${terms.join(', ')})` : ''}`
}
export function casesUsingRule(id:string) {
  return records.filter(record=>record.bundle.proof.some(step=>step.reason.rule===id))
}
export function filterRules(query:string,jurisdiction='all',kind='all') {
  const q=query.trim().toLowerCase()
  return lawModule.rules.filter(rule=>{
    if(jurisdiction!=='all'&&rule.jurisdiction!==jurisdiction)return false
    if(kind!=='all'&&ruleAuthority(rule).kind!==kind)return false
    return !q || `${ruleTitle(rule)} ${ruleSummary(rule)} ${JSON.stringify(rule)} ${authorityName(ruleAuthority(rule))}`.toLowerCase().includes(q)
  })
}
export function formatDate(value:string|null) {
  return value ? new Intl.DateTimeFormat('en-US',{year:'numeric',month:'short',day:'numeric',timeZone:'UTC'}).format(new Date(value)) : 'Not applicable'
}
export async function verifySnapshot(bytes:ArrayBuffer,expected:string) {
  const digest=await crypto.subtle.digest('SHA-256',bytes)
  const actual=Array.from(new Uint8Array(digest),byte=>byte.toString(16).padStart(2,'0')).join('')
  return {actual,matches:actual===expected}
}

export const sortText=(sort:Sort)=>typeof sort==='string'?sort:'domain' in sort?sort.domain:`quantity(${sort.quantity.unit})`
