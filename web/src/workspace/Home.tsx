import { ArrowRight, BookOpen, Braces, CheckCircle2, GitBranch, Library, Scale } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { AppLink } from './shared'

const principles=[
  {icon:BookOpen,title:'State the inputs',text:'Record the source, scope, date, evidence, and assumptions that an argument relies on.'},
  {icon:GitBranch,title:'Show every step',text:'Connect facts to rules in a derivation that a person can read and another tool can replay.'},
  {icon:CheckCircle2,title:'Check the result',text:'Use the deterministic Rust kernel to confirm that each step follows from the declared inputs.'},
]

export function Home(){
  return <div className="marketing-page">
    <section className="marketing-hero">
      <div className="marketing-hero-copy">
        <Badge variant="outline">Open source · Oath language 0.5</Badge>
        <h1>Legal reasoning you can inspect.</h1>
        <p>Oathkeeper is an open language and proof checker for legal arguments. It turns rules, evidence, and assumptions into a derivation that people can read and software can verify.</p>
        <div className="marketing-actions"><Button size="lg" asChild><AppLink to="/getting-started">Try a small example<ArrowRight/></AppLink></Button><Button size="lg" variant="outline" asChild><AppLink to="/lawlib"><Library/>Explore lawlib</AppLink></Button></div>
        <div className="marketing-audience"><span>For legal researchers</span><span>For public-interest technologists</span><span>For developers</span></div>
      </div>
      <Card className="marketing-proof-card">
        <div className="proof-card-top"><span>example/library-access.case.oath</span><Badge variant="secondary">Verified</Badge></div>
        <div className="proof-card-logic"><div><small>FACT</small><code>resident(alex)</code></div><span>+</span><div><small>RULE</small><code>resident → may_enter</code></div></div>
        <div className="proof-card-arrow">↓</div>
        <div className="proof-card-result"><CheckCircle2/><div><small>DERIVATION VERIFIED</small><code>may_enter(alex)</code></div></div>
        <p>The result is conditional on the rule and fact supplied to the checker.</p>
      </Card>
    </section>

    <section className="marketing-section">
      <div className="section-heading"><span>HOW IT WORKS</span><h2>From source material to a checkable argument</h2><p>Oathkeeper keeps the parts of legal reasoning visible instead of collapsing them into a single answer.</p></div>
      <div className="principle-grid">{principles.map(({icon:Icon,title,text},index)=><Card key={title} className="principle-card"><span className="principle-number">0{index+1}</span><Icon/><h3>{title}</h3><p>{text}</p></Card>)}</div>
    </section>

    <section className="marketing-section marketing-split">
      <div><span className="section-kicker">THE LANGUAGE</span><h2>Readable enough to review. Precise enough to run.</h2><p>Oath files declare typed predicates, source-backed rules, facts, a query, and explicit proof steps. Imports and evidence can be pinned by SHA-256 so the same argument can be replayed against the same inputs.</p><Button variant="outline" asChild><AppLink to="/language"><Braces/>Read the language docs<ArrowRight/></AppLink></Button></div>
      <Card className="marketing-code-sample"><pre><code>{`rule example.access(person: Person) @ "1" {
  requires library.resident($person);
  ensures library.may_enter($person);
}

step allowed: library.may_enter(alex)
  by apply example.access @ "1"
  using [resident];`}</code></pre></Card>
    </section>

    <section className="marketing-section">
      <div className="section-heading"><span>BUILT FOR REVIEW</span><h2>A shared object for research, tooling, and critique</h2></div>
      <div className="use-grid"><div><Scale/><h3>Model bounded legal rules</h3><p>Capture a rule’s jurisdiction, time, authority, scope, and source without claiming broader coverage.</p></div><div><Library/><h3>Publish reusable modules</h3><p>Organize formalizations in lawlib so others can inspect, test, and improve them.</p></div><div><GitBranch/><h3>Compare arguments</h3><p>Make supporting, opposing, and missing premises explicit under the same frozen context.</p></div></div>
    </section>

    <section className="boundary-callout"><div><span>AN IMPORTANT BOUNDARY</span><h2>The checker verifies the derivation, not the world.</h2></div><p>A verified proof shows that a conclusion follows from declared inputs. It does not establish that a source is current law, that evidence is true, or that a court would accept the formalization.</p><AppLink to="/resources">Read about trust and limits<ArrowRight/></AppLink></section>

    <section className="marketing-cta"><div><span>START SMALL</span><h2>See one fact become one checked conclusion.</h2><p>The getting-started guide explains the language through a compact fictional example.</p></div><Button size="lg" asChild><AppLink to="/getting-started">Open the guide<ArrowRight/></AppLink></Button></section>
  </div>
}
