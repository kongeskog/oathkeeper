import { useEffect, isValidElement, type ReactNode } from 'react'
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import { ArrowLeft, ArrowRight, FileCode2 } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { AppLink, CodeBlock, EmptyState, PageHeading } from './shared'
import { docs, docLink } from './documentation'

export function headingId(text:string){return text.toLowerCase().replace(/[^a-z0-9\s-]/g,'').trim().replace(/\s+/g,'-')}
function textContent(node:ReactNode):string {
  if(typeof node==='string'||typeof node==='number')return String(node)
  if(Array.isArray(node))return node.map(textContent).join('')
  if(isValidElement<{children?:ReactNode}>(node))return textContent(node.props.children)
  return ''
}
export function Documentation({id,section,basePath,standalone=false}:{id:string;section:string|null;basePath?:string;standalone?:boolean}){
  const index=docs.findIndex(doc=>doc.id===id)
  const doc=docs[index]
  useEffect(()=>{if(section)document.getElementById(`doc-${section}`)?.scrollIntoView({block:'start'})},[id,section])
  if(!doc)return <EmptyState title="Document not found" description="Choose a document from the navigation." action={<AppLink to="/getting-started">Getting started</AppLink>}/>
  const headings=doc.content.split('\n').filter(line=>/^## /.test(line)).map(line=>line.replace(/^## /,''))
  const route=basePath??`/docs/${id}`
  return <div className="documentation-page"><div className="docs-topline"><span>{doc.category}</span><span><FileCode2 size={13}/> Repository documentation · v0.5</span></div><div className="docs-layout"><article className="document"><PageHeading title={doc.title} description={doc.summary}/><ReactMarkdown remarkPlugins={[remarkGfm]} components={{
    h1:()=>null,
    h2:({children})=><h2 id={`doc-${headingId(textContent(children))}`}>{children}</h2>,
    h3:({children})=><h3 id={`doc-${headingId(textContent(children))}`}>{children}</h3>,
    a:({href='',children})=>href.startsWith('#')?<AppLink to={`${route}?section=${headingId(href.slice(1))}`}>{children}</AppLink>:<a href={docLink(href,id)} {...(href.startsWith('http')?{target:'_blank',rel:'noreferrer'}:{})}>{children}</a>,
    pre:({children})=><>{children}</>,
    code:({children,className})=>className||String(children).endsWith('\n')?<CodeBlock language={className?.replace('language-','')||'text'}>{String(children).replace(/\n$/,'')}</CodeBlock>:<code className="inline-code">{children}</code>,
    table:({children})=><div className="document-table"><table>{children}</table></div>,
  }}>{doc.content}</ReactMarkdown>{!standalone&&<div className="docs-pagination">{index>0?<AppLink to={`/docs/${docs[index-1].id}`}><ArrowLeft size={15}/><span><small>Previous</small>{docs[index-1].title}</span></AppLink>:<span/>}{index<docs.length-1&&<AppLink to={`/docs/${docs[index+1].id}`}><span><small>Next</small>{docs[index+1].title}</span><ArrowRight size={15}/></AppLink>}</div>}</article><aside className="docs-toc"><strong>On this page</strong>{headings.map((heading,i)=><AppLink key={i} to={`${route}?section=${headingId(heading)}`} className={section===headingId(heading)?'active':''}>{heading.replace(/`/g,'')}</AppLink>)}<div className="toc-note"><p>Explore the implementation behind this document.</p><Button variant="outline" size="sm" asChild><AppLink to="/reference">Code reference <ArrowRight size={13}/></AppLink></Button></div></aside></div></div>
}
