import type { CoverageResult } from './Coverage'
import { useEffect, useState } from 'react'
import data from '@/data/workbench-cases.json'
import type { Claim, Fact, ProofStep, Term } from '@/types'

export type Support='supported'|'opposed'|'both'|'neither'
export const supportLabel:Record<Support,string>={supported:'Support found',opposed:'Opposition found',both:'Conflicting arguments',neither:'Unresolved'}
export interface OraclePolicy {oracle:string;atom:string;question:string;evidence_sha256:string;arguments:Term[];providers:{id:string;version:string}[];protocol:{role:string;agreement:string;procedure:string}}
export interface Context {facts:Fact[];proof:ProofStep[];retractions?:{target_kind:string;target:string}[];oracle_policy:OraclePolicy[];attestations:unknown[];query:{goal:Claim;root:string;event_at:string;as_of:string;authority_at:string;known_at:string;purpose:string;temporal_rationale:string;forum:string};[key:string]:unknown}
export interface Argument {proof:ProofStep[];report:{goal:Claim;fact_assumptions:Fact[];oracle_count:number;assurance:Record<string,unknown>;[key:string]:unknown}}
export interface Analysis {exception_checks?:{goal:Claim;status:string}[];exception_checks_complete?:boolean;coverage?:CoverageResult;kernel_version:string;context_sha256:string;status:Support;goal:Claim;supporting:Argument[];opposing:Argument[];open_requirements:{claim:Claim;needed_by:string[];oracle_requests:string[]}[];excluded_rules:{rule:string;version:string;reason:string}[];oracle_requests:{request:string;role:string;agreement:string;status:string;authentication:string}[];search:{complete_for_ground_domain:boolean;ground_rules:number;nodes_visited:number;limits_reached:string[];meaning:string};sensitivity:{kind:string;id:string;without:Support;search_complete:boolean}[];sensitivity_complete:boolean;legal_conclusion:string}
export const contexts=data as unknown as {slug:string;title:string;bundle:Context;baseline:Analysis}[]
export const DEFAULT_CASE='long-parking-evidence-review'

export function withoutEvidence(source:Context,excluded:string[]):Context {
  return {...source,proof:[],facts:source.facts.filter(f=>!excluded.includes(f.id)),retractions:source.retractions?.filter(r=>r.target_kind!=='fact'||!excluded.includes(r.target))}
}
export function useAnalysis(context:Context,attempt:number) {
  const [state,setState]=useState<{key:string;result?:Analysis;error?:string}>({key:''})
  const key=JSON.stringify(context)
  useEffect(()=>{
    let active=true
    let worker:Worker|undefined
    const timeout=setTimeout(()=>{worker?.terminate();if(active)setState({key,error:'Analysis exceeded 20 seconds. Reduce the context or run the CLI with explicit limits.'})},20000)
    setState({key})
    try{
      worker=new Worker('/assets/analysis-worker.js?v=0.5.0',{type:'module'})
      worker.onmessage=(event:MessageEvent<{result?:Analysis;error?:string}>)=>{if(active){clearTimeout(timeout);setState({key,...event.data});worker?.terminate()}}
      worker.onerror=()=>{if(active){clearTimeout(timeout);setState({key,error:'The Rust analysis worker could not start. Reload or use the downloadable context with the CLI.'});worker?.terminate()}}
      worker.postMessage({context:key,sensitivity:true})
    }catch(error){clearTimeout(timeout);setState({key,error:error instanceof Error?error.message:'Analysis unavailable.'})}
    return()=>{active=false;clearTimeout(timeout);worker?.terminate()}
  },[key,attempt])
  return state.key===key?state:{key}
}
export function downloadJson(value:unknown,name:string) {
  const url=URL.createObjectURL(new Blob([JSON.stringify(value,null,2)+'\n'],{type:'application/json'}))
  const link=document.createElement('a');link.href=url;link.download=name;link.click();setTimeout(()=>URL.revokeObjectURL(url),1000)
}
