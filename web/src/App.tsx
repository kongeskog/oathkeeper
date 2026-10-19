import { useLayoutEffect } from 'react'
import { TooltipProvider } from '@/components/ui/tooltip'
import { Shell } from './workspace/Shell'
import { Tax } from './workspace/Tax'
import { Workbench } from './workspace/Workbench'
import { Review } from './workspace/Review'
import { Home } from './workspace/Home'
import { Lawlib, RuleDetail, AuthorityDetail } from './workspace/Lawlib'
import { Cases, CaseDetail } from './workspace/Cases'
import { Sources } from './workspace/Sources'
import { Documentation } from './workspace/Docs'
import { ReferenceIndex, SourceReference } from './workspace/Reference'
import { AppLink, EmptyState } from './workspace/shared'
import { useRoute } from './workspace/router'

export default function App(){
  const {pathname,params}=useRoute()
  useLayoutEffect(()=>{window.scrollTo({top:0,behavior:'instant'});document.title=`${pathname==='/'?'Workspace':pathname.startsWith('/tax')?'Federal tax':pathname==='/workbench'?'Argument workbench':pathname==='/review'?'Review & evaluation':pathname.startsWith('/lawlib')?'Law library':pathname.startsWith('/cases')?'Case proofs':pathname.startsWith('/docs')?'Documentation':pathname.startsWith('/reference')?'Code reference':'Oathkeeper'} · Oathkeeper`},[pathname])
  let page:React.ReactNode
  const segments=pathname.split('/').filter(Boolean)
  let identifier=''
  try{identifier=decodeURIComponent(segments.at(-1)||'')}catch{/* Invalid URL gets the standard missing page. */}
  if(pathname==='/')page=<Home/>
  else if(pathname==='/tax/backdoor-roth')page=<Tax params={params}/>
  else if(pathname==='/workbench')page=<Workbench key={params.get('case')??'default'} slug={params.get('case')??undefined}/>
  else if(pathname==='/review')page=<Review/>
  else if(pathname==='/lawlib')page=<Lawlib params={params}/>
  else if(segments[0]==='lawlib'&&segments[1]==='rules'&&segments.length===3)page=<RuleDetail key={identifier} id={identifier}/>
  else if(segments[0]==='lawlib'&&segments[1]==='authorities'&&segments.length===3)page=<AuthorityDetail id={identifier}/>
  else if(pathname==='/cases')page=<Cases/>
  else if(segments[0]==='cases'&&segments.length===2)page=<CaseDetail key={`${identifier}:${params.get('step')??''}`} slug={identifier} step={params.get('step')??undefined}/>
  else if(pathname==='/sources')page=<Sources/>
  else if(pathname==='/docs')page=<Documentation id="getting-started" section={params.get('section')}/>
  else if(segments[0]==='docs'&&segments.length===2)page=<Documentation id={identifier} section={params.get('section')}/>
  else if(pathname==='/reference')page=<ReferenceIndex/>
  else if(segments[0]==='reference'&&segments.length===2)page=<SourceReference id={identifier} line={params.get('line')}/>
  else page=<EmptyState title="Page not found" description="Explore the law library or open the documentation to continue." action={<AppLink to="/">Return to overview</AppLink>}/>
  return <TooltipProvider delayDuration={250}><Shell pathname={pathname}>{page}</Shell></TooltipProvider>
}
