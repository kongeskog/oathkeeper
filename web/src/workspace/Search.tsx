import { useEffect, useMemo, useState } from 'react'
import { Search, CornerDownLeft, FileText, BookOpen, Braces, Scale } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Dialog, DialogContent, DialogDescription, DialogTitle } from '@/components/ui/dialog'
import { Command, CommandEmpty, CommandGroup, CommandInput, CommandItem, CommandList } from '@/components/ui/command'
import { lawModule, records, rulePath, authorityPath, authorityName, casePath, ruleTitle, ruleSummary } from './catalog'
import { taxData } from './tax-model'
import { docs, sourceFiles, codeSymbols } from './documentation'
import { navigate } from './router'

export function SearchDialog(){
  const [open,setOpen]=useState(false)
  const [query,setQuery]=useState('')
  useEffect(()=>{function onKey(event:KeyboardEvent){if((event.metaKey||event.ctrlKey)&&event.key.toLowerCase()==='k'){event.preventDefault();setOpen(value=>!value)}}window.addEventListener('keydown',onKey);return()=>window.removeEventListener('keydown',onKey)},[])
  const entries=useMemo(()=>[
    {id:'tax',title:'Backdoor Roth IRA calculator',detail:'2025 federal tax · local Rust / WASM',terms:'pro rata basis contribution conversion IRA tax',group:'Workspace',path:'/tax/backdoor-roth'},
    ...taxData.module.rules.map(rule=>({id:rule.id,title:rule.id,detail:rule.source.citation,terms:JSON.stringify(rule),group:'Rules',path:`/tax/backdoor-roth?view=rules&rule=${encodeURIComponent(rule.id)}`})),
    {id:'shank',title:'Shank v. Commissioner',detail:'T.C. Memo. 2018-33 · actual basis findings',terms:'IRS Form 8606 IRA basis evidence testimony',group:'Case proofs',path:'/tax/backdoor-roth?view=precedent'},
    {id:'workbench',title:'Argument workbench',detail:'Live Rust analysis of real evidence',terms:'assumptions opposing gaps sensitivity',group:'Workspace',path:'/workbench'},
    {id:'review',title:'Review & evaluation',detail:'Source mappings and pending reviews',terms:'pilot baseline',group:'Workspace',path:'/review'},
    ...lawModule.rules.map(rule=>({id:rule.id,title:ruleTitle(rule),detail:rule.id,terms:ruleSummary(rule),group:'Rules',path:rulePath(rule.id)})),
    ...lawModule.authorities.map(authority=>({id:authority.id,title:authorityName(authority),detail:authority.source.citation,terms:authority.kind,group:'Authorities',path:authorityPath(authority.id)})),
    ...records.map(record=>({id:record.slug,title:record.title,detail:'City of Seattle v. Long · historical case file',terms:record.bundle.query.goal.atom,group:'Case proofs',path:casePath(record.slug)})),
    ...docs.map(doc=>({id:doc.id,title:doc.title,detail:doc.summary,terms:doc.content,group:'Documentation',path:`/docs/${doc.id}`})),
    ...sourceFiles.flatMap(file=>[{id:file.id,title:file.title,detail:file.path,terms:file.summary,group:'Code reference',path:`/reference/${file.id}`},...codeSymbols(file.code).map(symbol=>({id:`${file.id}:${symbol.line}`,title:symbol.name,detail:`${symbol.kind} · ${file.path}:${symbol.line}`,terms:'',group:'Code reference',path:`/reference/${file.id}?line=${symbol.line}`}))]),
  ],[])
  const results=entries.filter(entry=>!query.trim()||`${entry.title} ${entry.detail} ${entry.terms}`.toLowerCase().includes(query.toLowerCase())).slice(0,35)
  const groups=['Workspace','Rules','Authorities','Case proofs','Documentation','Code reference']
  return <><Button variant="outline" className="global-search-trigger" onClick={()=>setOpen(true)}><Search size={15}/><span>Search Oathkeeper…</span><kbd>⌘ K</kbd></Button><Dialog open={open} onOpenChange={value=>{setOpen(value);if(!value)setQuery('')}}><DialogContent className="global-search-dialog"><DialogTitle className="sr-only">Search Oathkeeper</DialogTitle><DialogDescription className="sr-only">Search the loaded law library, case proofs, documentation, and Rust code symbols.</DialogDescription><Command shouldFilter={false}><CommandInput placeholder="Search the library and documentation…" value={query} onValueChange={setQuery}/><CommandList><CommandEmpty>No results. Try a rule name, citation, or API function.</CommandEmpty>{groups.map(group=>{const items=results.filter(item=>item.group===group);return items.length?<CommandGroup key={group} heading={group}>{items.slice(0,8).map(item=><CommandItem key={`${group}:${item.id}`} value={`${group}:${item.id}`} onSelect={()=>{navigate(item.path);setOpen(false);setQuery('')}}>{group==='Rules'?<Scale/>:group==='Code reference'?<Braces/>:group==='Documentation'?<BookOpen/>:<FileText/>}<span><strong>{item.title}</strong><small>{item.detail}</small></span><CornerDownLeft className="search-return"/></CommandItem>)}</CommandGroup>:null})}</CommandList><div className="search-dialog-footer">↑ ↓ Navigate <span>↵ Open</span><span>Esc Close</span></div></Command></DialogContent></Dialog></>
}
