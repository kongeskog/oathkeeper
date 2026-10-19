# Oathkeeper 0.4: conditional arguments and evidence

Status: experimental. Rust, CLI, Python, and WebAssembly use the same checker.
This specification extends the [0.3 syntax](SPEC-0.3.md). All canonical lawlib
authoring files now use `oathkeeper "0.4";`. Imported files must use the same
language version. Older input formats remain readable by the verifier.

## The judgment and its limits

A checked derivation establishes a signed proposition under the supplied
formalization, case premises, oracle agreements, authority assessments, forum,
and time selections. Legal rules remain sourced, trusted axioms. A citation does
not prove that the rule faithfully translates its source. A court finding in a
reconstruction is an attributed input, not a new independent merits determination.

All successful reports have `status: derivation_verified`. No report contains a
normalized verifiability score. The `assurance` object separates derivation status,
legal fidelity, corpus completeness, evidence authenticity, oracle authentication,
counterargument search, and judicial acceptance. The last six are not established
by a successful derivation. Reported review labels are not authenticated approvals.
Raw fact, finding, assumption, request, and response counts remain diagnostics.

## Kernel-enforced types

The kernel independently checks nominal types, signatures, substitutions, and
all supplied declarations, including direct JSON. Quantities add exact signed
64-bit values with a unit tag:

```oath
predicate parking.duration(Matter, quantity(ms));
// Term syntax: 259200000[ms]
```

The IR term is `{"kind":"quantity","value":259200000,"unit":"ms"}` and
its sort is `{"quantity":{"unit":"ms"}}`. Unit names are exact identifiers.
No implicit conversions, floating-point arithmetic, currency rounding, calendar
arithmetic, or measurement-error assumptions are supplied. Library conventions
must define each unit. The Seattle pilot uses elapsed milliseconds.

Built-ins `oathkeeper.quantity.gt`, `.lt`, and `.eq` require two quantities with
the same unit. Proof reasons are `compare quantity_greater_than`,
`quantity_less_than`, and `quantity_equal`. Integer comparison reasons are
unchanged. An explicitly negative comparison is accepted only when the comparison
is false. Facts, oracles, and legal rules cannot bypass checked built-ins.

## Four time selections

Version 0.4 queries require:

| Field | Meaning |
| --- | --- |
| `event_at` | Event RFC3339 instant or ISO calendar date; date-only precision does not invent a time of day |
| `as_of` | Selected governing-law instant, retained for API continuity |
| `authority_at` | Instant at which authority availability and treatment are assessed |
| `known_at` | Recording cutoff for supplied rules, evidence, responses, assessments, and withdrawals |
| `temporal_rationale` | Explicit explanation of the selected relationship between these times |
| `purpose` | `historical_reconstruction`, `new_matter_assessment`, or `synthetic_test` |

Each rule requires `time_basis law_time;` or `time_basis authority_time;`.
Effective intervals use that selected clock. Authority assessments match the exact
forum and `authority_at`; the latest visible recorded assessment controls. A case
authority must have been decided by both `authority_at` and `known_at`.

`event_at` is descriptive context. The checker does not infer retroactivity or
choose governing law from it. The supplied rationale is reviewable, not proved.
The Long reconstruction selects the 2016 statutory context and the August 2021
authority context explicitly. No query certifies present-day law.

## Evidence and withdrawals

Fact kinds include `record_fact`, `judicial_finding`, `scope_assumption`,
`interpretation_assumption`, `party_allegation`, and `stipulated_fact`.
Every kind remains a supplied premise; labeling an allegation does not adjudicate
it. The report preserves the category, source, pinpoint, and explanation.

A case may add a withdrawal event without deleting its old evidence:

```text
retraction withdrawal-id {
  target_kind fact;
  target fact-id;
  recorded_at "2026-10-19T00:00:00Z";
  source source-id;
  rationale "Reason for the withdrawal";
}
```

The target kind is `fact` or `attestation`. It must identify existing evidence;
the withdrawal cannot predate that evidence. A withdrawal visible at `known_at`
blocks use of its target. Earlier cutoffs can replay the original proof.
A replacement is new evidence with a new ID. Withdrawal never proves the opposite.
The host must preserve event history: the kernel cannot discover omitted events.

## Oracle protocols

A 0.4 policy requires a protocol, for example:

```text
protocol {
  role legal_assessment;
  agreement unanimous;
  procedure "Both named reviewers assess the same frozen record; either may abstain.";
};
```

Roles are `observation`, `classification`, `legal_assessment`, and
`institutional_determination`. They describe the input's intended use and do not
confer institutional authority. Agreements are `single` (exactly one named
provider) and `unanimous` (all named providers must assert the same claim).

Each response supplies `disposition asserted;` or `disposition abstained;`.
At most one live response per request/provider/version is permitted at the
cutoff. Revisions require withdrawing the old response. Responses must match the
question, evidence digest, predicate, subject arguments, and agreed provider.
An abstention's claim identifies the requested proposition but asserts no polarity.

The protocol reports `pending`, `abstained`, `disputed`, `threshold_not_met`, or
`accepted_under_protocol`. Only accepted assertions can supply proof premises.
A unanimous oracle step depends on every contributing response, even though its
proof reason references one of them. A model cannot select the favorable reviewer.

Numeric `confidence` and `min_confidence` are optional. If either is used, the
protocol requires `confidence_meaning`. Thresholds do not establish calibration,
legal burdens, or the probability of a correct legal result. Confidence is never
multiplied across responses. Provider identity and evidence authenticity require
verification outside the kernel; agreement is checked only over supplied records.

## Supporting and opposing arguments

`analyze(bundle, options)` constructs proof candidates outside the checker, then
independently checks every returned certificate with `verify`.

| Status | Observed support for the query and its explicit opposite |
| --- | --- |
| `supported` | A supporting argument was found |
| `opposed` | An opposing argument was found |
| `both` | Separately consistent arguments were found for both |
| `neither` | Neither argument was found |

These are support states, not judicial dispositions. Inspect search completeness
before interpreting a missing argument. Opposite propositions may coexist in the
input; a single derivation cannot contain both. There is no inference from
contradiction, and no negation from missing evidence.

Search grounds rules over the supplied constants and literal terms appearing in
rules, evidence, requests, and the goal. It explores finite acyclic derivations,
preserving alternative consistent proofs. Budgets bound groundings, nodes, depth,
argument combinations, and proof size. `complete_for_ground_domain` means no
search budget was reached within this finite supplied vocabulary. It does not
mean the corpus or legal vocabulary is complete, or that every real-world legal
argument was considered. Any truncation is explicit and cannot prove negation.

`open_requirements` lists unsupported propositions in explored argument paths.
They are not a cumulative list of legally mandatory evidence: different paths
may require different premises. `excluded_rules` explains temporal, forum,
jurisdiction, and treatment exclusions. The analysis includes checked supporting
and opposing proofs, protocol states, and a hash of the proof-independent context.

Sensitivity withholds each used fact or entire oracle request and reruns search.
It considers alternative proofs, rather than simply breaking the displayed graph.
The result is a hypothetical input intervention, not a real evidence withdrawal.
Its own completeness and dependency budget are explicit. It does not measure
legal confidence or causal effects in the real world.

## Authority and legal policy

Imports select packages; jurisdiction graph edges collect declared applicability.
Neither depth nor import order assigns legal priority. Binding and persuasive
labels remain explicit reported assessments. Version 0.4 removes the universal
kernel ban on treating unpublished authorities as binding; such rules belong in
reviewed forum-specific policy. Legacy versions retain their original restriction.

There is no general defeasible winner-selection algorithm, evidentiary burden
allocation, or modal logic of obligations in 0.4. The pilot uses explicit
qualifications and separately sourced exceptions. A source-backed override or
procedural decision requires its own justified formalization. `both` and `neither`
are useful outcomes, not errors to conceal.

## Review and source maintenance

Compilation preflights 0.4 module metadata and links in addition to signatures.
It does not supply a missing proof or certify source interpretation.
`tools/review_packet.py` produces premise mappings, source references, unresolved
questions, and exact snapshot/rule digests. Legal and formalization review records
are separate. Approved records require an identified independent reviewer and
the exact current snapshot; stale approvals fail the audit. These are declared
identities, not cryptographically authenticated credentials.

`tools/compare_snapshots.py` identifies changed declarations and proof dependencies
needing replay. New rules may introduce counterarguments even when an old proof
still verifies, so reassessment is tracked separately. Source-byte updates do not
automatically establish a change in law; a reviewer must classify the change.

## Compatibility and trust

0.1–0.3 inputs remain supported. The **0.4 report API is intentionally changed for
all inputs**: `verifiability` and `score_version` are removed, and statuses become
`derivation_verified`. Consumers should use `assurance` and raw dependencies.
Archived 0.3 source fixtures retain their previous input semantics.

The language compiler, search, and UI are untrusted relative to the derivation
checker: they emit explicit typed certificates. The implementation is tested,
not itself formally verified. See [soundness scope](SOUNDNESS.md),
[research decisions](RESEARCH.md), and [training boundaries](TRAINING.md).

## Resource boundaries

Serialized bundle parsing and module/case JSON composition accept at most 16 MiB. The browser binding caps input at 4 MiB. Kernel limits include 2,000 rules, 4,000 facts, 2,000 proof steps, 500 attestations, 500 oracle policies, 4,000 retractions, 128 jurisdictions, 1,024 types, 4,000 predicates, 10,000 constants, 2,000 authorities, and 8,000 authority assessments. Predicates have at most 32 arguments; rules have at most 16 parameters and 64 premises. These are implementation limits, not legal restrictions. Search budgets and browser timeouts add separate limits.

## Additive kernel 0.4.1 arithmetic extension

Language 0.4 modules may require kernel 0.4.1. Older kernels reject unknown operators; no fallback treats their results as facts. `by compute quantity_add`, `quantity_subtract`, and `quantity_allocate` introduce checked positive relations:

- `oathkeeper.quantity.add(a, b, result)` checks signed i64 addition without overflow.
- `oathkeeper.quantity.subtract(a, b, result)` checks signed i64 subtraction without overflow.
- `oathkeeper.quantity.allocate(amount, basis, total, result)` checks `round_half_up(amount * min(basis,total) / total)` using exact i128 intermediate arithmetic. All arguments share a unit; `0 <= amount <= total`, `basis >= 0`, and `total > 0`. This is a general allocation operation, not an assertion of tax law. It rounds to the integer unit explicitly selected by the author (USD cents in the tax pilot).

Negative arithmetic relations, unit mismatches, zero denominators, malformed arity and overflow are rejected. Reserved arithmetic predicates cannot be established by facts, oracles, or legal-rule conclusions. Ground arithmetic premises are available to bounded argument search; the tax pilot uses a deterministic witness builder followed by full kernel replay, without claiming exhaustive argument search.

`administrative_guidance` distinguishes forms and agency instructions from legislation. It requires `published` publication and no judicial decision date, and uses the authority role `text`. Authority force remains an explicit reviewed-or-unreviewed assessment; guidance is not automatically binding. The tax pilot records persuasive, unreviewed assessments.
