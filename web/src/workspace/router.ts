import { useSyncExternalStore } from 'react'

function subscribe(listener: () => void) {
  window.addEventListener('hashchange',listener)
  window.addEventListener('popstate',listener)
  return () => {window.removeEventListener('hashchange',listener);window.removeEventListener('popstate',listener)}
}
export function useRoute() {
  const hash=useSyncExternalStore(subscribe,() => window.location.hash,() => '#/')
  const [pathname,query='']=(hash.replace(/^#/,'') || '/').split('?')
  return {pathname,params:new URLSearchParams(query)}
}
export function routeHref(path: string) { return `#${path}` }
export function navigate(path:string,{replace=false}:{replace?:boolean}={}) {
  if(replace)window.history.replaceState(null,'',routeHref(path))
  else window.history.pushState(null,'',routeHref(path))
  window.dispatchEvent(new HashChangeEvent('hashchange'))
}
export function setRouteFilter(path:string,params:URLSearchParams,key:string,value:string) {
  const next=new URLSearchParams(params)
  if(value)next.set(key,value);else next.delete(key)
  navigate(path+(next.size ? `?${next}` : ''),{replace:true})
}
