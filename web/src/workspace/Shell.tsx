import { Home, Library, BookOpen, Braces, Download, ArrowUpRight, Bookmark, Scale, Compass } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Separator } from '@/components/ui/separator'
import { Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbList, BreadcrumbPage, BreadcrumbSeparator } from '@/components/ui/breadcrumb'
import { Sidebar, SidebarProvider, SidebarHeader, SidebarContent, SidebarFooter, SidebarGroup, SidebarGroupLabel, SidebarGroupContent, SidebarMenu, SidebarMenuItem, SidebarMenuButton, SidebarMenuBadge, SidebarInset, SidebarTrigger, useSidebar } from '@/components/ui/sidebar'
import { AppLink } from './shared'
import { SearchDialog } from './Search'
import { useSavedRules } from './bookmarks'
import { lawModule } from './catalog'

const primary=[
  {path:'/',label:'Home',icon:Home,count:undefined},
  {path:'/getting-started',label:'Getting started',icon:BookOpen,count:undefined},
  {path:'/language',label:'Oath language',icon:Braces,count:undefined},
  {path:'/lawlib',label:'Lawlib explorer',icon:Library,count:lawModule.rules.length},
  {path:'/resources',label:'Resources',icon:Compass,count:undefined},
]

function WorkspaceSidebar({pathname}:{pathname:string}){
  const saved=useSavedRules()
  const {setOpenMobile}=useSidebar()
  const closeMobile=()=>setOpenMobile(false)
  const active=(path:string)=>path==='/'?pathname==='/':path==='/reference'||path==='/lawlib'?pathname.startsWith(path):pathname===path||pathname.startsWith(path+'/')
  return <Sidebar>
    <SidebarHeader><AppLink to="/" className="workspace-brand" onClick={closeMobile}><Scale/><span>Oathkeeper</span><Badge variant="outline">0.5</Badge></AppLink></SidebarHeader>
    <SidebarContent>
      <SidebarGroup><SidebarGroupLabel>Explore</SidebarGroupLabel><SidebarGroupContent><SidebarMenu>{primary.map(item=><SidebarMenuItem key={item.path}><SidebarMenuButton asChild isActive={active(item.path)}><AppLink to={item.path} aria-current={active(item.path)?'page':undefined} onClick={closeMobile}><item.icon/><span>{item.label}</span>{item.count!==undefined&&<SidebarMenuBadge>{item.count}</SidebarMenuBadge>}</AppLink></SidebarMenuButton></SidebarMenuItem>)}{saved.ids.length>0&&<SidebarMenuItem><SidebarMenuButton asChild><AppLink to="/lawlib?saved=1" onClick={closeMobile}><Bookmark/><span>Saved rules</span><SidebarMenuBadge>{saved.ids.length}</SidebarMenuBadge></AppLink></SidebarMenuButton></SidebarMenuItem>}</SidebarMenu></SidebarGroupContent></SidebarGroup>
    </SidebarContent>
    <SidebarFooter><Separator/><div className="sidebar-corpus"><strong>Open language and checker</strong><small>Version 0.5 · Apache 2.0</small></div><Button variant="outline" size="sm" asChild><a href="/assets/oathkeeper-0.5-source.zip" download><Download/>Download source</a></Button><p className="sidebar-license">Proofs are conditional on declared inputs.</p></SidebarFooter>
  </Sidebar>
}

export function Shell({pathname,children}:{pathname:string;children:React.ReactNode}){
  const section=pathname==='/getting-started'?'Getting started':pathname==='/language'?'Oath language':pathname.startsWith('/lawlib')?'Lawlib explorer':pathname==='/resources'?'Resources':pathname.startsWith('/tax')?'Federal tax example':pathname==='/workbench'?'Argument workbench':pathname==='/review'?'Review & evaluation':pathname.startsWith('/cases')?'Case proofs':pathname.startsWith('/sources')?'Source snapshots':pathname.startsWith('/docs')?'Documentation':pathname.startsWith('/reference')?'Code reference':'Home'
  return <SidebarProvider><a className="workspace-skip" href="#main-content" onClick={event=>{event.preventDefault();document.getElementById('main-content')?.focus()}}>Skip to content</a><WorkspaceSidebar pathname={pathname}/><SidebarInset className="min-w-0"><header className="app-topbar"><SidebarTrigger aria-label="Open navigation"/><Separator orientation="vertical"/><Breadcrumb><BreadcrumbList><BreadcrumbItem><BreadcrumbLink asChild><AppLink to="/">Oathkeeper</AppLink></BreadcrumbLink></BreadcrumbItem><BreadcrumbSeparator/><BreadcrumbItem><BreadcrumbPage>{section}</BreadcrumbPage></BreadcrumbItem></BreadcrumbList></Breadcrumb><SearchDialog/><Badge className="private-label" variant="outline">Open source</Badge></header><main id="main-content" className="app-main" tabIndex={-1}>{children}</main><footer className="app-footer"><span>Oathkeeper 0.5 · Checkable arguments with explicit inputs.</span><AppLink to="/resources">Resources <ArrowUpRight size={12}/></AppLink></footer></SidebarInset></SidebarProvider>
}
