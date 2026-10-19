# Oathkeeper language specification — 0.1

Status: experimental, executable reference implementation in
`crates/oathkeeper-core`. Normative version identifier: `oathkeeper/0.1`.
Specification changes that alter accepted proofs require a new language version.
Kernel package versions and legal package/rule versions are separate identities.

## 1. Transport and vocabulary

`.oath` files use TOML syntax, including arrays of tables and inline tables. JSON
is an equivalent transport for the same typed data model. This is a ground
propositional proof language: it has no variables, quantification, arithmetic,
dependent types, higher-order functions, imports, or executable user code.
Future elaborators may produce this proof representation without entering the
trusted kernel. See `examples/hoa.oath` for a complete executable program.

The root fields are `language`, `package`, `version`, `jurisdictions`, `rules`,
`facts`, `attestations`, `oracle_policy`, `query`, and `proof`. All are required;
empty collections are explicit. Unknown fields, duplicate object fields, invalid
types, and unknown enum tags are errors. TOML rejects duplicate table keys.
Identifiers are 1–256 ASCII bytes from `[A-Za-z0-9._:/-]`. Package and rule version
labels use the same syntax; semantic versioning is recommended for packages but
not enforced. Native quoted RFC3339 strings are required for timestamps.

A claim is `{ atom = "case.notice_delivered", negative = false }`.
`negative` defaults to false; all other claim fields are required. Atom names
are opaque and case-sensitive: `case.age_ge_35` does not perform arithmetic.
Negative claims are explicit propositions. Absence of a claim does not prove its
negation. There is no explosion rule, implicit negation, or automatic inference
from English names.

The Rust structures in `crates/oathkeeper-core/src/model.rs` define the exhaustive
wire fields. In JSON, optional `effective_until` may be absent or null. In TOML it
is omitted for an open interval. `parents` and `negative` have defaults; other
arrays must be present. Metadata text and evidence bodies must remain outside
the language unless explicitly represented by the schema.

## 2. Jurisdictions and authority

A jurisdiction has `id`, `name`, and `parents` (default `[]`). Its directed edges
point toward inherited applicability contexts. The entire graph must be acyclic;
every parent and every rule jurisdiction must exist. Multiple roots and multiple
parents are supported, without hardcoded US names or geographic levels.

The applicable set is the query jurisdiction plus its transitive parents. A rule
outside that set cannot support a proof. A rule's stable identity cannot move to
another jurisdiction between versions. IDs are globally namespaced in a bundle.

No priority derives from depth, recency, or array order. HOA covenants, tribal
sovereignty, federalism, agency authority, treaty application, and private
agreements cannot generally be reduced to one geographic tree. The DAG models a
declared applicability context, not a determination that the context is legally
correct. The caller must supply reviewed edges and explicit legal premises.

## 3. Rules and bitemporal snapshots

A rule contains `id`, `version`, `jurisdiction`, `effective_from`, optional
`effective_until`, `recorded_at`, `source`, ordered `premises`, and `conclusion`.
`source` contains nonempty `citation`, nonempty `uri`, and a lowercase 64-character
`sha256`. The digest references the exact source bytes; source retrieval and
matching the digest are the reviewer's responsibility in this version.

Each `(id, version)` pair is unique. Content under that identity is immutable by
library policy; the checker has no global registry to detect changes between
separate bundles. Use committed snapshots and the bundle digest to detect them.

A rule is active exactly when:

1. Its jurisdiction belongs to the query's applicable set.
2. `effective_from <= query.as_of < effective_until` (if an end exists).
3. `recorded_at <= query.known_at`.

Timestamps are compared as RFC3339 instants, including offsets. Endpoints are
exclusive at the end; a rule's end must follow its start. Retroactive rules are
representable: recording time need not precede legal time. No condition requires
`as_of <= known_at`, so prospective snapshots are possible. The caller chooses
the intended time semantics and evidence relevance.

At most one version of a given rule ID may be active in a snapshot. Ambiguity is
an error even if that rule is unused. No latest-version selection occurs. To
encode a repeal or amendment, close the old interval and add an immutable new
version. Correcting a historically erroneous package requires a new package
snapshot; v0.1 does not model transaction-time retractions in an append-only log.

## 4. Facts and probabilistic oracles

A fact has `id`, `claim`, `evidence_sha256`, and `recorded_at`. Facts are explicitly
trusted axioms supplied with a case, not automatically established truths. A
referenced fact must be recorded by `known_at`. There is no implicit legal-time
interval for a fact; encode time relevance into propositions/premises as needed.

An oracle implements the external partial probabilistic function:

```text
evaluate(oracle_id, question, evidence_bytes, provider_version)
    -> abstain | { polarity, confidence in [0,1], provenance }
```

The agreed atom identifies the proposition being assessed. `polarity` selects
that atom or its explicit negation. Abstention produces no usable attestation.
The same request may produce different outputs; each output is frozen with a
distinct ID. The kernel checks a snapshot and never re-runs a stochastic function.

An attestation has `id`, `oracle`, `provider`, `provider_version`, `question`,
`claim`, finite `confidence` in `[0,1]`, `evidence_sha256`, and `recorded_at`.
A unique agreement in `oracle_policy` contains `oracle`, `atom`, `question`,
`evidence_sha256`, nonempty `providers = [{ id, version }]`, and finite
`min_confidence` in `[0,1]`.

For each used attestation the checker requires:

- An agreement for that oracle ID.
- Exact provider ID and version membership in the agreement.
- Exact question, evidence digest, and atom match to the agreement.
- Exact signed claim match to the proof step.
- Confidence at least the agreed minimum and recording time by `known_at`.

Agreements bind the question and evidence, preventing accidental cross-case
reuse. They are inputs, not authenticated contracts. v0.1 does not verify digital
signatures, identities, evidence contents, model calibration, quorum protocols,
or whether humans actually agreed. Those belong in an authenticated ingestion
layer. A policy may permit positive or negative output for its atom.

## 5. Proof judgments

A query supplies `jurisdiction`, `as_of`, `known_at`, signed `goal`, and `root`
(the ID of its final proof step). Each proof step has a unique `id`, signed
`claim`, and tagged `reason`. Steps are checked in supplied topological order:

```text
Fact:   fact(f).claim = C; fact visible at cutoff
        ----------------------------------------
        { claim = C, reason = { kind = "fact", fact = f } }

Oracle: approved attestation(a).claim = C; visible at cutoff
        --------------------------------------------------
        { claim = C, reason = { kind = "oracle", attestation = a } }

Apply:  active rule(r, v) = [C1, ..., Cn] -> C
        earlier step(s1) = C1 ... earlier step(sn) = Cn
        -----------------------------------------------------------
        { claim = C, reason = { kind = "apply", rule = r, version = v,
                               premises = [s1, ..., sn] } }
```

Premise matching is exact and ordered; repeated premises may reference the same
step. Empty-premise rules are explicit trusted legal axioms. Forward references
and cyclic proofs fail. The root must prove the query's exact signed goal. All
submitted steps must be in the root dependency closure, disallowing padding.

If both polarities of an atom occur in a submitted proof, verification fails.
This detects conflicts in the submitted derivation, **not every possible conflict
in the entire legal corpus**. The checker does not search for counterproofs or
prove completeness. A separate valid counterproof can exist in inconsistent
inputs. Preemption and exceptions need positive applicability conditions whose
establishment is itself part of the proof. v0.1 has no defeasible override engine.

Every proof can be checked by induction over its steps: each claim is a supplied
axiom, a permitted external assumption, or the conclusion of a declared rule
whose premises were already checked. This is conditional syntactic soundness for
this ground rule calculus, not a proof of correspondence to real law.

## 6. Reports, provenance, and normalized verifiability

Successful reports include the kernel/language/package versions, exact goal and
time context, applicable jurisdiction IDs, rule version pairs used, fact IDs used,
and attestation IDs used. Dependency lists are sorted and deduplicated.

Let `O(p)` be the union of attestation IDs reachable from the proof root:

```text
O(fact)        = empty
O(oracle a)    = {a}
O(apply r ps)  = union(O(p) for p in ps)
N             = |O(root)|
V(root)       = 1 / (1 + N)
score_version = "oracle-count-v1"
```

`V` lies in `(0,1]`; zero is its asymptotic lower bound. It cannot be increased by
adding deterministic proof steps, and reusing an attestation counts once. Two
distinct attestations from the same provider count twice. Independent oracle
assumptions do not imply statistically independent events. The score does not
multiply confidence values or estimate a correctness probability.

The count covers oracle assumptions used by this proof, not a lower bound over
all possible proofs. It does not count how many providers were consulted before
selecting an attestation. Reported fact assumptions are a separate trust surface.
Input authors could reclassify opinions as facts or unsound legal axioms to obtain
a high score; library governance and pinned evaluation contexts must prevent that.

Status is `verified_from_facts` if `N = 0`, or `verified_conditional` otherwise.
Neither status asserts legal authority, case completeness, or court acceptance.
Errors produce no successful report. The CLI exits nonzero; Python raises
`ValueError`.

`bundle_sha256` hashes the compact `serde_json` serialization of the parsed Bundle
under this kernel version. Struct field order follows `model.rs`; claim defaults
are materialized and absent rule ends serialize as null. Array order and string
timestamp spelling remain significant. This is a **version-bound snapshot
fingerprint**, not RFC8785 canonical JSON or an interimplementation signature
format. It binds policy, facts, rules, proof, query, and attestations together.
Pin raw input hashes/commits as well for durable interchange and audit.

## 7. Resource limits and trust

The checker accepts in-memory bundles. It validates the entire input structure
but only checks evidence authorization and cutoff eligibility for referenced
facts and attestations. Unused future evidence can coexist in a package without
supporting the current proof. Source and evidence hashes are format-checked for
all entries. The kernel has no external effects.

v0.1 does not enforce input byte, graph-depth, step-count, or execution-time limits.
Services handling hostile inputs must bound these externally. Jurisdiction
traversal is recursive. The parser and dependencies are part of the trusted
computing base; the implementation has tests, not a machine-checked metaproof.
