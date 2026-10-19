import { useState } from 'react'
import { ArrowUpRight, CheckCircle2, Fingerprint, LoaderCircle, AlertCircle } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Card } from '@/components/ui/card'
import { Accordion, AccordionContent, AccordionItem, AccordionTrigger } from '@/components/ui/accordion'
import { AppLink, CopyButton, DownloadButton, Notice, PageHeading } from './shared'
import { snapshots, sourceAsset, verifySnapshot, formatDate, lawModule, authorityPath, authorityName } from './catalog'

type CheckState={status:'checking'|'match'|'mismatch'|'error';message:string}
export function Sources(){
  const [checks,setChecks]=useState<Record<string,CheckState>>({})
  async function check(id:string){
    const source=snapshots.find(source=>source.id===id)!
    setChecks(value=>({...value,[id]:{status:'checking',message:'Computing SHA-256…'}}))
    try{
      const response=await fetch(sourceAsset(source))
      if(!response.ok)throw new Error('Snapshot could not be downloaded.')
      const result=await verifySnapshot(await response.arrayBuffer(),source.sha256)
      setChecks(value=>({...value,[id]:{status:result.matches?'match':'mismatch',message:result.matches?'SHA-256 matches the pinned snapshot.':`Digest mismatch: ${result.actual}`}}))
    }catch(error){setChecks(value=>({...value,[id]:{status:'error',message:error instanceof Error?error.message:'Integrity check failed.'}}))}
  }
  const running=Object.values(checks).some(check=>check.status==='checking')
  async function checkAll(){for(const source of snapshots)await check(source.id)}
  return <div className="page-container"><PageHeading eyebrow="PROVENANCE & INTEGRITY" title="Source snapshots" description="Inspect the exact artifacts behind the formalization and verify their bytes against the source lock." actions={<><DownloadButton href="/assets/sources.lock.json" label="Source lock"/><Button onClick={checkAll} disabled={running}>{running?<LoaderCircle className="animate-spin" size={15}/>:<Fingerprint size={15}/>}Verify all snapshots</Button></>}/><p className="mb-4 text-sm"><AppLink className="underline" to="/tax/backdoor-roth?view=sources">Federal tax sources · five official IRS and Tax Court PDFs</AppLink></p><Notice title="Byte integrity is one part of provenance">These checks download the pinned artifact and compute its SHA-256 in your browser. A matching digest does not establish authenticity, current legal validity, or faithful formalization.</Notice><div className="source-cards">{snapshots.map(source=>{const state=checks[source.id];const authority=lawModule.authorities.find(authority=>authority.source.sha256===source.sha256);return <Card key={source.id} className="source-card"><div className="source-heading"><div><Badge variant="outline">{source.extractor==='identity'?'PDF snapshot':'Normalized judicial text'}</Badge><h2>{authority?authorityName(authority):'Long · archival mirror'}</h2><p>{source.file.split('/').at(-1)} · retrieved {formatDate(source.retrieved_at)}</p></div><Button variant="outline" size="sm" onClick={()=>check(source.id)} disabled={state?.status==='checking'}>{state?.status==='checking'?<LoaderCircle className="animate-spin" size={14}/>:<Fingerprint size={14}/>}Verify bytes</Button></div><div className="source-digest"><span>SHA-256</span><code>{source.sha256}</code><CopyButton text={source.sha256} label="Copy hash"/></div>{state&&<div role="status" className={`integrity-result ${state.status}`}>{state.status==='match'?<CheckCircle2 size={15}/>:state.status==='checking'?<LoaderCircle className="animate-spin" size={15}/>:<AlertCircle size={15}/>}<span>{state.message}</span></div>}<p className="source-status">{source.source_status}</p><Accordion type="single" collapsible><AccordionItem value="representation"><AccordionTrigger>Representation, extraction & rights</AccordionTrigger><AccordionContent><h3>Byte representation</h3><p>{source.representation}</p><h3>Rights</h3><p>{source.rights}</p></AccordionContent></AccordionItem></Accordion><div className="source-actions"><DownloadButton href={sourceAsset(source)} label="Pinned artifact"/><Button variant="ghost" size="sm" asChild><a href={source.official_uri} target="_blank" rel="noopener">Official source<ArrowUpRight size={13}/></a></Button>{authority&&<AppLink to={authorityPath(authority.id)}>View linked authority<ArrowUpRight size={13}/></AppLink>}</div></Card>})}</div></div>
}
