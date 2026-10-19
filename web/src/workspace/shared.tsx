import { useState } from 'react'
import { Check, Copy, ArrowDown, ArrowUpRight, CircleDashed, Info } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Card, CardHeader, CardTitle, CardDescription } from '@/components/ui/card'
import { Alert, AlertTitle, AlertDescription } from '@/components/ui/alert'
import { prettyJson } from '@/lib/proof-model'
import { routeHref } from './router'

export function AppLink({to,children,...props}:{to:string;children:React.ReactNode}&React.AnchorHTMLAttributes<HTMLAnchorElement>) {
  return <a href={routeHref(to)} {...props}>{children}</a>
}
export function ReviewBadge(){return <Badge variant="outline" className="status-review tracking-normal"><CircleDashed/>Needs review</Badge>}
export function PageHeading({eyebrow,title,description,actions}:{eyebrow?:string;title:string;description?:string;actions?:React.ReactNode}) {
  return <div className="page-heading"><div>{eyebrow&&<div className="page-eyebrow">{eyebrow}</div>}<h1>{title}</h1>{description&&<p>{description}</p>}</div>{actions&&<div className="page-actions">{actions}</div>}</div>
}
export function CopyButton({text,label='Copy',className=''}:{text:string;label?:string;className?:string}) {
  const [copied,setCopied]=useState(false)
  const [error,setError]=useState(false)
  async function copy(){try{await navigator.clipboard.writeText(text);setCopied(true);setError(false);setTimeout(()=>setCopied(false),2000)}catch{setError(true)}}
  return <span className="copy-control"><Button type="button" variant="ghost" size="sm" className={className} onClick={copy} aria-label={`${label} to clipboard`}>{copied?<Check size={13}/>:<Copy size={13}/>}<span>{copied?'Copied':label}</span></Button>{error&&<span role="status" className="copy-error">Select the text to copy.</span>}</span>
}
export function JsonView({value,label='JSON'}:{value:unknown;label?:string}) {
  const text=JSON.stringify(value,null,2)
  return <div className="code-block"><div className="code-block-toolbar"><span>{label}</span><CopyButton text={text}/></div><pre><code dangerouslySetInnerHTML={{__html:prettyJson(value)}}/></pre></div>
}
export function CodeBlock({children,language='shell'}:{children:string;language?:string}) {
  return <div className="code-block"><div className="code-block-toolbar"><span>{language}</span><CopyButton text={children}/></div><pre><code>{children}</code></pre></div>
}
export function EmptyState({title,description,action}:{title:string;description:string;action?:React.ReactNode}) {
  return <div className="empty-state"><div className="empty-glyph">∅</div><h3>{title}</h3><p>{description}</p>{action}</div>
}
export function Notice({children,title='Historical, unreviewed formalization'}:{children:React.ReactNode;title?:string}) {
  return <Alert className="notice"><Info/><AlertTitle>{title}</AlertTitle><AlertDescription>{children}</AlertDescription></Alert>
}
export function DownloadButton({href,label='Download',download=true}:{href:string;label?:string;download?:boolean}) {
  return <Button variant="outline" size="sm" asChild><a href={href} download={download||undefined}>{download?<ArrowDown size={14}/>:<ArrowUpRight size={14}/>} {label}</a></Button>
}
export function DataCard({label,value,detail}:{label:string;value:React.ReactNode;detail?:string}) {
  return <Card className="data-card"><CardHeader><CardDescription>{label}</CardDescription><CardTitle>{value}</CardTitle>{detail&&<CardDescription>{detail}</CardDescription>}</CardHeader></Card>
}
