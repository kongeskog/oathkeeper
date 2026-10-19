import type { CoverageResult } from './Coverage'
import { useEffect, useState } from 'react'
import data from '@/data/tax.json'
import type { CaseRecord, LawModule, ProofStep } from '@/types'

export interface TaxInput {
  tax_year:number; conversion_year:number; recorded_at:string; purpose:'new_matter_assessment'|'synthetic_test'; scope_confirmed:boolean|null
  age_at_year_end:number|null; taxable_compensation_cents:number|null; nondeductible_contribution_cents:number|null
  contribution_made_in_2026_cents:number|null; other_regular_ira_contributions_cents:number|null; prior_basis_cents:number|null
  dec31_ira_value_cents:number|null; roth_conversion_cents:number|null
}
export interface TaxResult {
  coverage?:CoverageResult|null
  status:'calculated'|'needs_information'|'outside_scope'; module_snapshot_sha256:string; module_version:string; scope:string; rounding:string; review_status:string
  issues:{field:string;message:string}[]
  allocation:null|{annual_limit_cents:number;total_basis_cents:number;available_basis_cents:number;aggregate_pool_cents:number;fraction_numerator_cents:number;fraction_denominator_cents:number;nontaxable_conversion_cents:number;taxable_conversion_cents:number;remaining_basis_cents:number}
  certificate:null|{proof:ProofStep[];[key:string]:unknown}
  verification:null|{status:string;kernel_version:string;oracle_count:number;fact_assumptions:unknown[];[key:string]:unknown}
}
export const taxData=data as unknown as {
  module:LawModule;rule_sources:Record<string,string>
  sources:{id:string;file:string;citation:string;uri:string;sha256:string;retrieved_at:string;provenance:string}[]
  review:{snapshot_sha256:string;precedent_snapshot_sha256:string;criteria:string[];limitations:string[]}
  cases:{name:string;bundle:CaseRecord['bundle'];report:CaseRecord['report']}[]
  fixtures:{name:string;input:TaxInput;baseline:TaxResult}[]
}
export const dollar=(cents:number)=>new Intl.NumberFormat('en-US',{style:'currency',currency:'USD',minimumFractionDigits:2}).format(cents/100)
export function parseDollars(value:string):number|null {
  if(!value.trim())return null
  if(!/^\d+(\.\d{1,2})?$/.test(value.trim()))throw new Error('Enter a nonnegative dollar amount with at most two decimal places.')
  const [whole,fraction='']=value.trim().split('.')
  const cents=Number(whole)*100+Number(fraction.padEnd(2,'0'))
  if(!Number.isSafeInteger(cents)||cents>1_000_000_000_000)throw new Error('Amount exceeds this pilot’s supported range.')
  return cents
}
export function useTaxAssessment(request:string|null) {
  const [state,setState]=useState<{key:string|null;result?:TaxResult;error?:string}>({key:null})
  useEffect(()=>{
    if(!request)return
    let active=true
    let worker:Worker|undefined
    const timeout=setTimeout(()=>{worker?.terminate();if(active)setState({key:request,error:'Calculation exceeded 20 seconds. Retry or use the CLI.'})},20000)
    setState({key:request})
    try {
      worker=new Worker('/assets/tax-worker.js?v=0.5.0',{type:'module'})
      worker.onmessage=(event:MessageEvent<{result?:TaxResult;error?:string}>)=>{if(active){clearTimeout(timeout);setState({key:request,...event.data});worker?.terminate()}}
      worker.onerror=()=>{if(active){clearTimeout(timeout);setState({key:request,error:'The Rust WebAssembly worker could not start. Retry or run the downloadable scenario with the CLI.'});worker?.terminate()}}
      worker.postMessage({input:request})
    } catch(error) {clearTimeout(timeout);setState({key:request,error:error instanceof Error?error.message:'Calculation unavailable.'})}
    return()=>{active=false;clearTimeout(timeout);worker?.terminate()}
  },[request])
  return state.key===request&&request?state:{key:request}
}
