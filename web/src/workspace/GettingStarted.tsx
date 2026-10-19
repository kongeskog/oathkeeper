import { ArrowRight, CheckCircle2, CircleDot, Download, FileCode2, Play } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Card } from '@/components/ui/card'
import { AppLink, CodeBlock } from './shared'

const rule=`predicate library.resident(Person);
predicate library.may_enter(Person);

rule example.library.resident-access(person: Person) @ "1" {
  requires library.resident($person);
  ensures library.may_enter($person);
}`

const proof=`fact resident: library.resident(example.alex) {
  kind stipulated_fact;
  // Source, date, pinpoint, and explanation are explicit.
}

proof {
  step resident: library.resident(example.alex)
    by fact resident;
  step allowed: library.may_enter(example.alex)
    by apply example.library.resident-access @ "1"
    (person = example.alex) using [resident];
}`

export function GettingStarted(){
  return <div className="page-container learning-page">
    <div className="learning-hero"><Badge variant="outline">10 minute guide</Badge><h1>Your first checked argument</h1><p>Start with one fictional rule and one agreed fact. Oathkeeper will verify that the conclusion follows, then report exactly what the result depends on.</p><div><Button asChild><a href="https://github.com/kongeskog/oathkeeper/tree/main/examples" target="_blank" rel="noreferrer"><Download/>Open the example files</a></Button><Button variant="outline" asChild><AppLink to="/language">Keep the language reference nearby<ArrowRight/></AppLink></Button></div></div>

    <section className="learning-model"><div><span>FACT</span><code>resident(alex)</code></div><ArrowRight/><div><span>RULE</span><code>resident(x) → may_enter(x)</code></div><ArrowRight/><div className="learning-model-result"><span>CONCLUSION</span><code>may_enter(alex)</code></div></section>

    <section className="learning-step"><div className="learning-step-label"><span>01</span><div><h2>Write a bounded rule</h2><p>A predicate names a claim. A rule lists the claims it requires and the single claim it establishes.</p></div></div><Card className="learning-code"><CodeBlock language="oath">{rule}</CodeBlock></Card></section>

    <section className="learning-step"><div className="learning-step-label"><span>02</span><div><h2>Supply a fact and a proof</h2><p>Facts do not become true because they appear in a file. Their source and basis stay visible, and the proof names the exact steps used.</p></div></div><Card className="learning-code"><CodeBlock language="oath">{proof}</CodeBlock></Card></section>

    <section className="learning-step"><div className="learning-step-label"><span>03</span><div><h2>Run the checker</h2><p>Clone the repository, then verify the complete example with the Rust CLI.</p></div></div><Card className="learning-code"><CodeBlock>{`git clone https://github.com/kongeskog/oathkeeper.git
cd oathkeeper
cargo run -- verify examples/library-access.case.oath`}</CodeBlock></Card></section>

    <section className="verified-result"><div className="verified-result-icon"><CheckCircle2/></div><div><span>STATUS</span><h2>derivation_verified</h2><p>The proof is structurally valid: the fact matches the rule’s premise, the substitution is type-correct, and the final step matches the query.</p></div><div className="verified-result-deps"><span><CircleDot/>1 fact</span><span><FileCode2/>1 rule</span><span><Play/>2 steps</span></div></section>

    <section className="learning-next"><div><span>WHAT YOU HAVE PROVED</span><h2>A precise conditional claim</h2><p>If the fictional ordinance is accepted as formalized and Alex’s residency is accepted as evidence, then the conclusion follows. Oathkeeper does not independently confirm either input.</p></div><div><h3>Continue from here</h3><AppLink to="/language">Learn the complete Oath language<ArrowRight/></AppLink><AppLink to="/lawlib">Inspect source-backed modules<ArrowRight/></AppLink><AppLink to="/resources">Choose a CLI or API guide<ArrowRight/></AppLink></div></section>
  </div>
}
