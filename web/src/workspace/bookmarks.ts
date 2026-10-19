import { useSyncExternalStore } from 'react'
const key='oathkeeper.saved-rules.v1'
function subscribe(callback:()=>void){window.addEventListener('storage',callback);window.addEventListener('oathkeeper-saved',callback);return()=>{window.removeEventListener('storage',callback);window.removeEventListener('oathkeeper-saved',callback)}}
function snapshot(){try{return localStorage.getItem(key)??'[]'}catch{return '[]'}}
export function useSavedRules(){
  const raw=useSyncExternalStore(subscribe,snapshot,()=> '[]')
  let ids:string[]=[]
  try{const parsed:unknown=JSON.parse(raw);if(Array.isArray(parsed))ids=parsed.filter((item):item is string=>typeof item==='string')}catch{/* Ignore invalid browser-local preferences. */}
  function toggle(id:string){const next=ids.includes(id)?ids.filter(value=>value!==id):[...ids,id];try{localStorage.setItem(key,JSON.stringify(next));window.dispatchEvent(new Event('oathkeeper-saved'))}catch{/* Browser storage can be disabled. */}}
  return {ids,toggle}
}
