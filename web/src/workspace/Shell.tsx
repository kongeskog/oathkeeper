import { Home, Library, Files, Fingerprint, BookOpen, Braces, Map, Download, ArrowUpRight, GitBranch, Bookmark, Scale } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Separator } from '@/components/ui/separator'
import { Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbList, BreadcrumbPage, BreadcrumbSeparator } from '@/components/ui/breadcrumb'
import { Sidebar, SidebarProvider, SidebarHeader, SidebarContent, SidebarFooter, SidebarGroup, SidebarGroupLabel, SidebarGroupContent, SidebarMenu, SidebarMenuItem, SidebarMenuButton, SidebarMenuBadge, SidebarInset, SidebarTrigger, useSidebar } from '@/components/ui/sidebar'
import { AppLink } from './shared'
import { SearchDialog } from './Search'
import { useSavedRules } from './bookmarks'
import { lawModule, records, snapshots } from './catalog'

const primary=[
  {path:'/',label:'Overview',icon:Home,count:undefined},
  {path:'/tax/backdoor-roth',label:'Federal tax',icon:Scale,count:undefined},
  {path:'/workbench',label:'Argument workbench',icon:Scale,count:undefined},
  {path:'/review',label:'Review & evaluation',icon:GitBranch,count:undefined},
  {path:'/lawlib',label:'Law library',icon:Library,count:lawModule.rules.length},
  {path:'/cases',label:'Case proofs',icon:Files,count:records.length},
  {path:'/sources',label:'Source snapshots',icon:Fingerprint,count:snapshots.length},
]
const reference=[
  {path:'/docs/getting-started',label:'Getting started',icon:BookOpen},
  {path:'/docs/tax',label:'Tax module guide',icon:BookOpen},
  {path:'/docs/language',label:'Language reference',icon:Braces},
  {path:'/reference',label:'Code reference',icon:Braces},
  {path:'/docs/oracles',label:'Oracle protocols',icon:Fingerprint},
  {path:'/docs/architecture',label:'Trust boundaries',icon:GitBranch},
  {path:'/docs/python',label:'Python API',icon:BookOpen},
  {path:'/docs/cli',label:'CLI reference',icon:BookOpen},
  {path:'/docs/roadmap',label:'Coverage & roadmap',icon:Map},
]

function WorkspaceSidebar({pathname}:{pathname:string}){
  const saved=useSavedRules()
  const {setOpenMobile}=useSidebar()
  const closeMobile=()=>setOpenMobile(false)
  const active=(path:string)=>path==='/'?pathname==='/':path==='/reference'||path==='/lawlib'?pathname.startsWith(path):pathname===path||pathname.startsWith(path+'/')
  return <Sidebar>
    <SidebarHeader><AppLink to="/" className="workspace-brand" onClick={closeMobile}><Scale/><span>Oathkeeper</span><Badge variant="outline">0.4</Badge></AppLink></SidebarHeader>
    <SidebarContent>
      <SidebarGroup><SidebarGroupLabel>Workspace</SidebarGroupLabel><SidebarGroupContent><SidebarMenu>{primary.map(item=><SidebarMenuItem key={item.path}><SidebarMenuButton asChild isActive={active(item.path)}><AppLink to={item.path} aria-current={active(item.path)?'page':undefined} onClick={closeMobile}><item.icon/><span>{item.label}</span>{item.count!==undefined&&<SidebarMenuBadge>{item.count}</SidebarMenuBadge>}</AppLink></SidebarMenuButton></SidebarMenuItem>)}{saved.ids.length>0&&<SidebarMenuItem><SidebarMenuButton asChild><AppLink to="/lawlib?saved=1" onClick={closeMobile}><Bookmark/><span>Saved rules</span><SidebarMenuBadge>{saved.ids.length}</SidebarMenuBadge></AppLink></SidebarMenuButton></SidebarMenuItem>}</SidebarMenu></SidebarGroupContent></SidebarGroup>
      <SidebarGroup><SidebarGroupLabel>Documentation</SidebarGroupLabel><SidebarGroupContent><SidebarMenu>{reference.map(item=><SidebarMenuItem key={item.path}><SidebarMenuButton asChild isActive={active(item.path)}><AppLink to={item.path} aria-current={active(item.path)?'page':undefined} onClick={closeMobile}><item.icon/><span>{item.label}</span></AppLink></SidebarMenuButton></SidebarMenuItem>)}</SidebarMenu></SidebarGroupContent></SidebarGroup>
    </SidebarContent>
    <SidebarFooter><Separator/><div className="sidebar-corpus"><strong>Seattle + U.S. federal tax</strong><small>Versioned modules · review pending</small></div><Button variant="outline" size="sm" asChild><a href="/assets/oathkeeper-0.5-source.zip" download><Download/>Download Oathkeeper</a></Button><p className="sidebar-license">Open specification · Apache 2.0 code</p></SidebarFooter>
  </Sidebar>
}

export function Shell({pathname,children}:{pathname:string;children:React.ReactNode}){
  const section=pathname.startsWith('/tax')?'Federal tax':pathname==='/workbench'?'Argument workbench':pathname==='/review'?'Review & evaluation':pathname.startsWith('/lawlib')?'Law library':pathname.startsWith('/cases')?'Case proofs':pathname.startsWith('/sources')?'Source snapshots':pathname.startsWith('/docs')?'Documentation':pathname.startsWith('/reference')?'Code reference':'Overview'
  return <SidebarProvider><a className="workspace-skip" href="#main-content" onClick={event=>{event.preventDefault();document.getElementById('main-content')?.focus()}}>Skip to content</a><WorkspaceSidebar pathname={pathname}/><SidebarInset className="min-w-0"><header className="app-topbar"><SidebarTrigger aria-label="Open navigation"/><Separator orientation="vertical"/><Breadcrumb><BreadcrumbList><BreadcrumbItem><BreadcrumbLink asChild><AppLink to="/">Oathkeeper</AppLink></BreadcrumbLink></BreadcrumbItem><BreadcrumbSeparator/><BreadcrumbItem><BreadcrumbPage>{section}</BreadcrumbPage></BreadcrumbItem></BreadcrumbList></Breadcrumb><SearchDialog/><Badge className="private-label" variant="outline">Research workspace</Badge></header><main id="main-content" className="app-main" tabIndex={-1}>{children}</main><footer className="app-footer"><span>Oathkeeper 0.4 · A derivation is conditional on its declared inputs.</span><AppLink to="/docs/architecture">Trust boundaries <ArrowUpRight size={12}/></AppLink></footer></SidebarInset></SidebarProvider>
}
