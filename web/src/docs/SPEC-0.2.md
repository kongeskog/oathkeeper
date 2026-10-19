# Oathkeeper language specification — 0.2

Status: experimental. Language identifier `oathkeeper/0.2`; kernel package 0.2.0.
This extends the [0.1 calculus](SPEC.md) while keeping 0.1 input support. Features
below require the 0.2 identifier. Optional empty fields are omitted from bundle
serialization so the original tutorial's 0.1 snapshot fingerprint is preserved.
New report fields are additive. The kernel has not been formally proved correct.

## Terms and parameterized rules

A claim has `atom`, `negative` (default false), and `arguments` (default empty).
Atom names do not encode arguments; arguments participate in exact equality and
in contradiction checks. Terms are tagged values:

```json
{"kind":"symbol","value":"long.98824-2"}
{"kind":"integer","value":72}
{"kind":"variable","name":"matter"}
```

Symbols/variable names follow the identifier restrictions from 0.1. Integers
are signed 64-bit mathematical integers, with no units or arithmetic overflow
semantics because arithmetic is not yet implemented. Terms do not nest.

Each rule declares `parameters = [{name, sort}]`, where `sort` is `symbol` or
`integer`. Parameter names are unique; every rule variable must be declared.
The rule's premises, conclusion, and authority scope can contain those variables.
A rule application supplies `substitution`, an object mapping **exactly** all
declared parameters to ground terms of the declared sorts. Extra bindings,
missing bindings, and variable-valued substitutions fail. Each referenced proof
claim must match the corresponding fully instantiated premise, including all
arguments and polarity. The conclusion must match the instantiated conclusion.

Proof steps, query goals, facts, attestations, and oracle policy arguments must
be ground. There is no implicit unification, variable inference, quantification,
subtyping, or global predicate-signature registry. Sort checking applies to
declared rule parameters. A constant can be used only if the ground claim matches.

For oracle agreements, `arguments` (default empty) is now part of the request
binding along with atom, question, evidence digest, provider/version, and minimum
confidence. An attestation for one case argument cannot satisfy an agreement
for another case argument.

## Checked integer comparisons

The proof reason `{"kind":"compare","operator":"greater_than"}` proves exactly
the claim `oathkeeper.integer.gt` with two integer arguments. `less_than` and
`equal` use `oathkeeper.integer.lt` and `oathkeeper.integer.eq`, respectively.
`negative` must equal the negation of the computed comparison result. Thus a
negative `gt(72,72)` claim is accepted and a positive one is rejected.

Claims in the reserved `oathkeeper.integer.` namespace can be established only
by this reason, preventing a fact or legal rule from overriding checked integer
semantics. Comparisons have no evidence dependencies. Evidence for a measured
quantity remains a separate input: proving `73 > 72` does not prove that a
vehicle was actually parked for 73 hours. Unit conversions and fractional time
are not implicit.

## Authorities and scope

0.2 bundles add `authorities` and `authority_assessments` arrays (default empty).
Every 0.2 rule must reference an authority; purely numeric compare steps need
none. Queries require `forum`, an explicit identifier distinct from geographic
jurisdiction. The forum is a policy context, not an automatically inferred court.

An authority has:

- Unique `id`, `kind` (`case`, `ordinance`, `statute`, `constitution`), existing
  `jurisdiction`, and `issuing_body` identifier.
- `source` with citation, URI, and SHA-256.
- `publication`: `published`/`unpublished` for a case, `enacted` for legislation.
- `decided_at`: required RFC3339 timestamp for cases; absent/null for legislation.
  Enactment/effective history is distinct and is represented in rule intervals
  and source metadata, not by inventing a judicial decision date for legislation.

Each rule's `authority` reference has `authority` (ID), `role` (`holding` for a
case or `text` for legislation), nonempty `pinpoint`, and a signed `scope` claim.
The scope must be an explicit member of the rule's premises. Consequently it
must be established by a prior proof step, with the same substitution as the
other premises. Scope cannot be satisfied just by matching case names or facts
that superficially resemble the original litigation.

The rule and authority must have the same jurisdiction and source URI/hash.
Pinpoints identify the encoded passage; the kernel does not interpret English
or verify that the passage supports the rule. Source hashing remains a byte
integrity mechanism, not legal review.

## Authority assessments and treatment

An assessment has unique `id`, `authority`, exact `rule`/`rule_version`, `forum`,
legal `as_of`, recording `recorded_at`, `force` (`binding` or `persuasive`),
`treatment`, `review_status`, `source`, and nonempty `rationale`.

Treatment is `operative`, `overruled`, `vacated`, `repealed`, or `superseded`.
Review status is `machine_assisted_unreviewed` or `reviewed`. These are supplied
assessments; v0.2 does not authenticate a reviewer or search a citator. Even a
`reviewed` label is an assertion whose authenticity belongs in ingestion policy.

For each used rule, select assessments with the exact rule/version, exact query
forum, equal `as_of` instant, and `recorded_at <= known_at`. The most recently
recorded qualifying assessment controls. A tie at the latest timestamp is an
error. The selected assessment must reference the rule's authority and have
`operative` treatment. Unpublished cases cannot be assigned binding force.
Published cases may be binding or persuasive under the declared forum policy;
publication alone does not establish binding force. A used case must have been
decided by both the legal time and knowledge cutoff.

All assessment references and declared rule-authority/source relationships are
validated even when unused. Eligibility/treatment is enforced on actual rule
applications. Negative treatment is per rule/version, allowing a future corpus
to distinguish treatment of different holdings from the same case.

This mechanism enforces an auditable supplied policy. It does not automatically
determine vertical/horizontal precedent, distinguish dictum by reading an
opinion, decide preemption, or certify current good law. A positive historical
assessment does not extend to another date or forum. Missing assessments fail
instead of being inferred indefinitely. Different holding scopes need separate
rules and explicit policy records.

## Fact provenance and assumption reports

Every 0.2 fact requires `basis` containing:

- `kind`: `record_fact`, `judicial_finding`, or `scope_assumption`.
- `source`: citation, URI, and SHA-256 matching the fact's evidence digest.
- Nonempty `pinpoint` and `explanation`.

A judicial finding records what a court determined; it does not transform a
contested evaluative judgment into independently verified data. For historical
case reconstruction that finding can be trusted input, with attribution. For a
new case, establish a new finding or use a justified oracle attestation instead.
Missing facts never imply negative exception claims.

Reports add full `authorities_used`, `authority_assessments_used`, and
`fact_assumptions` records, plus counts of unreviewed authority assessments and
explicit scope assumptions. The original `facts_used`, `rules_used`, and
oracle dependency lists remain. The oracle-count score and its version are
unchanged: judicial findings and scope assumptions remain visible fact axioms,
not silently manufactured oracle outputs with invented confidence values.

Acceptance and score are conditional on all these declared inputs. A score of
1.0 can coexist with unreviewed legal interpretation and subjective judicial
findings. Use the full report when deciding what a proof establishes.

## Reusable modules and case composition

The JSON `LawModule` schema contains `language`, `package`, `version`,
`jurisdictions`, `rules`, `authorities`, and `authority_assessments`.
The JSON `CaseFile` schema contains `facts`, `attestations`, `oracle_policy`,
`query`, and `proof`. Unknown fields are rejected in both. Composing the two
produces a normal bundle; the kernel then verifies it. Cases cannot override
module-owned laws or assessments. Transitive imports are not implemented.

Rust exposes `assemble_case(module_json, case_json)`. The CLI supports
`verify-case` and `normalize-case`; normalization first verifies the result.
Python exposes `verify_case(module_json, case_json)`, returning JSON or raising
`ValueError`, through the same Rust kernel.

Module files and case files are authoring/packaging formats, not a new trust
source. An RL host must still freeze the law module and case context before
accepting candidate proof steps. Module versioning alone cannot prevent an
author from changing bytes under an existing ID; pin snapshots and hashes.

## Source locks and coverage

The separate Python auditor checks `oathkeeper-source-lock/1` records, local
snapshot hashes, source/evidence references, coverage rule IDs, and every listed
case through the Rust CLI. A lock records exact bytes/representation, retrieval
URI, official URI, retrieval time, extraction method, source status, and rights.
Normalized opinion text must never be represented as the hash of a raw PDF or
an entire HTML response. Replay downloads reject upstream changes.

`coverage.json` is an explicit inventory of encoded rules, examples, omissions,
review status, and completeness limitations. It is not a kernel proof of legal
completeness. The current Seattle inventory declares both inventory and coverage
incomplete and reports no percentage without a known denominator.

All resource-limit, source-authentication, corpus-consistency, and legal-fidelity
limitations from 0.1 remain unless explicitly changed here. See
[SEATTLE_PLAN.md](SEATTLE_PLAN.md) for constructs still needed for comprehensive
Seattle law.
