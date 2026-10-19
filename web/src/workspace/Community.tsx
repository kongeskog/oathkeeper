import { useEffect, useRef, useState } from 'react'
import community from '@/data/community.json'
import lawPackage from '@/data/lawlib-package.json'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardHeader, CardTitle, CardDescription, CardContent } from '@/components/ui/card'
import { Textarea } from '@/components/ui/textarea'
import { Select, SelectTrigger, SelectValue, SelectContent, SelectItem } from '@/components/ui/select'
import { Accordion, AccordionItem, AccordionTrigger, AccordionContent } from '@/components/ui/accordion'
import { CopyButton, DownloadButton, JsonView, Notice } from './shared'

export function CommunityReview() {
  const [moduleId,setModuleId]=useState(community.modules[0].id)
  const selected=community.modules.find(m=>m.id===moduleId)!
  const [journal,setJournal]=useState(JSON.stringify({format:'oathkeeper.journal/1',events:[]},null,2))
  const [trust,setTrust]=useState(JSON.stringify({keys:[]},null,2))
  const [result,setResult]=useState<unknown>()
  const [error,setError]=useState('')
  const [pending,setPending]=useState(false)
  const worker=useRef<Worker|null>(null)
  const generation=useRef(0)
  const timer=useRef<ReturnType<typeof setTimeout>|undefined>(undefined)
  function cancel(){generation.current++;worker.current?.terminate();worker.current=null;clearTimeout(timer.current)}
  useEffect(()=>()=>cancel(),[])
  function changed(){cancel();setResult(undefined);setError('');setPending(false)}
  function verify(){
    changed()
    try {
      const request=JSON.stringify({operation:'reviews',artifact_sha256:selected.reviews.artifact_sha256,journal:JSON.parse(journal),trust:JSON.parse(trust),authors:['oathkeeper.machine-assisted-author','kongeskog']})
      const id=generation.current;const active=new Worker('/assets/policy-worker.js?v=0.5.0',{type:'module'});worker.current=active;setPending(true)
      timer.current=setTimeout(()=>{if(id===generation.current){cancel();setPending(false);setError('Verification exceeded 20 seconds. Use the CLI for a larger journal.')}},20000)
      active.onmessage=({data})=>{if(id!==generation.current)return;clearTimeout(timer.current);setPending(false);active.terminate();if(data.error)setError(data.error);else setResult(data.result)}
      active.onerror=()=>{if(id!==generation.current)return;clearTimeout(timer.current);setPending(false);active.terminate();setError('The Rust verifier could not start. Retry or use the CLI.')}
      active.postMessage({request})
    }catch(e){setPending(false);setError(String(e))}
  }
  return <div className="my-5 space-y-4"><Card><CardHeader><CardTitle>Community lawlib</CardTitle><CardDescription>Anyone can contribute sources, corrections, tests and reviews. Professional credentials are not required.</CardDescription></CardHeader><CardContent className="space-y-4"><div className="flex flex-wrap gap-2"><Button variant="outline" asChild><a href="https://github.com/kongeskog/oathkeeper-lawlib" target="_blank" rel="noreferrer">Lawlib repository</a></Button><Button variant="outline" asChild><a href="https://github.com/kongeskog/oathkeeper" target="_blank" rel="noreferrer">Engine and app</a></Button><DownloadButton href="/assets/community.json" label="Community review records"/></div><div className="flex flex-wrap items-center gap-2 text-xs"><Badge variant="outline">Lawlib {lawPackage.package_version}</Badge><a className="font-mono underline" href={`${lawPackage.repository}/commit/${lawPackage.revision}`} target="_blank" rel="noreferrer">{lawPackage.revision.slice(0,12)}</a><DownloadButton href="/assets/lawlib.lock.json" label="Pinned package lock"/></div><p className="text-sm text-muted-foreground">Legal sources live in a separate repository. This app uses an immutable snapshot. Reviews bind the source, scope contract and interpretation notes; they do not approve future versions.</p>{community.modules.map(m=><div key={m.id} className="space-y-2 border-t pt-3"><div className="flex flex-wrap items-center gap-2"><span className="font-medium text-sm">{m.id}</span><Badge variant="outline">{m.reviews.current.length} signed review statements</Badge><Badge variant="secondary">{m.coverage.length} coverage contracts</Badge></div><div className="flex items-center gap-2"><code className="break-all text-xs">{m.reviews.artifact_sha256}</code><CopyButton text={m.reviews.artifact_sha256} label={`Copy ${m.id} review digest`}/></div></div>)}</CardContent></Card><Accordion type="single" collapsible><AccordionItem value="verify"><AccordionTrigger>Verify signed community reviews locally</AccordionTrigger><AccordionContent><div className="space-y-4"><p className="text-sm text-muted-foreground">The Rust WASM checker verifies signatures against the keys you supply and shows current statements for the selected artifact. Key ownership, reviewer identity and legal correctness remain separate questions. An empty journal contains no approvals.</p><Select value={moduleId} onValueChange={v=>{changed();setModuleId(v)}}><SelectTrigger aria-label="Review artifact"><SelectValue/></SelectTrigger><SelectContent>{community.modules.map(m=><SelectItem key={m.id} value={m.id}>{m.id}</SelectItem>)}</SelectContent></Select><div><label className="text-sm font-medium" htmlFor="review-journal">Signed journal JSON</label><Textarea id="review-journal" className="mt-2 min-h-36 font-mono text-xs" value={journal} onChange={e=>{changed();setJournal(e.target.value)}}/></div><div><label className="text-sm font-medium" htmlFor="review-keys">Trusted keys JSON</label><Textarea id="review-keys" className="mt-2 min-h-24 font-mono text-xs" value={trust} onChange={e=>{changed();setTrust(e.target.value)}}/></div><Button onClick={verify} disabled={pending}>{pending?'Checking signatures…':'Verify review journal'}</Button>{error&&<Notice title="Review verification failed">{error}</Notice>}{result!==undefined&&<Card><CardHeader><CardTitle className="text-base">Journal checked for this artifact</CardTitle><CardDescription>Inspect each statement, stale event and trust limitation.</CardDescription></CardHeader><CardContent><JsonView value={result}/></CardContent></Card>}<a className="text-sm underline" href="https://github.com/kongeskog/oathkeeper-lawlib/blob/main/community/README.md" target="_blank" rel="noreferrer">Create a review statement and register a public key</a></div></AccordionContent></AccordionItem></Accordion></div>
}
