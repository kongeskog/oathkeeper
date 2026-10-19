# Oathkeeper and lawlib: revised technical foundation

## 1. Summary and design decisions

Build **Oathkeeper**, a programming language implemented in Rust, and **lawlib**, a separate repository of legal programs written in `.oath` files.

The long-term goal remains **representing an entire legal system**: its institutions, sources, substantive rules, procedures, interpretations, adjudication, enforcement, remedies, and changes over time. The first release establishes the foundation through household rules, selected US constitutional provisions and precedent, and a constrained IRA example.

Representing a legal question does not require making it mechanically decidable. Oathkeeper must support executable conclusions, unresolved questions, competing interpretations, and explicit human findings within the same model.

The programming-language and legal-design review leads to these decisions:

| Concern | Decision |
|---|---|
| Language design | A small, statically typed functional language with explicit queries, rules, and priority |
| Legal vocabulary | Cases, provisions, treatments, findings, and similar concepts are typed library records |
| Time | Calendar values and arithmetic belong in the standard library; legal timing rules belong in lawlib |
| Sources | Versioned references, locators, and hashes; document storage is optional and external |
| Uncertainty | Missing facts and disputed claims remain explicit |
| Results | One result variant determines whether a query resolved, conflicted, was inapplicable, or could not be answered |
| Verification | An independent checker validates derivations against an identified program and scenario |
| Organization | Hierarchical directories for navigation; explicit relationships determine authority and applicability |
| Public operations | `run`, `test`, and `verify` have distinct responsibilities; no separate public `eval` |

Three assurances remain separate throughout the system:

1. **Computational verification:** the result follows from the encoded program and supplied premises.
2. **Source integrity:** inspected bytes match the expected source hash.
3. **Legal review:** a recorded reviewer assessed the encoding, interpretation, or coverage.

A successful check in one category does not establish the others.

Initial exclusions are training infrastructure, automatic natural-language formalization, a graphical interface, and a general proof assistant. These exclusions do not narrow the long-term legal-system goal.

## 2. Repositories and Rust implementation

### Separate repositories

Create independent `oathkeeper` and `lawlib` Git repositories from the first commit, with separate releases and CI.

**Oathkeeper owns:**

- Language grammar and semantics.
- Compiler, interpreter, and certificate checker.
- Standard library.
- Package, scenario, result, and certificate formats.
- CLI and Rust API.
- Conformance tests and developer documentation.

**Lawlib owns:**

- Shared legal-model libraries.
- Jurisdiction and institution models.
- Encoded sources, provisions, precedent, and legal processes.
- Interpretations, coverage statements, and review records.
- Legal examples and scenario fixtures.

Oathkeeper builds and tests without lawlib. Lawlib pins an Oathkeeper release or explicitly identified candidate build. Oathkeeper release candidates test against a pinned lawlib revision; neither repository’s CI depends on both default branches moving together.

### Cargo workspace

Start with four crates. Keep compiler, runtime, and package orchestration as modules until separate crates provide a concrete benefit.

| Crate | Responsibility |
|---|---|
| `oath-core` | Exact values, identifiers, typed IR, diagnostics, result/certificate structures, canonical serialization |
| `oath-verify` | Independent certificate checker; depends on `oath-core` |
| `oathkeeper` | Public API, compiler, interpreter, package loading, source auditing, test harness |
| `oath-cli` | The `oath` executable |

The critical dependency boundary is that **`oath-verify` cannot depend on the interpreter or call it**. Shared code may implement exact arithmetic and serialization, but must not contain a shared rule-selection algorithm that substitutes for independent checking.

`run` and `verify` operate on already-loaded programs and requests. They perform no filesystem access, network access, clock reads, or random operations. Package loading, source retrieval, and output writing occur outside them.

Use the installed Rust 1.98.1 toolchain initially, with edition 2024. Pin compatible dependencies in `Cargo.lock`. For exact arithmetic, begin with `num-rational` 0.4.2 and its compatible `num-bigint` 0.4 family. [Dependency declaration](https://docs.rs/crate/num-rational/0.4.2/source/Cargo.toml)

Implement a handwritten lexer, recursive-descent declaration parser, and Pratt expression parser. Preserve source spans through typed IR. Execute that IR directly in the initial interpreter; defer native compilation and optimization.

## 3. Language, types, and execution semantics

### Minimal declaration surface

The initial declaration forms are:

```text
module
use
type
enum
const
fn
query
rule
priority
```

Their responsibilities are distinct:

- `fn` defines a closed computation.
- `query` defines a question whose answer is determined by the included rules.
- `rule` contributes a conditional answer to a query.
- `priority` explicitly orders competing rules for that query.

`SourceRef`, `ProvisionRevision`, `Case`, `Opinion`, `LegalProposition`, `Treatment`, `Issue`, and `CoverageRecord` are library types instantiated through ordinary constants. They do not each require a language keyword.

Declarations can attach a typed constant through a `meta` field. This preserves citations, encoding rationale, coverage, and interpretation records without making every legal concept part of the grammar.

Metadata supplies provenance. Executable expressions supply applicability and authority conditions. Merely citing a provision does not automatically activate it.

### Core values and library types

The core supports:

- `Unit`, `Bool`, `Text`.
- Arbitrary-precision `Int`.
- Exact `Rational`.
- Nominal records and variants.
- Generic type parameters.
- Finite immutable lists.
- Functions and immutable bindings.

`Option`, `Result`, sets, maps, identifiers, and domain-specific wrappers are standard-library definitions. Collection traversal and serialization have specified deterministic ordering.

| Library | Initial responsibilities |
|---|---|
| `std.numeric` | Explicit rounding and numeric helpers |
| `std.date` | Civil dates, parsing, comparison, and calendar arithmetic |
| `std.money` | Exact monetary amounts with distinct currency types |
| `std.collections` | Deterministic collection operations |
| `std.provenance` | Source references, anchors, encoding notes, and coverage records |
| Lawlib shared libraries | Legal instruments, authority, findings, inventories, deadlines, precedent, and legal effects |

Decimal literals represent exact decimal fractions. Rational values use reduced numerators and positive denominators. Floating-point arithmetic is outside v0.1.

Currency conversion requires an explicit supplied rate or rule. Rounding requires an explicit quantum and mode.

### Time without additional kernel primitives

`Date` is a standard-library nominal type represented by a civil-day index, with a documented Gregorian calendar and epoch. Parsing and year/month/day construction use library functions that report invalid input explicitly.

Start with:

```text
add_days(date, count)
add_months(date, count, overflow_policy)
days_between(start, end)
```

`add_months` requires an explicit policy such as rejecting an invalid resulting date or clamping it to the final day of that month.

There is no initial `CalendarPeriod` type. Introduce a library record for calendar adjustments only when programs need to store or manipulate those adjustments as values.

An elapsed duration can be a nominal numeric type with a documented unit. `LocalTime`, `Instant`, and timezone handling are deferred until an example needs them; they remain library concerns.

Legal rules for anniversaries, business days, holidays, filing deadlines, excluded days, extensions, and tolling belong in lawlib. The runtime supplies no universal legal counting convention.

### Functional subset

Expressions support:

- Immutable local bindings.
- Function calls.
- Record construction and access.
- Variant construction and pattern matching.
- Conditionals.
- Arithmetic, comparisons, and Boolean operations.
- Finite `map`, `filter`, `fold`, `all`, and `any`.

Collection combinators use statically identified function bodies. v0.1 has no unrestricted recursion, mutable state, dynamic function dispatch, or language-level I/O.

Functions call functions. Query and rule expressions may additionally depend on other queries. Executable dependency cycles are rejected. Legal records may reference each other cyclically because references are data, not recursive evaluation.

Invalid operations, including division by zero, produce execution errors with source spans. They do not produce legal conclusions.

### Example syntax

This excerpt imports typed policy and provenance constants from another module. The fictional policy was adopted in 2026, commences in 2029, and expressly permits emergency noise.

```oath
module household.quiet_hours;

use std.date.Date;
use household.policy.{
    commencement,
    policy_note,
    ordinary_note,
    emergency_note
};

enum NoiseStatus {
    Permitted,
    Prohibited
}

type NoiseEvent = {
    on: Date,
    member: Bool,
    after_hours: Bool,
    emergency: Bool
};

query noise_status(event: NoiseEvent) -> NoiseStatus {
    scope = true;

    applies =
        event.on >= commencement
        and event.member
        and event.after_hours;

    meta = policy_note;
}

rule ordinary for noise_status(event) {
    when = true;
    yield = NoiseStatus.Prohibited;
    meta = ordinary_note;
}

rule emergency for noise_status(event) {
    when = event.emergency;
    yield = NoiseStatus.Permitted;
    meta = emergency_note;
}

priority emergency_before_ordinary for noise_status(event) {
    higher = emergency;
    lower = ordinary;
    when = true;
    meta = emergency_note;
}
```

There is one priority mechanism. Do not also introduce an `overrides` field with overlapping semantics.

### Compilation

The compilation pipeline is:

```text
Locked package snapshot
→ parsed modules
→ resolved names and metadata
→ typed expressions
→ dependency and priority checks
→ typed IR, rule index, and provenance map
```

The compiler must:

- Resolve imports through the package lock.
- Compile the entry module’s complete transitive import closure.
- Register every rule in that closure targeting an included query, regardless of expression reachability.
- Check argument, condition, result, and metadata types.
- Require priority endpoints to target the same query.
- Reject executable cycles and cycles among potentially enabled priority edges.
- Validate identifiers and references in standard provenance schemas.
- Preserve source spans, metadata, and the included-module inventory.

Import closure is explicit. The compiler does not discover additional law from unrelated files or installed packages.

### Input knowledge

The scenario protocol supports an observation envelope at each input field:

```text
Observation<T> =
    Known(value, evidence)
  | Missing(issue)
  | Disputed(claims)
```

Source-language field types remain ordinary types. The runtime tracks knowledge separately.

Required distinctions:

- Known false differs from a missing Boolean.
- Known absence, such as `None`, differs from a missing optional value.
- A known empty list does not establish that a real-world inventory is complete.
- Agreeing claims can combine support; differing claims remain disputed unless explicitly resolved.

Expressions propagate relevant missing or disputed inputs. Boolean operations support decisive truth values: false can establish a conjunction as false, and true can establish a disjunction as true, without resolving every operand.

An unresolved conditional does not cause the runtime to guess a branch. No rule uses failure to prove something as an implicit negative fact.

### One query-result model

Use one mutually exclusive outcome:

```text
QueryOutcome<T> =
    Resolved(value, support)
  | Inapplicable(reasons)
  | OutsideCoverage(reasons)
  | Undetermined(issues)
  | Conflicted(alternatives)
```

A `QueryResult` adds query identity, explanation references, declared coverage, and used premises.

This replaces separate applicability and value-status fields that could admit contradictory combinations.

- `OutsideCoverage` means a known scenario condition is outside the encoded package’s supported scope.
- `Inapplicable` means the encoded applicability condition is false.
- `Undetermined` identifies the unresolved stage: coverage, applicability, a rule condition, priority, or a required value.
- `Conflicted` contains different supported answers that remain after priority.
- A resolved Boolean retains its actual true or false value.

Consuming an unresolved query as a value propagates its blockers. Consuming an inapplicable or unsupported query as a value produces an explicit dependency issue; neither silently becomes false or zero.

### Rule selection

For each query invocation:

1. Evaluate `scope`. False produces `OutsideCoverage`; unresolved scope produces `Undetermined`.
2. Evaluate `applies`. False produces `Inapplicable`; unresolved applicability produces `Undetermined`.
3. Obtain the complete candidate-rule set from the compiled program.
4. Evaluate each candidate’s `when` expression.
5. Evaluate conditional priority edges.
6. Determine maximal applicable rules under the enabled priority relation.
7. Evaluate the values yielded by those maximal rules.
8. Combine equal values or report incompatible values.

Additional semantics:

- No applicable rule produces `Undetermined(NoSupportingRule)`.
- Several maximal rules yielding the same normalized value combine provenance.
- Different maximal values produce `Conflicted`.
- Unresolved candidate conditions or priority conditions conservatively block an unconditional answer for that query in v0.1.
- Values of defeated rules need not be evaluated.
- Defaults must be explicit rules.
- Source order creates no priority.
- Priority is a strict partial order scoped to a query. Its transitive relation can pass through an inapplicable intermediate rule.
- Potential priority cycles are rejected even when an author believes their conditions are mutually exclusive.

These are programming-language semantics. A legal claim that one rule outranks another must still be encoded and cited. Catala provides a useful reference for explicit exceptions and the interpretive choices involved in their ordering. [Catala exception semantics](https://book.catala-lang.org/en/2-2-conditionals-exceptions.html)

## 4. Legal models in lawlib

### Shared legal vocabulary

Create a jurisdiction-neutral shared package before US-specific packages. Its records and functions cover:

| Model | Purpose |
|---|---|
| Instrument and provision revisions | Identify legal text, enactments, amendments, and lifecycle relationships |
| Institutions and authority relationships | Describe jurisdiction, competence, delegation, and membership |
| Cases, opinions, and judgments | Separate litigation history, reasoning, and dispositions |
| Legal propositions and treatments | Identify interpreted holdings and their subsequent treatment |
| Findings and interpretations | Record factual resolutions, legal readings, and discretionary choices |
| Legal effects | Represent rights, duties, permissions, prohibitions, powers, and consequences |
| Proceedings and events | Represent procedural state and institutional acts |

Use LegalRuleML as a review checklist for legal concepts such as authority, provenance, temporality, and defeasible norms. Do not adopt its XML format or claim conformance in v0.1. [LegalRuleML specification](https://docs.oasis-open.org/legalruleml/legalruleml-core-spec/v1.0/os/legalruleml-core-spec-v1.0-os.html)

### Hierarchy and authority

Lawlib can use familiar geographic directories for browsing, including federal, state, county, and municipal materials. Package identity and legal authority remain independent of those paths.

Represent explicit relationships for:

- Geographic location.
- Membership.
- Jurisdiction over people, places, subjects, or proceedings.
- Delegation and institutional competence.
- Amendment and interpretation.
- Contextual priority and preemption.

A household has a private identity with membership and location links. It is not automatically a legal descendant of a city. Tribal, territorial, federal, state, and local systems receive their own relationships rather than being forced into one authority tree.

There is no universal numeric rank for sources and no rule that “the most local” or “the most recent” automatically wins.

### Sources beyond codified statutes

Support constitutions, enactments, statutory notes, regulations, procedural rules, executive instruments, treaties, administrative decisions, judgments, orders, and private instruments. Distinguish legal instruments from guidance, summaries, and editorial material.

The US Code is not the whole US legal system. It omits categories including agency regulations and treaties, and statutory notes can themselves contain enacted law. Effective-date provisions may appear outside the main section text. These distinctions must survive source ingestion and encoding. [Office of the Law Revision Counsel’s guide](https://uscode.house.gov/detailed_guide.html)

Definitions and cross-references also carry scope. A definition applicable to one subsection must not silently become a global definition throughout lawlib.

### Legal change and delayed commencement

Keep the following concepts distinct:

- Passage or adoption.
- Enactment, ratification, or issuance.
- Commencement or entry into force.
- Applicability to particular events, persons, periods, or proceedings.
- Amendment, suspension, repeal, and expiration.
- Transitional and savings provisions.

Store event history and revision relationships as records. Implement their consequences through ordinary functions and queries.

A provision may commence on a fixed date, after a specified event, after a calendar offset, or when a condition is met. Different provisions of one instrument may commence at different times. Unknown triggering events remain unknown.

For example, a rule adopted in 2026 with commencement in 2029 exists in the source history before 2029, while its operative conditions can remain false for earlier events.

Every run distinguishes:

1. **Scenario event dates:** when the relevant conduct or transaction occurred.
2. **Assessment context:** the forum, question, and any explicit historical-authority cutoff.
3. **Program snapshot:** the exact source references and encodings used.

A historical-authority cutoff answers which authorities the assessment is allowed to consider. It is separate from whether a later authority has an encoded effect on earlier conduct.

There is no implicit current date, latest-version selection, retroactivity rule, or repeal rule. Endpoint inclusion, savings clauses, and transition conditions are explicit.

Preserve historical revisions. If two revisions produce incompatible applicable answers, report that conflict unless encoded conditions or priority resolve it.

### Precedent and adjudication

Keep these records separate:

- `Case`: litigation identity and procedural history.
- `Opinion`: reasoning and its majority, concurrence, dissent, or other status.
- `Judgment`: disposition and its effects on specified parties or proceedings.
- `LegalProposition`: an interpreted holding or other proposition with supporting passages.
- `Treatment`: what a later authority did to a particular proposition, opinion, or judgment.

A proposition records its issue, material facts, procedural posture, relevant passages, and interpretation rationale.

Treatment records identify the treating authority, target, treatment kind, scope, and temporal reach. Vacatur, reversal, overruling, distinction, and statutory supersession must not collapse into one whole-case `valid` Boolean.

Evaluate binding force, persuasive relevance, publication status, and citation permission separately. These can depend on the court and context; the Ninth Circuit’s treatment of opinions and memorandum dispositions illustrates why publication alone cannot be a universal authority rule. [Ninth Circuit decisions](https://www.ca9.uscourts.gov/decisions/)

Holding extraction, factual analogy, standards of review, and interpretive choices can remain unresolved or use supplied findings. The engine does not automatically turn opinion text into authoritative rules.

### Findings, evidence, and discretion

A finding identifies:

- The typed issue and supplied resolution.
- Its declared decision-maker and role.
- Evidence and rationale.
- Applicable parties, events, dates, and questions.
- Provenance and any explicitly superseded finding.

Separate factual findings, legal interpretations, and discretionary choices. Preserve competing claims when recording an accepted resolution.

A scenario may select named premises or interpretation profiles for a run. These selections are included in the request and certificate. Missing selections remain explicit where the result depends on them.

The label “judge” does not authenticate a finding. Rules concerning authority to decide, burdens of proof, admissibility, and standards of review are themselves legal models. Do not assign invented numerical probabilities to legal standards.

### Legal effects and institutional processes

Represent legal effects as ordinary typed values.

An obligation can identify its actor, action, trigger, deadline, discharge conditions, and source. A power can identify its holder, authorized act, prerequisites, and resulting change in legal state.

Keep violation, liability, enforceability, invalidity, and remedy as separate questions. No automatic inference connects them.

Compatible effects are explicitly accumulated through collections or contribution queries with authored empty results. They are not competing answers merely because several obligations apply.

Procedural and institutional processes use pure state-transition functions over supplied events. Running a program can determine an encoded consequence of an act; it does not perform that act.

## 5. Sources, packages, and public interfaces

### Source references and optional storage

A standard source record contains:

```text
SourceRef {
    stable_id,
    citation,
    edition_or_version,
    locations,
    expected_sha256,
    publication_metadata,
    retrieval_metadata
}
```

An anchor identifies a source record and a locator such as a section, paragraph, page, or span. Source identity and provision identity are distinct.

Requirements:

- Published lawlib source references pin an expected digest.
- Hash the original document bytes.
- If extracted text is retained, give it a separate digest and extraction provenance.
- Stable IDs do not depend on current paths or section numbering.
- Encoding records connect source anchors to rules, rationale, assumptions, and review status.
- Renumbering changes locators and aliases without silently changing legal identity.

**No repository `sources/` directory or vendored document collection is required.** Source records can live beside the relevant `.oath` code.

Original documents may live in an optional external content-addressed cache. Private or synthetic sources can use local fixtures or cache identifiers without public hosting.

A URL and hash identify an expected version; they do not recover missing historical bytes. Archiving is therefore optional infrastructure with a separate availability guarantee.

`run` and computational `verify` neither fetch nor require original documents. Source auditing reports `Matched`, `Mismatched`, `Unavailable`, or `NotChecked` separately. Cache contents and audit status do not alter a computation’s result or certificate digest.

### Package format

Use TOML manifests and explicit locks:

```toml
[package]
name = "us.federal.tax.ira.2025"
version = "0.1.0"
language = "=0.1.0"
entry = "src/main.oath"

[dependencies.us_authority]
path = "../../../authority"
version = "=0.1.0"
```

The standard library is also versioned and pinned. Its implementation ships in the Oathkeeper repository, with no third repository required.

For v0.1:

- Resolve dependencies from explicit local locations.
- Record stable package identities, versions, content digests, and transitive dependencies.
- Treat local paths as resolution hints rather than identities.
- Require `oath lock` to create or update locks.
- Other commands validate locks without changing them.
- Never fetch packages or source documents during execution or verification.

Package digests cover normalized manifest data, included source paths and bytes, source-reference records, and dependency digests. Exclude absolute checkout locations, external document caches, and generated outputs.

Use LF checkout rules for authored text. Preserve released snapshots and distinguish a legal amendment from an encoding correction, even when both require a new package release.

### Rust API

Expose one public execution operation:

```rust
pub fn compile(
    package: &ResolvedPackage,
) -> Result<CompiledProgram, Vec<Diagnostic>>;

pub fn run(
    program: &CompiledProgram,
    request: &RunRequest,
    limits: &ExecutionLimits,
) -> Result<RunBundle, ExecutionError>;

pub fn verify(
    program: &CompiledProgram,
    request: &RunRequest,
    certificate: &Certificate,
    limits: &ExecutionLimits,
) -> Result<VerifiedRun, VerificationError>;
```

A `RunRequest` identifies one root query and its typed argument observations. Required legal context, findings, interpretation selections, and completeness assertions are explicit inputs rather than ambient state.

`RunBundle` contains the query result, explanation graph, and certificate. Query dependencies may invoke other queries; independent root questions use separate calls.

The interpreter may use an internal function named `eval`. There is no additional public operation with that name.

### CLI

Primary operations:

```text
oath run PACKAGE --request FILE --format text|json --certificate FILE
oath test PACKAGE
oath verify PACKAGE --request FILE --certificate FILE
```

Optional execution-limit configuration is accepted consistently by `run` and `verify`.

Authoring and maintenance operations:

```text
oath lock PACKAGE
oath check PACKAGE
oath coverage PACKAGE --json
oath sources check PACKAGE [--fetch]
```

Their responsibilities are:

| Command | Responsibility |
|---|---|
| `run` | Compute one scenario and produce its result and certificate |
| `test` | Run fixtures and compare results with authored expectations |
| `verify` | Independently check an existing certificate |
| `check` | Compile and perform structural validation |
| `sources check` | Audit source bytes against expected hashes; fetch only when explicitly requested |

The test harness uses the same public `run` operation. Successful semantic fixtures also pass their certificates through `verify`; the harness does not implement another evaluator.

Exit codes:

- `0`: successful operation, including a prohibited, unresolved, or conflicted legal result.
- `1`: compilation, execution, verification, test, or requested source-audit failure.
- `2`: invalid command usage.

### Serialization and limits

Use versioned JSON schemas with explicit representations for exact values:

```json
{
  "type": "rational",
  "numerator": "7",
  "denominator": "100"
}
```

Integers and rational components use canonical decimal strings. Library types serialize through their nominal type identity and ordinary structure; they do not require new kernel value tags.

Reject duplicate object keys, malformed numbers, and unknown semantic fields. Reserve an explicit metadata location for extensions.

Normalize typed values, then use RFC 8785 canonical JSON and SHA-256 with distinct domain prefixes for packages, programs, requests, results, and certificates. [RFC 8785](https://www.rfc-editor.org/rfc/rfc8785.html)

Initial deterministic execution limits per root query:

- 1,000,000 evaluation steps.
- 250,000 trace nodes.
- 100,000 entries per collection.
- 128 nested calls.
- 65,536 bits per integer numerator or denominator.

Limits are configurable and recorded. Exceeding them is an execution or verification resource error, never a legal answer. Readers also enforce bounded input size and nesting before allocating unbounded structures.

## 6. Certificates and the trust boundary

A certificate records:

- Language-semantics and certificate-format versions.
- Program, package, and request digests.
- Execution-limit profile.
- Result digest.
- Trace nodes and result roots.
- Referenced premises, interpretations, and provenance.

Trace nodes cover input reads, calculations, branches, coverage and applicability checks, candidate conditions, priority edges, selected values, conflicts, and unresolved dependencies.

The independent checker must:

1. Match the certificate to the caller-supplied program, request, and limit profile.
2. Validate node types, references, and dependency structure.
3. Check primitive calculations and branch decisions.
4. Reconstruct the query’s candidate set from program declarations.
5. Check rule applicability, priority, and selection independently.
6. Reject omitted candidates or unresolved conditions that could block the result.
7. Confirm that used premises and selections are present in the request.
8. Reconstruct the outcome, explanations, and blockers.

A certificate cannot choose its own trusted program or scenario.

The documented trusted computing base includes the compiler, IR validation, checker, exact arithmetic, hashing, and serialization. The compiler’s source-to-IR translation is not formally proved in v0.1.

Calendar and legal functions implemented in `.oath` are checked as program computations. Their domain correctness is addressed through tests, source mapping, and review rather than a hidden trusted legal oracle.

A verified certificate establishes correct execution under the identified encoding and premises. Coverage reports describe which questions and authorities that encoding claims to address. Neither mechanism proves that all relevant law has been discovered.

## 7. Initial lawlib packages

Create a shared legal-model package and these initial content packages:

```text
private.household.demo
us.authority
us.federal.constitution
us.federal.tax.ira.2025
```

Each contains `.oath` modules, source-reference records, encoding rationale, coverage declarations, review records, and scenario fixtures. Initial legal review status is explicitly `Unreviewed`.

### Household example

Encode membership, quiet hours, express emergency permission, delayed commencement, a later revision, and a separately defined consequence.

| Scenario | Expected result |
|---|---|
| Before commencement | Inapplicable |
| Applicable ordinary after-hours noise | Prohibited |
| Applicable emergency noise with accepted finding | Permitted under this policy |
| Missing emergency finding | Undetermined |
| Conflicting emergency claims | Undetermined with the dispute identified |
| Applicable incompatible rules without priority | Conflicted |
| Established violation without a consequence rule | No remedy inferred |

This example tests the mechanisms without importing unrelated public-law conclusions.

### Constitution and precedent example

Create a source map for all seven Articles and 27 Amendments. Mark source mapping separately from executable coverage.

Initially execute selected:

- Age and duration requirements.
- Voting thresholds with explicit denominators.
- Commencement provisions.
- Related institutional powers.
- A narrow proposition from *U.S. Term Limits, Inc. v. Thornton* concerning additional qualifications for congressional service.

The Twentieth Amendment example distinguishes ratification from commencement of the affected sections. The precedent example separates controlling reasoning from separate opinions and does not claim to decide overall candidate eligibility. [Amendment text](https://www.archives.gov/founding-docs/amendments-11-27), [Thornton opinion text](https://www.leg.state.nv.us/Division/Research/Publications/ResearchBriefs/TermLimitsBrief/USTermLimitsIncvThornton.pdf)

Use clearly labeled synthetic fixtures for proposition-specific treatment, forum-dependent authority, unresolved analogy, and conflicting interpretations.

### IRA example

Use tax year 2025, with separate questions for contribution requirements, deductibility or nondeductible treatment, conversion treatment, taxable conversion amount, and supported reporting requirements.

Require complete inventories of relevant accounts, basis, balances, and transactions. Keep contribution year distinct from transaction dates.

Initially support ordinary same-year transactions involving the taxpayer’s own accounts and qualifying compensation. Exclude inherited accounts, required minimum distributions, corrections, outstanding rollovers, cross-year complications, unsupported distributions, special transactions, and state taxation.

Reconcile the encoded statutory rules with the dated [2025 Form 8606](https://www.irs.gov/pub/irs-prior/f8606--2025.pdf) and [instructions](https://www.irs.gov/pub/irs-prior/i8606--2025.pdf). Exact arithmetic and any form-specific rounding remain separate, documented operations.

Required fixtures, assuming the supported scenario and no other distributions:

| Basis | Conversion | Relevant remaining year-end balance | Taxable conversion |
|---:|---:|---:|---:|
| $7,000 | $7,000 | $0 | $0 |
| $7,000 | $7,000 | $93,000 | $6,510 |

Also test missing balances, incomplete inventories, prior basis, unsupported transactions, and rounding boundaries.

The package answers these identified questions. It does not replace them with an unrestricted “backdoor Roth is legal” Boolean.

## 8. Implementation sequence and acceptance tests

### Milestone 1: specification and repository foundation

Create both repositories, the four-crate workspace, standard-library skeleton, package formats, and shared legal-model package.

Write conformance fixtures before implementing substantive law. Freeze the initial grammar and outcome semantics against these fixtures.

### Milestone 2: compiler and deterministic interpreter

Implement parsing, typing, import resolution, typed IR, exact values, knowledge propagation, queries, and priority.

Acceptance includes:

- Useful source spans on errors.
- Rejected executable and priority cycles.
- Stable legal outcomes under rule declaration reordering.
- Distinct known absence, missing facts, and disputed claims.
- Explicit arithmetic and resource-limit errors.
- No implicit clock, filesystem, or network dependencies.

### Milestone 3: provenance and independent verification

Implement metadata preservation, explanations, certificates, and the separate checker.

Required adversarial cases include:

- Altered input or result.
- Forged arithmetic.
- Omitted candidate or unresolved exception.
- Invented premise.
- Changed priority edge.
- Mismatched program or request.
- Cyclic or malformed trace.
- Excessive nesting or resource consumption.

Test a three-rule priority chain where the middle rule is inapplicable, confirming the specified transitive ordering.

### Milestone 4: legal models and three examples

Complete the household, constitutional, and IRA packages with explicit source anchors, assumptions, scope predicates, expected results, and review status.

Temporal tests must cover:

- Adoption before commencement.
- Different commencement dates within one instrument.
- Unknown commencement triggers.
- Amendments and overlapping revisions.
- Savings and transition conditions.
- Historical-authority cutoffs.
- Leap days, month overflow, endpoint inclusion, and an example legal deadline convention.

### Milestone 5: reproducible developer release

Complete CLI/API documentation, package locking, source auditing, coverage reporting, and Windows/Linux CI.

Additional checks:

- Property tests for rational normalization, rounding, and canonical serialization.
- Evaluator/checker agreement plus independently authored expected results.
- Parser and certificate-reader fuzzing.
- Offline execution with an empty source cache.
- Source-hash mismatch and unavailable-source reporting.
- Identical results and certificates for identical program, request, limits, and implementation versions.
- Package portability across checkout locations.

External source audits run separately from deterministic computational tests.

## 9. Expansion and maintenance

Maintain a coverage matrix for sources and history, institutions, lawmaking, rights and liabilities, interpretation, evidence, procedure, adjudication, appeals, enforcement, and remedies.

For each area, track separately:

- Representation support.
- Executable coverage.
- Real examples.
- Unresolved mechanisms.
- Source-audit status.
- Legal-review status.

Coverage records must distinguish a reviewed exclusion from a source or question that has never been investigated. An inventory of encoded rules is not evidence that no other rules exist.

Preserve historical releases and generate dependency-based impact reports for legal amendments, changed interpretations, source revisions, and implementation corrections.

The initial executable subset rejects recursion and conservatively blocks on unresolved rule conditions. These are documented v0.1 limitations. Extend the language only when a concrete legal model demonstrates the need, updating semantics, compiler, interpreter, checker, and conformance fixtures together.

The resulting foundation supports incremental formalization of the entire legal system while keeping every computed answer tied to its program, sources, scope, premises, and unresolved questions.
