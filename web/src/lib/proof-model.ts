import type { ApplyStep, CaseRecord, ProofStep } from '../types'

/** Dependency visualization of fixed Rust-verified fixtures, not a proof kernel. */
export function traceDependencies(proof: ProofStep[], excludedFact: string | null = null): Map<string,boolean> {
  const available = new Map<string,boolean>()
  for (const step of proof) {
    let complete = true
    if (step.reason.kind === 'fact') complete = step.reason.fact !== excludedFact
    if (step.reason.kind === 'apply') complete = step.reason.premises?.every(id => available.get(id) === true) ?? false
    available.set(step.id, complete)
  }
  return available
}

export function applicationSteps(record: CaseRecord): ApplyStep[] {
  return record.bundle.proof.filter((step): step is ApplyStep => step.reason.kind === 'apply')
}

export function escapeHtml(value: unknown): string {
  const replacements: Record<string,string> = {'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}
  return String(value).replace(/[&<>"']/g, char => replacements[char])
}

export function prettyJson(value: unknown): string {
  return escapeHtml(JSON.stringify(value,null,2)).replace(/(&quot;.*?&quot;)(\s*:)?|\b(true|false|null|\d+)\b/g, (match: string, string: string | undefined, colon: string | undefined) => {
    if (string) return `<span class="${colon ? 'json-key' : 'json-string'}">${string}</span>${colon ?? ''}`
    return `<span class="json-value">${match}</span>`
  })
}
