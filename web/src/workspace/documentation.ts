import storage from '@generated/docs/STORAGE.md?raw'
import taxGuide from '@generated/docs/TAX.md?raw'
import taxCode from '@generated/reference/tax.rs?raw'
import arithmeticCode from '@generated/reference/arithmetic.rs?raw'
import taxRules from '@generated/reference/source-tax.oath?raw'
import shankRules from '@generated/reference/source-shank.oath?raw'
import compilerCode from '@generated/reference/compiler.rs?raw'
import syntaxCode from '@generated/reference/syntax.rs?raw'
import schemaCode from '@generated/reference/schema.rs?raw'
import argumentsCode from '@generated/reference/arguments.rs?raw'
import seattleCode from '@generated/reference/source-seattle.oath?raw'
import washingtonCode from '@generated/reference/source-washington.oath?raw'
import federalCode from '@generated/reference/source-federal.oath?raw'
import vocabularyCode from '@generated/reference/vocabulary.oath?raw'
import pendingOracleCode from '@generated/reference/pending-oracle.oath?raw'
import oracles from '@generated/docs/ORACLES.md?raw'
import python from '@generated/docs/PYTHON.md?raw'
import cli from '@generated/docs/CLI.md?raw'

import gettingStarted from '@generated/docs/GETTING_STARTED.md?raw'
import language from '@generated/docs/SPEC-0.5.md?raw'
import v03 from '@generated/docs/SPEC-0.3.md?raw'
import research from '@generated/docs/RESEARCH.md?raw'
import soundness from '@generated/docs/SOUNDNESS.md?raw'
import analysisCode from '@generated/reference/analysis.rs?raw'
import oraclesCode from '@generated/reference/oracles.rs?raw'
import wasmCode from '@generated/reference/wasm.rs?raw'
import pilotCode from '@generated/reference/parking-assessment.oath?raw'
import previousSpec from '@generated/docs/SPEC-0.2.md?raw'
import originalSpec from '@generated/docs/SPEC.md?raw'
import architecture from '@generated/docs/ARCHITECTURE.md?raw'
import training from '@generated/docs/TRAINING.md?raw'
import seattle from '@generated/docs/SEATTLE.md?raw'
import roadmap from '@generated/docs/SEATTLE_PLAN.md?raw'
import coreCode from '@generated/reference/core.rs?raw'
import modelCode from '@generated/reference/model.rs?raw'
import kernelCode from '@generated/reference/kernel.rs?raw'
import pythonCode from '@generated/reference/python.rs?raw'
import cliCode from '@generated/reference/cli.rs?raw'

export const docs=[
  {id:'storage',title:'Lawlib source, builds & evidence',category:'Architecture',summary:'Authored .oath, reproducible JSON builds, and hash-verified external evidence.',content:storage},
  {id:'tax',title:'Federal tax · backdoor Roth',category:'Start here',summary:'2025 IRA allocation, scope, rounding, real precedent and executable APIs.',content:taxGuide},
  {id:'getting-started',title:'Getting started',category:'Start here',summary:'Install, verify a real case, and understand the report.',content:gettingStarted},
  {id:'seattle',title:'The Seattle module',category:'Start here',summary:'Real authorities, historical case proofs, scope, and known limitations.',content:seattle},
  {id:'language',title:'Language specification 0.5',category:'Language',summary:'Readable declarations, kernel-enforced signatures, pinned imports, and explicit proofs.',content:language},
  {id:'spec-0-3',title:'Archived specification 0.3',category:'Language',summary:'Previous authoring syntax and retired report contract.',content:v03},
  {id:'research',title:'Research & design decisions',category:'Project',summary:'Primary sources, related systems, and unresolved design questions.',content:research},
  {id:'soundness',title:'Soundness and limits',category:'Architecture',summary:'Conditional proof calculus, trust assumptions, and search limits.',content:soundness},
  {id:'spec-0-2',title:'Previous specification 0.2',category:'Language',summary:'Authority scope, historical assessments, and the original module format.',content:previousSpec},
  {id:'spec-0-1',title:'Original specification 0.1',category:'Language',summary:'The ground proof calculus and original oracle-count score.',content:originalSpec},
  {id:'architecture',title:'Kernel & trust boundaries',category:'Architecture',summary:'What the deterministic Rust kernel checks and which inputs it trusts.',content:architecture},
  {id:'oracles',title:'Oracle protocols',category:'Architecture',summary:'Roles, explicit agreement, abstention, disagreement, and confidence meaning.',content:oracles},
  {id:'python',title:'Python API',category:'API reference',summary:'Compile .oath, verify files or JSON, compare arguments, and inspect reports.',content:python},
  {id:'cli',title:'CLI reference',category:'API reference',summary:'Verify, compose, normalize, and audit source snapshots.',content:cli},
  {id:'training',title:'Verifiable rewards',category:'Architecture',summary:'Pin the context and allow models to submit proof steps only.',content:training},
  {id:'roadmap',title:'Seattle coverage plan',category:'Project',summary:'Inventory, review states, omissions, and language work still required.',content:roadmap},
]

export interface SourceFile {id:string;title:string;path:string;summary:string;code:string;language?:string}
export const sourceFiles:SourceFile[]=[
  {id:'source-tax',title:'Federal IRA rules · 2025',path:'oathkeeper-lawlib/us/tax/backdoor-roth-2025.oath',summary:'Source-backed contribution and pro-rata allocation rules with exact quantity witnesses.',code:taxRules,language:'Oathkeeper'},
  {id:'source-shank',title:'Shank · actual basis findings',path:'oathkeeper-lawlib/us/tax/shank-basis.oath',summary:'Three narrowly scoped rules from the official 2018 Tax Court opinion.',code:shankRules,language:'Oathkeeper'},
  {id:'tax',title:'Proof-producing tax calculator',path:'crates/oathkeeper-core/src/tax.rs',summary:'Bounded input validation, unresolved information and certificate construction followed by kernel replay.',code:taxCode},
  {id:'arithmetic',title:'Checked quantity arithmetic',path:'crates/oathkeeper-core/src/arithmetic.rs',summary:'Exact addition, subtraction and pro-rata allocation with explicit units and rounding.',code:arithmeticCode},
  {id:'analysis',title:'Bounded argument search',path:'crates/oathkeeper-core/src/analysis.rs',summary:'Supporting and opposing certificates, limits, and sensitivity.',code:analysisCode},
  {id:'oracles',title:'Oracle agreements',path:'crates/oathkeeper-core/src/oracles.rs',summary:'Unanimity, abstention, disputes, withdrawals, and explicit trust.',code:oraclesCode},
  {id:'wasm',title:'WebAssembly bindings',path:'crates/oathkeeper-wasm/src/lib.rs',summary:'The same Rust core used by the browser worker.',code:wasmCode},
  {id:'parking-assessment',title:'Real-record parking assessment',path:'oathkeeper-lawlib/us/wa/seattle/cases/long-parking-evidence-review.case.oath',summary:'Incomplete exception evidence, without an invented verdict.',code:pilotCode,language:'Oathkeeper'},
  {id:'source-seattle',title:'Seattle parking law',path:'oathkeeper-lawlib/us/wa/seattle/parking.oath',summary:'Actual SMC 11.72.440 rules, explicit exclusions, sources, and authority assessments.',code:seattleCode,language:'Oathkeeper'},
  {id:'source-washington',title:'Washington precedent',path:'oathkeeper-lawlib/us/wa/long.oath',summary:'Selected Long majority holdings with their precise scope and citations.',code:washingtonCode,language:'Oathkeeper'},
  {id:'source-federal',title:'Federal precedent',path:'oathkeeper-lawlib/us/timbs.oath',summary:'The Timbs incorporation holding and federal jurisdiction declaration.',code:federalCode,language:'Oathkeeper'},
  {id:'vocabulary',title:'Lawlib vocabulary',path:'oathkeeper-lawlib/vocabulary.oath',summary:'Matter and Vehicle domain types with exact predicate signatures.',code:vocabularyCode,language:'Oathkeeper'},
  {id:'pending-oracle',title:'Pending oracle reassessment',path:'oathkeeper-lawlib/us/wa/seattle/cases/long-oracle-reassessment.case.oath',summary:'An actual Long record application awaiting an agreed response; deliberately unverified.',code:pendingOracleCode,language:'Oathkeeper'},
  {id:'compiler',title:'Source compiler',path:'crates/oathkeeper-core/src/compiler/mod.rs',summary:'Transitive hash checks, declaration linking, explicit IR, and source maps.',code:compilerCode},
  {id:'syntax',title:'Language parser',path:'crates/oathkeeper-core/src/compiler/syntax.rs',summary:'The lexer and parser for the .oath authoring language.',code:syntaxCode},
  {id:'schema',title:'Kernel type checks',path:'crates/oathkeeper-core/src/schema.rs',summary:'Predicate arity, nominal domain types, and ground substitutions.',code:schemaCode},
  {id:'arguments',title:'Counterargument comparison',path:'crates/oathkeeper-core/src/arguments.rs',summary:'Check supplied opposing arguments under exactly the same frozen context.',code:argumentsCode},
  {id:'core',title:'Core API',path:'crates/oathkeeper-core/src/lib.rs',summary:'Parsing, verification, errors, and module/case composition.',code:coreCode},
  {id:'model',title:'Wire types',path:'crates/oathkeeper-core/src/model.rs',summary:'Claims, terms, rules, authorities, cases, and reports.',code:modelCode},
  {id:'kernel',title:'Verification kernel',path:'crates/oathkeeper-core/src/kernel.rs',summary:'Deterministic proof checking and dependency tracking.',code:kernelCode},
  {id:'python',title:'Python bindings',path:'crates/oathkeeper-python/src/lib.rs',summary:'The native PyO3 entry points and exception mapping.',code:pythonCode},
  {id:'cli',title:'CLI implementation',path:'crates/oathkeeper-cli/src/main.rs',summary:'Command parsing, normalization, and exit status.',code:cliCode},
]
export function codeSymbols(code:string) {
  return code.split('\n').flatMap((line,index)=>{
    const match=line.match(/^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?(fn|struct|enum|type)\s+([A-Za-z_][A-Za-z0-9_]*)/) ?? line.match(/^\s*(rule|predicate|authority|assessment|constant|jurisdiction|source|module|case|step)\s+([A-Za-z0-9_.\/-]+)/)
    return match ? [{kind:match[1],name:match[2],line:index+1}] : []
  })
}
export function docLink(href:string) {
  if(href.startsWith('http'))return href
  if(href.includes('metadata/us/tax/'))return href.endsWith('sources.lock.json')?'#/tax/backdoor-roth?view=sources':'#/tax/backdoor-roth?view=review'
  if(href.includes('metadata/us/wa/seattle/'))return href.endsWith('sources.lock.json')?'#/sources':href.endsWith('coverage.json')?'#/lawlib':'#/review'
  if(href.includes('tests/scenarios/us/tax/'))return '#/tax/backdoor-roth'
  if(href.includes('lawlib/us/tax/README'))return '#/docs/tax'
  if(href.includes('tax.rs'))return '#/reference/tax'
  if(href.includes('arithmetic.rs'))return '#/reference/arithmetic'
  if(href.includes('backdoor-roth-2025.oath'))return '#/reference/source-tax'
  if(href.includes('shank-basis.oath'))return '#/reference/source-shank'
  if(href.includes('oathkeeper-python/src'))return '#/reference/python'
  if(href.includes('oathkeeper-cli/src'))return '#/reference/cli'
  if(href.includes('analysis.rs'))return '#/reference/analysis'
  if(href.includes('oracles.rs'))return '#/reference/oracles'
  if(href.includes('oathkeeper-wasm'))return '#/reference/wasm'
  if(href.includes('model.rs'))return '#/reference/model'
  if(href.includes('kernel.rs'))return '#/reference/kernel'
  if(href.includes('lib.rs'))return '#/reference/core'
  const file=href.split('/').at(-1)?.split('#')[0]
  const mapping:Record<string,string>={'STORAGE.md':'storage','SPEC-0.5.md':'language','SPEC-0.4.md':'language','SPEC-0.3.md':'spec-0-3','RESEARCH.md':'research','SOUNDNESS.md':'soundness','SPEC-0.2.md':'spec-0-2','SPEC.md':'spec-0-1','SEATTLE_PLAN.md':'roadmap','ARCHITECTURE.md':'architecture','TRAINING.md':'training'}
  if(file&&mapping[file])return `#/docs/${mapping[file]}`
  const oathFiles:Record<string,string>={'long-parking-evidence-review.case.oath':'parking-assessment','parking.oath':'source-seattle','long.oath':'source-washington','timbs.oath':'source-federal','vocabulary.oath':'vocabulary','long-oracle-reassessment.case.oath':'pending-oracle'}
  if(file&&oathFiles[file])return `#/reference/${oathFiles[file]}`
  if(file?.endsWith('.case.oath'))return `#/cases/${file.replace('.case.oath','')}`
  if(href.includes('lawlib')||file==='parking.module.json'||file==='coverage.json')return '#/lawlib'
  if(file==='sources.lock.json')return '#/sources'
  if(href.includes('README')||file==='ROADMAP.md')return '#/docs/getting-started'
  return '#/reference'
}
