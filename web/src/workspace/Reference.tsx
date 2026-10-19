import { useEffect, useState } from 'react'
import { ArrowRight, Search, FileCode2, Braces } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Badge } from '@/components/ui/badge'
import { AppLink, CopyButton, DownloadButton, EmptyState, PageHeading } from './shared'
import { sourceFiles, codeSymbols } from './documentation'

export function ReferenceIndex(){
  return <><PageHeading eyebrow="DEVELOPER REFERENCE" title="From proposition to proof check" description="Read the actual lawlib source, compiler, Rust kernel, and native Python interface." actions={<Button variant="outline" asChild><AppLink to="/docs/language">Language specification <ArrowRight size={14}/></AppLink></Button>}/><div className="reference-flow"><span><Braces size={17}/> .oath source → typed JSON</span><ArrowRight size={17}/><span>Rust verification kernel</span><ArrowRight size={17}/><span>Dependency report</span></div><div className="reference-grid">{sourceFiles.map(file=><Card key={file.id} className="reference-card"><div className="reference-card-icon"><FileCode2 size={20}/><Badge variant="outline">{file.language??'Rust'}</Badge></div><h2><AppLink to={`/reference/${file.id}`}>{file.title}<ArrowRight size={17}/></AppLink></h2><p>{file.summary}</p><code>{file.path}</code><div className="reference-card-bottom"><span>{file.code.split('\n').length.toLocaleString()} lines · {codeSymbols(file.code).length} symbols</span><AppLink to={`/reference/${file.id}`}>Browse source</AppLink></div></Card>)}</div><div className="api-shortcuts"><h2>Use Oathkeeper</h2><AppLink to="/docs/python"><span>Python API</span><span>compile_file · verify_file · verify_case <ArrowRight size={14}/></span></AppLink><AppLink to="/docs/cli"><span>Command line</span><span>compile · verify · normalize <ArrowRight size={14}/></span></AppLink><AppLink to="/docs/architecture"><span>Trust boundaries</span><span>What an accepted proof establishes <ArrowRight size={14}/></span></AppLink></div></>
}

export function SourceReference({id,line}:{id:string;line:string|null}){
  const file=sourceFiles.find(file=>file.id===id)
  const [query,setQuery]=useState('')
  useEffect(()=>{setQuery('')},[id])
  useEffect(()=>{if(line)document.getElementById(`source-line-${line}`)?.scrollIntoView({block:'center'})},[id,line])
  if(!file)return <EmptyState title="Source file not found" description="Browse the source index to choose a source file." action={<AppLink to="/reference">Code reference</AppLink>}/>
  const symbols=codeSymbols(file.code)
  const lines=file.code.split('\n')
  const matches=query?lines.flatMap((text,index)=>text.toLowerCase().includes(query.toLowerCase())?[index+1]:[]):[]
  const selected=Number(line)
  return <><div className="back-link"><AppLink to="/reference">← Code reference</AppLink></div><PageHeading eyebrow={`${file.language??'Rust'} SOURCE`} title={file.title} description={file.summary} actions={<><CopyButton text={file.code} label="Copy file"/><DownloadButton href={`/assets/${id}.${file.path.endsWith('.oath')?'oath':'rs'}`} label={`Download .${file.path.endsWith('.oath')?'oath':'rs'}`}/></>}/><div className="source-layout"><div className="source-view"><div className="source-toolbar"><span><FileCode2 size={15}/><code>{file.path}</code></span><Badge variant="outline">{lines.length} lines</Badge></div><div className="source-search"><Search size={15}/><Input aria-label="Find in source" placeholder="Find in this file…" value={query} onChange={event=>setQuery(event.target.value)}/>{query&&<span>{matches.length} matches</span>}</div>{query&&<div className="source-matches">{matches.length?matches.slice(0,20).map(number=><AppLink key={number} to={`/reference/${id}?line=${number}`}>L{number}</AppLink>):<span>No matching lines.</span>}{matches.length>20&&<span>+{matches.length-20} more</span>}</div>}<div className="source-code" role="region" aria-label={`${file.title} source code`} tabIndex={0}><pre>{lines.map((text,index)=><span key={index} id={`source-line-${index+1}`} className={`source-line ${selected===index+1?'selected':''} ${query&&matches.includes(index+1)?'match':''}`}><AppLink to={`/reference/${id}?line=${index+1}`} className="line-number" aria-label={`Line ${index+1}`}>{index+1}</AppLink><code>{text||' '}</code></span>)}</pre></div></div><aside className="source-symbols"><strong>Symbols <small>{symbols.length}</small></strong>{symbols.map(symbol=><AppLink key={symbol.line} to={`/reference/${id}?line=${symbol.line}`} className={selected===symbol.line?'active':''}><span className={`symbol-kind kind-${symbol.kind}`}>{symbol.kind==='fn'?'ƒ':symbol.kind==='struct'?'S':symbol.kind==='enum'?'E':'T'}</span><code>{symbol.name}</code><small>{symbol.line}</small></AppLink>)}</aside></div></>
}
