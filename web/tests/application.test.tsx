import { describe, test, expect, vi } from 'vitest'
import { act, render, screen, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { readFile, access } from 'node:fs/promises'
import path from 'node:path'
import App from '../src/App'
import { navigate } from '../src/workspace/router'
import { lawModule, records, snapshots, rulePath, ruleTitle, filterRules, sourceAsset, verifySnapshot } from '../src/workspace/catalog'
import { codeSymbols, sourceFiles } from '../src/workspace/documentation'
import { traceDependencies, applicationSteps, escapeHtml } from '../src/lib/proof-model'

function open(route:string){act(()=>navigate(route));return render(<App/>)}

describe('law library application',()=>{
  test('a focused overview links to separate application pages',async()=>{
    const user=userEvent.setup();open('/')
    expect(screen.getAllByRole('heading',{level:1})).toHaveLength(1)
    expect(screen.queryByTestId('proof-detail')).not.toBeInTheDocument()
    await user.click(screen.getByRole('link',{name:'Open law library'}))
    expect(await screen.findByRole('heading',{name:'Law library'})).toBeVisible()
    expect(window.location.hash).toBe('#/lawlib')
    expect(screen.getByText('Showing 14 of 14 rules')).toBeVisible()
  })
  test('search filters actual rules and persists in the URL',async()=>{
    const user=userEvent.setup();open('/lawlib')
    await user.type(screen.getByRole('textbox',{name:'Search law library'}),'homestead')
    expect(screen.getByText('Showing 2 of 14 rules')).toBeVisible()
    expect(window.location.hash).toContain('q=homestead')
    await user.clear(screen.getByRole('textbox',{name:'Search law library'}))
    await user.type(screen.getByRole('textbox',{name:'Search law library'}),'no such rule')
    expect(screen.getByRole('heading',{name:'No matching rules'})).toBeVisible()
    await user.click(screen.getByRole('link',{name:'Clear filters'}))
    expect(await screen.findByText('Showing 14 of 14 rules')).toBeVisible()
  })
  test('jurisdiction and authority-type filters compose on a deep link',async()=>{
    const user=userEvent.setup();open('/lawlib?jurisdiction=us.wa.seattle&kind=ordinance')
    expect(screen.getByText('Showing 7 of 14 rules')).toBeVisible()
    await user.click(screen.getByRole('button',{name:'United States 1'}))
    expect(screen.getByRole('heading',{name:'No matching rules'})).toBeVisible()
    await user.click(screen.getByRole('combobox',{name:'Filter authority type'}))
    await user.click(await screen.findByRole('option',{name:'Case holdings'}))
    expect(screen.getByText('Showing 1 of 14 rules')).toBeVisible()
  })
  test('rules open exact JSON and retain browser-local bookmarks',async()=>{
    const user=userEvent.setup();const rule=lawModule.rules[0];open(rulePath(rule.id))
    expect(screen.getByRole('heading',{name:ruleTitle(rule),level:1})).toBeVisible()
    await user.click(screen.getByRole('button',{name:'Save rule'}))
    expect(screen.getByRole('button',{name:'Saved'})).toHaveAttribute('aria-pressed','true')
    await user.click(screen.getByRole('tab',{name:'Compiled JSON'}))
    expect(screen.getByRole('tabpanel')).toHaveTextContent(rule.source.sha256)
    expect(screen.getByRole('tabpanel')).toHaveTextContent('effective_from')
    await user.click(screen.getByRole('link',{name:'Saved rules 1'}))
    expect(await screen.findByRole('heading',{name:'Saved rules'})).toBeVisible()
    expect(screen.getByText('Showing 1 of 14 rules')).toBeVisible()
    expect(localStorage.getItem('oathkeeper.saved-rules.v1')).toContain(rule.id)
  })
  test('rule source shows actual typed Oathkeeper alongside compiled JSON',async()=>{
    const user=userEvent.setup();open(rulePath('seattle.parking.authorized-vehicle-exclusion'))
    expect(screen.getByText('Vehicle')).toBeVisible()
    await user.click(screen.getByRole('tab',{name:'.oath source'}))
    expect(screen.getByRole('tabpanel')).toHaveTextContent('vehicle: Vehicle')
    expect(screen.getByRole('tabpanel')).toHaveTextContent('requires parking.on_municipal_property($matter);')
    expect(screen.getByRole('link',{name:'Open complete source'})).toHaveAttribute('href',expect.stringContaining('/reference/source-seattle?line='))
  })
  test('authority views expose real citations and their linked rules',async()=>{
    const user=userEvent.setup();open('/lawlib?view=authorities')
    const heading=screen.getByRole('heading',{name:'Timbs v. Indiana'})
    await user.click(within(heading).getByRole('link'))
    expect(await screen.findByRole('heading',{name:'Timbs v. Indiana',level:1})).toBeVisible()
    expect(screen.getByRole('heading',{name:'Formalized rules'})).toBeVisible()
    expect(screen.getByRole('link',{name:'Official source'})).toHaveAttribute('href',expect.stringContaining('supremecourt.gov'))
  })
  test('unknown and malformed routes show a usable missing page',()=>{
    open('/something/%E0%A4%A')
    expect(screen.getByRole('heading',{name:'Page not found'})).toBeVisible()
    expect(screen.getByRole('link',{name:'Return to overview'})).toHaveAttribute('href','#/')
  })
})

describe('real case proof workspaces',()=>{
  test('a linked step opens directly and missing evidence blocks its dependents',async()=>{
    const user=userEvent.setup();open('/cases/long-excessive-fines?step=violation')
    expect(within(screen.getByTestId('proof-detail')).getByRole('heading',{name:'The proportionality conclusion'})).toBeVisible()
    const toggle=screen.getByRole('switch',{name:'Include the court’s proportionality finding'})
    await user.click(toggle)
    expect(screen.getByTestId('proof-outcome')).toHaveTextContent('Blocked')
    expect(screen.getByTestId('proof-detail')).toHaveTextContent('This step is blocked')
    await user.click(toggle)
    expect(screen.getByTestId('proof-outcome')).toHaveTextContent('Rust-verified baseline')
  })
  test('issue selection navigates to a distinct shareable case page',async()=>{
    const user=userEvent.setup();open('/cases/long-excessive-fines')
    await user.click(screen.getByRole('combobox',{name:'Explore an issue'}))
    await user.click(await screen.findByRole('option',{name:'Homestead declaration'}))
    expect(await screen.findByRole('heading',{name:'Homestead declaration',level:1})).toBeVisible()
    expect(window.location.hash).toBe('#/cases/long-homestead')
    expect(screen.getByTestId('proof-detail')).toHaveTextContent('No additional declaration')
  })
  test('the assumptions dialog shows attributed court findings',async()=>{
    const user=userEvent.setup();open('/cases/long-excessive-fines')
    await user.click(screen.getByRole('button',{name:/Inspect the assumptions/}))
    const dialog=screen.getByRole('dialog',{name:'What this proof assumes.'})
    expect(within(dialog).getByText('Court’s disproportionality finding')).toBeVisible()
    await user.keyboard('{Escape}')
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument()
    await user.click(screen.getByRole('tab',{name:'Verification report'}))
    expect(screen.getByRole('tabpanel')).toHaveTextContent(records[0].report.bundle_sha256 as string)
    expect(screen.getByRole('tabpanel')).toHaveTextContent('fact_assumptions')
    act(()=>navigate('/cases/long-homestead'))
    expect(await screen.findByTestId('proof-detail')).toHaveTextContent('No additional declaration')
    expect(screen.getByRole('tab',{name:'Proof workspace'})).toHaveAttribute('data-state','active')
  })
})

describe('documentation and code reference',()=>{
  test('oracle documentation exposes agreement and separate assurance dimensions',()=>{
    open('/docs/oracles')
    expect(screen.queryByRole('slider')).not.toBeInTheDocument()
    expect(screen.getByRole('heading',{name:'Roles and agreement'})).toBeVisible()
    expect(screen.getByRole('heading',{name:'Assurance instead of a score'})).toBeVisible()
  })
  test('Python documentation renders the real interface and copies runnable code',async()=>{
    const user=userEvent.setup();const copy=vi.fn().mockResolvedValue(undefined)
    Object.defineProperty(navigator,'clipboard',{configurable:true,value:{writeText:copy}})
    open('/docs/python')
    expect(screen.getByRole('heading',{name:'Python API',level:1})).toBeVisible()
    expect(screen.getByRole('heading',{name:'verify_case(module_json, case_json)'})).toBeVisible()
    await user.click(screen.getAllByRole('button',{name:'Copy to clipboard'})[1])
    expect(copy).toHaveBeenCalledWith(expect.stringContaining('oathkeeper.verify_file'))
    expect(screen.getByRole('link',{name:'binding implementation'})).toHaveAttribute('href','#/reference/python')
  })
  test('the table of contents preserves the document route',async()=>{
    const user=userEvent.setup();open('/docs/language')
    await user.click(screen.getByRole('link',{name:'Kernel-enforced types'}))
    expect(await screen.findByRole('heading',{name:'Language specification 0.5',level:1})).toBeVisible()
    expect(window.location.hash).toBe('#/docs/language?section=kernel-enforced-types')
    expect(document.getElementById('doc-kernel-enforced-types')).toBeTruthy()
  })
  test('the source browser searches exact implementation lines and links symbols',async()=>{
    const user=userEvent.setup();open('/reference/core')
    const file=sourceFiles.find(file=>file.id==='core')!
    const symbol=codeSymbols(file.code).find(symbol=>symbol.name==='assemble_case')!
    await user.type(screen.getByRole('textbox',{name:'Find in source'}),'pub fn assemble_case')
    expect(screen.getByText('1 matches')).toBeVisible()
    await user.click(screen.getByRole('link',{name:`L${symbol.line}`}))
    expect(window.location.hash).toBe(`#/reference/core?line=${symbol.line}`)
    expect(document.getElementById(`source-line-${symbol.line}`)).toHaveClass('selected')
    expect(screen.getByRole('link',{name:'Download .rs'})).toHaveAttribute('href','/assets/core.rs')
  })
  test('command search finds a Rust function and navigates with the keyboard',async()=>{
    const user=userEvent.setup();open('/')
    await user.keyboard('{Control>}k{/Control}')
    const dialog=screen.getByRole('dialog',{name:'Search Oathkeeper'})
    await user.type(within(dialog).getByRole('combobox'),'verify_text')
    expect(within(dialog).getByRole('option',{name:/verify_text/})).toBeVisible()
    await user.keyboard('{Enter}')
    expect(await screen.findByRole('heading',{name:'Core API',level:1})).toBeVisible()
    expect(window.location.hash).toMatch(/^#\/reference\/core\?line=\d+/)
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument()
  })
})

describe('source integrity and verified fixtures',()=>{
  test('browser SHA-256 checks match every real pinned artifact and reject changed bytes',async()=>{
    for(const snapshot of snapshots){
      const bytes=await readFile(path.join(process.cwd(),'.generated/public',sourceAsset(snapshot)))
      const result=await verifySnapshot(Uint8Array.from(bytes).buffer,snapshot.sha256)
      expect(result.matches).toBe(true)
    }
    expect((await verifySnapshot(new TextEncoder().encode('modified').buffer,snapshots[0].sha256)).matches).toBe(false)
  })
  test('the source page reports an actual successful digest computation',async()=>{
    const user=userEvent.setup();const bytes=await readFile(path.join(process.cwd(),'.generated/public',sourceAsset(snapshots[0])))
    const fetchMock=vi.spyOn(globalThis,'fetch').mockResolvedValue({ok:true,arrayBuffer:async()=>Uint8Array.from(bytes).buffer} as Response)
    try{open('/sources');await user.click(screen.getAllByRole('button',{name:'Verify bytes'})[0]);expect(await screen.findByRole('status')).toHaveTextContent('SHA-256 matches the pinned snapshot.')}finally{fetchMock.mockRestore()}
  })
  test('all fixture roots retain their Rust-generated baseline and expose assumptions',()=>{
    expect(records).toHaveLength(5)
    for(const record of records){expect(record.report.kernel_version).toBe('0.5.0');expect(traceDependencies(record.bundle.proof).get(record.bundle.query.root)).toBe(true);expect(applicationSteps(record).length).toBeGreaterThan(0);expect(record.report.unreviewed_authority_assessment_count).toBeGreaterThan(0)}
    const trace=traceDependencies(records[0].bundle.proof,'disproportionality-finding');expect(trace.get('fine')).toBe(true);expect(trace.get('violation')).toBe(false)
  })
  test('structured filtering and source display remain safe',()=>{
    expect(filterRules('','us.wa.seattle','ordinance')).toHaveLength(7)
    expect(escapeHtml('<script>"x"</script>')).toBe('&lt;script&gt;&quot;x&quot;&lt;/script&gt;')
  })
  test('every linked local download is present',async()=>{
    const files=['/assets/oathkeeper-0.5-source.zip','/assets/parking.module.json','/assets/sources.lock.json',...sourceFiles.map(file=>`/assets/${file.id}.${file.path.endsWith('.oath')?'oath':'rs'}`),...snapshots.map(sourceAsset),...records.flatMap(record=>[`/assets/${record.slug}.bundle.json`,`/assets/${record.slug}.report.json`])]
    for(const file of files)await access(path.join(process.cwd(),'.generated/public',file))
  })
})
