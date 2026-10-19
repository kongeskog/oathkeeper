import { beforeEach, afterEach, describe, test, expect, vi } from 'vitest'
import { act, render, screen, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import App from '../src/App'
import { navigate } from '../src/workspace/router'
import { contexts, DEFAULT_CASE, withoutEvidence, type Analysis } from '../src/workspace/analysis'

class WorkerMock {
  static instances:WorkerMock[]=[]
  onmessage:((event:MessageEvent)=>void)|null=null
  onerror:(()=>void)|null=null
  terminate=vi.fn()
  postMessage=vi.fn()
  constructor(){WorkerMock.instances.push(this)}
  deliver(result:Analysis){this.onmessage?.({data:{result}} as MessageEvent)}
}
function open(route:string){act(()=>navigate(route));return render(<App/>)}
function current(){return WorkerMock.instances.at(-1)!}
beforeEach(()=>{WorkerMock.instances=[];vi.stubGlobal('Worker',WorkerMock)})
afterEach(()=>vi.unstubAllGlobals())

describe('live argument workbench',()=>{
  test('starts with the actual incomplete record and exposes all four clocks',()=>{
    open('/workbench')
    const payload=JSON.parse(current().postMessage.mock.calls[0][0].context)
    const record=contexts.find(c=>c.slug===DEFAULT_CASE)!
    expect(payload.proof).toEqual([])
    expect(payload.facts).toEqual(record.bundle.facts)
    expect(screen.getByText('2016-10-12')).toBeVisible()
    expect(screen.getByText('2016-10-12T00:00:00-07:00')).toBeVisible()
    expect(screen.getByText('2021-08-13T00:00:00Z')).toBeVisible()
    act(()=>current().deliver(record.baseline))
    expect(screen.getByTestId('live-status')).toHaveTextContent('Unresolved')
    expect(screen.queryByText('Verified legal conclusion')).not.toBeInTheDocument()
  })
  test('withholding evidence invalidates the old result and discards stale worker replies',async()=>{
    const user=userEvent.setup();open('/workbench?case=long-excessive-fines')
    const record=contexts.find(c=>c.slug==='long-excessive-fines')!
    const old=current()
    act(()=>old.deliver(record.baseline))
    expect(screen.getByTestId('live-status')).toHaveTextContent('Support found')
    await user.click(screen.getByRole('switch',{name:'Include disproportionality-finding'}))
    expect(screen.queryByTestId('live-status')).not.toBeInTheDocument()
    const payload=JSON.parse(current().postMessage.mock.calls[0][0].context)
    expect(payload.facts.some((f:{id:string})=>f.id==='disproportionality-finding')).toBe(false)
    expect(old.terminate).toHaveBeenCalled()
    act(()=>old.deliver(record.baseline))
    expect(screen.queryByTestId('live-status')).not.toBeInTheDocument()
    expect(screen.getByText('Hypothetical evidence withdrawal')).toBeVisible()
  })
  test('pending oracle and evidence gaps are inspectable without a fabricated response',async()=>{
    const user=userEvent.setup();open('/workbench?case=long-oracle-reassessment')
    act(()=>current().deliver(contexts.find(c=>c.slug==='long-oracle-reassessment')!.baseline))
    await user.click(screen.getByRole('tab',{name:'Oracles (1)'}))
    expect(screen.getByRole('tabpanel')).toHaveTextContent('pending')
    expect(screen.getByRole('tabpanel')).toHaveTextContent('provider identity not authenticated')
    await user.click(screen.getByRole('tab',{name:/Evidence gaps/}))
    expect(screen.getByRole('tabpanel')).toHaveTextContent('sanction.grossly_disproportionate')
  })
  test('worker failure is explicit and retry starts a fresh computation',async()=>{
    const user=userEvent.setup();open('/workbench')
    act(()=>current().onerror?.())
    expect(screen.getByText('Analysis unavailable')).toBeVisible()
    await user.click(screen.getByRole('button',{name:'Retry analysis'}))
    expect(WorkerMock.instances).toHaveLength(2)
    expect(screen.queryByText('Analysis unavailable')).not.toBeInTheDocument()
  })
  test('withholding a fact removes only its retractions and preserves other evidence history',()=>{
    const source=contexts[0].bundle
    const id=source.facts[0].id
    const result=withoutEvidence({...source,retractions:[{target_kind:'fact',target:id},{target_kind:'attestation',target:id}]},[id])
    expect(result.retractions).toEqual([{target_kind:'attestation',target:id}])
    expect(source.facts.some(f=>f.id===id)).toBe(true)
  })
  test('review page distinguishes passing engineering fixtures from unmeasured legal outcomes',async()=>{
    const user=userEvent.setup();open('/review')
    expect(screen.getByRole('heading',{name:'Review the formalization'})).toBeVisible()
    expect(screen.getAllByText('Pending')).toHaveLength(2)
    expect(screen.getByRole('link',{name:'Historical pilot notes'})).toHaveAttribute('href','https://github.com/kongeskog/oathkeeper-lawlib/blob/main/docs/SEATTLE-PILOT.md')
    expect(screen.getByText(/Reviewer time, legal agreement and held-out accuracy have not been measured/)).toBeVisible()
    const item=screen.getByRole('button',{name:'us.timbs.incorporation · v1'})
    await user.click(item)
    expect(screen.getByText('explicit applicability assumption')).toBeVisible()
    expect(screen.getByRole('link',{name:'Review packet'})).toHaveAttribute('href','/assets/review-packet.json')
  })
})
