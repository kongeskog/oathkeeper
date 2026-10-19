import { act, render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, beforeEach, describe, expect, test, vi } from 'vitest'
import { CommunityReview } from '../src/workspace/Community'
import { CoveragePanel } from '../src/workspace/Coverage'
import { contexts } from '../src/workspace/analysis'
class WorkerMock {
  static instances:WorkerMock[]=[]
  onmessage:((event:MessageEvent)=>void)|null=null
  onerror:(()=>void)|null=null
  terminate=vi.fn();postMessage=vi.fn()
  constructor(){WorkerMock.instances.push(this)}
}
beforeEach(()=>{WorkerMock.instances=[];vi.stubGlobal('Worker',WorkerMock)})
afterEach(()=>vi.unstubAllGlobals())
describe('community review and coverage',()=>{
  test('reviews use an exact artifact and edits discard the old result',async()=>{
    const user=userEvent.setup();render(<CommunityReview/>)
    expect(screen.getByRole('link',{name:'Lawlib repository'})).toHaveAttribute('href','https://github.com/kongeskog/oathkeeper-lawlib')
    await user.click(screen.getByRole('button',{name:'Verify signed community reviews locally'}))
    await user.click(screen.getByRole('button',{name:'Verify review journal'}))
    const old=WorkerMock.instances.at(-1)!
    const request=JSON.parse(old.postMessage.mock.calls[0][0].request)
    expect(request.operation).toBe('reviews');expect(request.artifact_sha256).toMatch(/^[a-f0-9]{64}$/)
    act(()=>old.onmessage?.({data:{result:{current:[],stale_events:0}}} as MessageEvent))
    expect(screen.getByText('Journal checked for this artifact')).toBeVisible()
    await user.type(screen.getByLabelText('Signed journal JSON'),' ')
    expect(screen.queryByText('Journal checked for this artifact')).not.toBeInTheDocument()
    act(()=>old.onmessage?.({data:{result:{current:['stale']}}} as MessageEvent))
    expect(screen.queryByText('Journal checked for this artifact')).not.toBeInTheDocument()
  })
  test('signature failures remain failures',async()=>{
    const user=userEvent.setup();render(<CommunityReview/>)
    await user.click(screen.getByRole('button',{name:'Verify signed community reviews locally'}))
    await user.click(screen.getByRole('button',{name:'Verify review journal'}))
    act(()=>WorkerMock.instances.at(-1)!.onmessage?.({data:{error:'[E_SIGNATURE] receipt signature is invalid'}} as MessageEvent))
    expect(screen.getByText('Review verification failed')).toBeVisible()
    expect(screen.queryByText('Journal checked for this artifact')).not.toBeInTheDocument()
  })
  test('coverage preserves unknown scope instead of showing general legal approval',()=>{
    const coverage=contexts.find(c=>c.baseline.coverage?.contract)?.baseline.coverage!
    render(<CoveragePanel coverage={{...coverage,status:'needs_information'}}/>)
    expect(screen.getByTestId('coverage-status')).toHaveTextContent('Scope needs more information')
    expect(screen.queryByText('Action is legal')).not.toBeInTheDocument()
  })
})
