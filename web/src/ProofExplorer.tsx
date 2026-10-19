import { useMemo, useState } from 'react'
import { ArrowDown, ArrowUpRight, Check, CircleAlert } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Switch } from '@/components/ui/switch'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Accordion, AccordionContent, AccordionItem, AccordionTrigger } from '@/components/ui/accordion'
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, DialogTrigger } from '@/components/ui/dialog'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import { traceDependencies, applicationSteps } from '@/lib/proof-model'
import { factLabels, issueControls, ruleNotes } from '@/data/guide-content'
import type { ApplyStep, CaseRecord } from './types'

export function ProofExplorer({ records, initialSlug, initialStepId, compact=false, onCaseChange }: {records: CaseRecord[];initialSlug?:string;initialStepId?:string;compact?:boolean;onCaseChange?:(slug:string)=>void}) {
  const initialRecord=records.find(record=>record.slug===initialSlug)??records[0]
  const [slug, setSlug] = useState(initialRecord.slug)
  const [stepIndex, setStepIndex] = useState(Math.max(0,applicationSteps(initialRecord).findIndex(step=>step.id===initialStepId)))
  const [includeEvidence, setIncludeEvidence] = useState(true)
  const record = records.find(record => record.slug === slug)!
  const steps = useMemo(() => applicationSteps(record) as ApplyStep[], [record])
  const step = steps[stepIndex]
  const rule = record.bundle.rules.find(rule => rule.id === step.reason.rule)!
  const note = ruleNotes[rule.id]
  const [excludedFact, evidenceLabel] = issueControls[slug]
  const trace = traceDependencies(record.bundle.proof, includeEvidence ? null : excludedFact)
  const complete = trace.get(step.id)
  const rootComplete = trace.get(record.bundle.query.root)

  function label(id: string) {
    if (factLabels[id]) return factLabels[id]
    const item = record.bundle.proof.find(s => s.id === id)
    return ruleNotes[item?.reason.rule ?? '']?.short ?? id
  }

  function chooseCase(value: string) {
    setSlug(value); setStepIndex(0); setIncludeEvidence(true)
    onCaseChange?.(value)
  }

  return <section className={`explorer-section ${compact?'workbench':''}`} id="explorer">
    <div className={compact?'workbench-inner':'wrap'}>
      {!compact&&<><div className="section-tag light"><span>02 / THE PROOF EXPLORER</span><span>A REAL SEATTLE CASE</span></div>
      <div className="explorer-heading"><div><p className="case-citation">CITY OF SEATTLE v. LONG · WASHINGTON SUPREME COURT · 2021</p><h2>A real case.<br/><em>A traceable argument.</em></h2></div><p>Seattle impounded Steven Long’s truck after a parking violation. His case raised questions about excessive fines and homestead protections. Follow the actual module’s reasoning, one dependency at a time.</p></div></>}
      <div className="proof-toolbar">
        <div className="case-select"><label htmlFor="case-select">EXPLORE AN ISSUE</label>
          <Select value={slug} onValueChange={chooseCase}>
            <SelectTrigger id="case-select" className="proof-select" aria-label="Explore an issue"><SelectValue /></SelectTrigger>
            <SelectContent className="case-select-popup">{records.map(item => <SelectItem key={item.slug} value={item.slug}>{item.title}</SelectItem>)}</SelectContent>
          </Select>
        </div>
        <div className="proof-context"><span>US <i>→</i> Washington <i>→</i> Seattle</span><small>LAW · OCT 2016 / AUTHORITY · AUG 2021</small></div>
        <Tooltip><TooltipTrigger asChild><Badge variant="outline" className="research-pill" tabIndex={0}><span/> Formalization awaiting review</Badge></TooltipTrigger><TooltipContent className="review-tooltip">Machine-assisted encoding. Independent legal and formal review remains outstanding.</TooltipContent></Tooltip>
      </div>
      <div className="proof-layout">
        <div className="proof-sidebar"><div className="sidebar-title"><span>THE DERIVATION</span><span>{String(steps.length).padStart(2,'0')} {steps.length === 1 ? 'STEP' : 'STEPS'}</span></div>
          <div className="proof-steps" aria-label="Choose a proof step">{steps.map((item, index) => <Button key={item.id} variant={index === stepIndex ? "secondary" : "ghost"} onClick={() => setStepIndex(index)} className={`step-button ${index === stepIndex ? 'active' : ''} ${!trace.get(item.id) ? 'blocked' : ''}`} aria-pressed={index === stepIndex}>
            <span className="step-number">{String(index+1).padStart(2,'0')}</span><span><strong>{ruleNotes[item.reason.rule].title}</strong><small>{ruleNotes[item.reason.rule].source}</small></span>
          </Button>)}</div>
          <div className="sidebar-footer"><span className="small-dot"/> Every step links to its inputs.</div>
        </div>
        <div className="proof-detail" aria-live="polite" aria-atomic="true" data-testid="proof-detail">
          <div className="detail-kicker"><Badge variant="outline">{note.kind.toUpperCase()}</Badge> STEP {String(stepIndex+1).padStart(2,'0')} OF {String(steps.length).padStart(2,'0')}</div>
          <h3>{note.short}</h3><p className="detail-explanation">{note.body}</p>
          <div className="detail-premises"><div className="detail-subheading">REQUIRED INPUTS · {step.reason.premises.length} DEPENDENCIES</div><div className="premise-list">{step.reason.premises.map(id => <Badge variant="outline" key={id} className={`premise-chip ${!trace.get(id) ? 'missing' : ''}`}>{!trace.get(id) ? '○' : '✓'} {label(id)}</Badge>)}</div></div>
          <div className="detail-conclusion"><span className="turnstile">{complete ? '⊢' : '⋯'}</span><span>{complete ? note.result : 'This step is blocked: a required input is missing.'}</span></div>
          <div className="detail-citation"><span>{rule.authority.pinpoint}</span><a href={`assets/${note.local}`} target="_blank" rel="noopener" className="inline-flex items-center gap-1">Read pinned source <ArrowUpRight size={12}/></a></div>
          <Accordion type="single" collapsible className="exact-step" key={`${slug}-${step.id}`}><AccordionItem value="step" className="border-none"><AccordionTrigger className="inspect-step">Inspect the exact proof step</AccordionTrigger><AccordionContent><pre>{JSON.stringify(step,null,2)}</pre></AccordionContent></AccordionItem></Accordion>
        </div>
      </div>
      <div className="proof-controls"><div><div className="evidence-toggle"><Switch id="evidence-toggle" checked={includeEvidence} onCheckedChange={setIncludeEvidence} aria-labelledby="evidence-label"/><label id="evidence-label" htmlFor="evidence-toggle">{evidenceLabel}</label></div><p>Remove this input to see where the reasoning stops.</p></div><div className="proof-outcome" aria-live="polite" data-testid="proof-outcome"><span className={`outcome-icon ${rootComplete ? '' : 'failed'}`}>{rootComplete ? <Check size={14}/> : <CircleAlert size={14}/>}</span><div><strong>{rootComplete ? 'Rust-verified baseline · inputs present' : 'Blocked · missing a required premise'}</strong><small>{rootComplete ? '0 ORACLE ATTESTATIONS · SOURCED FINDINGS STILL VISIBLE' : 'DEPENDENCY WALKTHROUGH · NO CONCLUSION ESTABLISHED'}</small></div></div></div>
      <div className="explorer-bottom"><p>Interactive dependency walkthrough of Rust-verified fixtures. Open the Argument workbench for live Rust analysis and alternative proof search.</p><div>
        <Dialog><DialogTrigger asChild><Button variant="link" className="subtle-button light-button">Inspect the assumptions <ArrowUpRight size={13}/></Button></DialogTrigger><DialogContent className="assumptions-modal"><DialogHeader><p className="eyebrow">THE TRUST SURFACE</p><DialogTitle>What this proof assumes.</DialogTitle><DialogDescription className="dialog-intro">These are the actual fact inputs from the selected proof bundle. A recorded court finding remains an attributed assumption of this reconstruction.</DialogDescription></DialogHeader><div className="assumptions-list">{record.report.fact_assumptions.map(fact => <article className="assumption-entry" key={fact.id}><Badge variant="outline" className="assumption-kind">{fact.basis.kind.replaceAll('_',' ').toUpperCase()}</Badge><h3>{factLabels[fact.id] ?? fact.id}</h3><p>{fact.basis.explanation}</p><small>{fact.basis.pinpoint}<br/>{fact.basis.source.citation}<br/>Recorded: {fact.recorded_at}</small></article>)}</div><div className="dialog-bottom">Source integrity checks identify bytes. They do not establish that an encoding is legally correct.</div></DialogContent></Dialog>
        <Button variant="link" className="subtle-button light-button" asChild><a href={`assets/${slug}.bundle.json`} download>Download proof <ArrowDown size={13}/></a></Button>
      </div></div>
      <div className="case-footnote"><span>WHAT YOU ARE SEEING</span><p>The excessive-fines reconstruction uses the court’s <em>recorded</em> disproportionality finding. It does not independently decide the merits. Applying a holding to a new person requires new evidence and established scope.</p></div>
    </div>
  </section>
}
