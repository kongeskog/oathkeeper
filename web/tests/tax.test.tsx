import { beforeEach, afterEach, describe, test, expect, vi } from 'vitest'
import { act, render, screen, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import App from '../src/App'
import { navigate } from '../src/workspace/router'
import { parseDollars, taxData, type TaxResult } from '../src/workspace/tax'

class WorkerMock {
  static instances:WorkerMock[]=[]
  onmessage:((event:MessageEvent)=>void)|null=null
  onerror:(()=>void)|null=null
  terminate=vi.fn()
  postMessage=vi.fn()
  constructor(){WorkerMock.instances.push(this)}
  deliver(result:TaxResult){this.onmessage?.({data:{result}} as MessageEvent)}
}
function open(route='/tax/backdoor-roth'){act(()=>navigate(route));return render(<App/>)}
function current(){return WorkerMock.instances.at(-1)!}
beforeEach(()=>{WorkerMock.instances=[];vi.stubGlobal('Worker',WorkerMock)})
afterEach(()=>vi.unstubAllGlobals())

describe('bounded federal tax application',()=>{
  test('starts empty and sends unknown amounts without substituting zero',async()=>{
    const user=userEvent.setup();open()
    expect(WorkerMock.instances).toHaveLength(0)
    expect(screen.getByLabelText('Prior nondeductible IRA basis')).toHaveValue('')
    await user.click(screen.getByRole('button',{name:'Calculate with Rust / WASM'}))
    const input=JSON.parse(current().postMessage.mock.calls[0][0].input)
    expect(input.prior_basis_cents).toBeNull();expect(input.scope_confirmed).toBeNull();expect(input.conversion_year).toBe(2025)
    act(()=>current().deliver({...taxData.fixtures[0].baseline,status:'needs_information',allocation:null,certificate:null,verification:null,issues:[{field:'prior_basis_cents',message:'Required input is unknown.'}]}))
    expect(screen.getByTestId('tax-status')).toHaveTextContent('More information needed')
    expect(screen.queryByText('Taxable conversion estimate')).not.toBeInTheDocument()
  })
  test('synthetic scenario runs the worker and displays conditional pro-rata amounts',async()=>{
    const user=userEvent.setup();open()
    await user.click(screen.getByRole('button',{name:'Compare pretax balance'}))
    await user.click(screen.getByRole('button',{name:'Calculate with Rust / WASM'}))
    const input=JSON.parse(current().postMessage.mock.calls[0][0].input)
    expect(input.dec31_ira_value_cents).toBe(6300000);expect(input.purpose).toBe('synthetic_test')
    act(()=>current().deliver(taxData.fixtures[1].baseline))
    expect(within(screen.getByText('Taxable conversion estimate').closest('[data-slot="card"]') as HTMLElement).getByText('$6,300.00')).toBeVisible()
    expect(screen.getByTestId('tax-status')).toHaveTextContent('Conditional derivation checked')
    expect(screen.getByRole('button',{name:'Proof certificate'})).toBeVisible()
    await user.click(screen.getByRole('button',{name:/Inspect \d+ proof steps/}))
    expect(screen.getAllByText('compute').length).toBeGreaterThan(0)
  })
  test('edits invalidate amounts and discard stale worker replies',async()=>{
    const user=userEvent.setup();open()
    await user.click(screen.getByRole('button',{name:'Load synthetic example'}));await user.click(screen.getByRole('button',{name:'Calculate with Rust / WASM'}))
    const old=current();act(()=>old.deliver(taxData.fixtures[0].baseline))
    expect(screen.getByTestId('tax-status')).toBeVisible()
    await user.clear(screen.getByLabelText('Prior nondeductible IRA basis'))
    expect(screen.queryByTestId('tax-status')).not.toBeInTheDocument();expect(old.terminate).toHaveBeenCalled()
    act(()=>old.deliver(taxData.fixtures[0].baseline));expect(screen.queryByTestId('tax-status')).not.toBeInTheDocument()
    await user.click(screen.getByRole('button',{name:'Calculate with Rust / WASM'}))
    expect(JSON.parse(current().postMessage.mock.calls[0][0].input).prior_basis_cents).toBeNull()
  })
  test('invalid fractional input is not rounded or sent to the worker',async()=>{
    const user=userEvent.setup();open();await user.type(screen.getByLabelText('Prior nondeductible IRA basis'),'10.001')
    await user.click(screen.getByRole('button',{name:'Calculate with Rust / WASM'}))
    expect(screen.getByText(/Prior nondeductible IRA basis: Enter a nonnegative/)).toBeVisible()
    expect(WorkerMock.instances).toHaveLength(0)
    expect(parseDollars('0.29')).toBe(29);expect(parseDollars('')).toBeNull();expect(()=>parseDollars('-1')).toThrow();expect(()=>parseDollars('1e4')).toThrow()
  })
  test('rules expose canonical code and form pinpoints; tabs preserve the scenario',async()=>{
    const user=userEvent.setup();open();await user.type(screen.getByLabelText('Prior nondeductible IRA basis'),'123')
    await user.click(screen.getByRole('tab',{name:'Rules (10)'}));await user.type(screen.getByRole('textbox',{name:'Search tax rules'}),'available-basis')
    const button=screen.getByRole('button',{name:/us.tax.2025.available-basis/});await user.click(button)
    expect(screen.getByRole('tabpanel')).toHaveTextContent('oathkeeper.quantity.subtract')
    await user.click(screen.getByRole('tab',{name:'Calculator'}));expect(screen.getByLabelText('Prior nondeductible IRA basis')).toHaveValue('123')
  })
  test('real precedent exposes the official opinion and two paths for one matter',()=>{
    open('/tax/backdoor-roth?view=precedent')
    expect(screen.getByText('Shank v. Commissioner')).toBeVisible()
    expect(screen.getByRole('link',{name:'Official opinion PDF'})).toHaveAttribute('href','/assets/tax/sources/shank-2018-official.pdf')
    expect(screen.getAllByRole('link',{name:'Replayable bundle'})).toHaveLength(2)
    expect(screen.getByText(/The opinion does not establish that Shank never filed/)).toBeVisible()
  })
  test('worker failures permit another calculation and do not display results',async()=>{
    const user=userEvent.setup();open();await user.click(screen.getByRole('button',{name:'Load synthetic example'}));await user.click(screen.getByRole('button',{name:'Calculate with Rust / WASM'}))
    act(()=>current().onerror?.());expect(screen.getByText(/The Rust WebAssembly worker could not start/)).toBeVisible()
    expect(screen.queryByTestId('tax-status')).not.toBeInTheDocument()
    await user.click(screen.getByRole('button',{name:'Calculate with Rust / WASM'}));expect(WorkerMock.instances).toHaveLength(2)
  })
  test('review route names the scope and independent reviews without claiming approval',()=>{
    open('/tax/backdoor-roth?view=review')
    expect(screen.getByText('Independent reviews are pending')).toBeVisible()
    expect(within(screen.getByRole('tabpanel')).getByText(/No automatic opposing-argument search/)).toBeVisible()
    expect(screen.getByRole('link',{name:'Tax review packet'})).toHaveAttribute('href','/assets/tax/review-packet.json')
  })
})
