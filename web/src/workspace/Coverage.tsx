import { Badge } from '@/components/ui/badge'
import { Card, CardHeader, CardTitle, CardDescription, CardContent } from '@/components/ui/card'
import { Accordion, AccordionItem, AccordionTrigger, AccordionContent } from '@/components/ui/accordion'
import { JsonView } from './shared'
import { claimText } from './catalog'
import type { Claim } from '@/types'

export interface CoverageResult {
  status:string;context_sha256:string;completeness:string;reasons:string[]
  contract:null|{id:string;question:string;conclusion_kind:string;jurisdictions:string[];forums:string[];law_from:string;law_until:string;exclusions:string[];known_omissions:string[]}
  conditions:{id:string;goal:Claim;status:string;search_complete:boolean}[]
}
const labels:Record<string,string>={not_declared:'Coverage not declared',conditions_supported:'Scope conditions supported',outside_scope:'Outside declared scope',needs_information:'Scope needs more information',conflicting_scope_evidence:'Scope evidence conflicts'}
export function CoveragePanel({coverage}:{coverage?:CoverageResult|null}) {
  if(!coverage)return null
  return <Card><CardHeader><div className="flex flex-wrap items-center gap-2"><CardTitle className="text-base">Declared coverage</CardTitle><Badge variant="outline" data-testid="coverage-status">{labels[coverage.status]??coverage.status}</Badge></div><CardDescription>{coverage.contract?.question??'This context has no selected coverage contract.'}</CardDescription></CardHeader><CardContent className="space-y-3"><p className="text-sm text-muted-foreground">{coverage.completeness}</p>{coverage.contract&&<><div className="flex flex-wrap gap-2"><Badge variant="secondary">{coverage.contract.conclusion_kind}</Badge><Badge variant="outline">{coverage.contract.jurisdictions.join(', ')}</Badge></div><p className="text-xs">Governing law interval: {coverage.contract.law_from} ≤ date &lt; {coverage.contract.law_until}</p>{coverage.conditions.map(c=><div className="border-l-2 pl-3" key={c.id}><code className="break-words text-xs">{claimText(c.goal)}</code><p className="text-xs text-muted-foreground">{c.status.replaceAll('_',' ')}{c.search_complete?'':' · search limited'}</p></div>)}<Accordion type="single" collapsible><AccordionItem value="limits"><AccordionTrigger>Exclusions and known omissions</AccordionTrigger><AccordionContent><ul className="list-disc space-y-2 pl-5 text-sm">{[...coverage.contract.exclusions,...coverage.contract.known_omissions].map((s,i)=><li key={i}>{s}</li>)}</ul></AccordionContent></AccordionItem><AccordionItem value="proofs"><AccordionTrigger>Scope certificates and exact context</AccordionTrigger><AccordionContent><JsonView value={coverage}/></AccordionContent></AccordionItem></Accordion></>}{coverage.reasons.map(r=><p key={r} className="text-sm">{r}</p>)}</CardContent></Card>
}
