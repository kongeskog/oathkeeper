import { ArrowRight, ArrowUpRight, BookOpen, Braces, Code2, FileCheck2, Github, GitPullRequest, Library, Scale, ShieldCheck, Terminal } from 'lucide-react'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { AppLink } from './shared'

const learn=[
  {icon:Braces,title:'Oath language reference',text:'Syntax and semantics for modules, rules, facts, queries, and proofs.',to:'/language'},
  {icon:ShieldCheck,title:'Trust boundaries',text:'What the kernel checks, what remains an input, and where judgment enters.',to:'/docs/architecture'},
  {icon:FileCheck2,title:'Soundness and limits',text:'The proof calculus, search boundaries, and meaning of a verified derivation.',to:'/docs/soundness'},
]
const build=[
  {icon:Terminal,title:'CLI reference',text:'Compile, verify, normalize, compose, and analyze Oath files.',to:'/docs/cli'},
  {icon:Code2,title:'Python API',text:'Use the verifier and argument tools from Python applications.',to:'/docs/python'},
  {icon:BookOpen,title:'Code reference',text:'Browse the Rust kernel, compiler, WASM bindings, and source modules.',to:'/reference'},
]

function ResourceCard({item}:{item:{icon:typeof BookOpen;title:string;text:string;to:string}}){const Icon=item.icon;return <Card className="resource-card"><Icon/><div><h3>{item.title}</h3><p>{item.text}</p><AppLink to={item.to}>Open resource<ArrowRight/></AppLink></div></Card>}

export function Resources(){
  return <div className="page-container resources-page">
    <div className="resources-hero"><Badge variant="outline">Resources</Badge><h1>Learn, build, and contribute</h1><p>Start with the concepts, use the interface that fits your project, and inspect every layer of the open-source implementation.</p></div>

    <section className="resource-section"><div className="resource-section-heading"><span>UNDERSTAND</span><h2>Core concepts and boundaries</h2></div><div className="resource-grid">{learn.map(item=><ResourceCard key={item.title} item={item}/>)}</div></section>
    <section className="resource-section"><div className="resource-section-heading"><span>BUILD</span><h2>Tools and interfaces</h2></div><div className="resource-grid">{build.map(item=><ResourceCard key={item.title} item={item}/>)}</div></section>

    <section className="project-links"><div><span>OPEN SOURCE</span><h2>Two repositories, one ecosystem</h2><p>The core repository contains the language, checker, bindings, and website. The lawlib repository is the community home for formalized legal modules.</p></div><div className="project-link-list"><a href="https://github.com/kongeskog/oathkeeper" target="_blank" rel="noreferrer"><Github/><span><strong>oathkeeper</strong><small>Language, Rust kernel, APIs, and web app</small></span><ArrowUpRight/></a><a href="https://github.com/kongeskog/oathkeeper-lawlib" target="_blank" rel="noreferrer"><Library/><span><strong>oathkeeper-lawlib</strong><small>Reusable, reviewable legal modules</small></span><ArrowUpRight/></a><a href="https://github.com/kongeskog/oathkeeper/blob/main/CONTRIBUTING.md" target="_blank" rel="noreferrer"><GitPullRequest/><span><strong>Contributing</strong><small>How to propose code, rules, and reviews</small></span><ArrowUpRight/></a></div></section>

    <section className="example-resources"><div className="resource-section-heading"><span>DEEPER EXAMPLES</span><h2>See the system applied to bounded domains</h2><p>These pilots demonstrate the machinery against selected source material. They are examples with explicit coverage limits.</p></div><div className="example-resource-grid"><Card><Scale/><h3>Argument workbench</h3><p>Change supplied evidence and inspect how a real proof path responds.</p><Button variant="outline" asChild><AppLink to="/workbench">Open workbench<ArrowRight/></AppLink></Button></Card><Card><Library/><h3>Case proof explorer</h3><p>Follow proof steps through selected historical Seattle case issues.</p><Button variant="outline" asChild><AppLink to="/cases">Explore cases<ArrowRight/></AppLink></Button></Card><Card><Code2/><h3>Federal tax pilot</h3><p>Inspect a bounded calculator backed by formalized 2025 IRA rules.</p><Button variant="outline" asChild><AppLink to="/tax/backdoor-roth">Open pilot<ArrowRight/></AppLink></Button></Card></div></section>
  </div>
}
