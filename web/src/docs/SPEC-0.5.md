# Oathkeeper 0.5: bounded coverage and community review

Status: experimental. This version adds explicit coverage contracts, exception prerequisites and interpretation selection to the [0.4 calculus](SPEC-0.4.md). Older sources retain their original behavior. Sources in an import graph use the same language version. Features introduced here are rejected when inserted into older-version JSON.

## Kernel-enforced types

Nominal types, predicate signatures and exact integer quantities remain enforced by the Rust checker, including direct JSON inputs. Arithmetic requires matching units and checked operations. Coverage parameters use those same types; every parameter must be bound by the selected goal.

## Coverage is a contract

A module can declare `coverage name { ... }`, and a case selects it with `coverage "name";` inside `query`. A contract contains:

- A question, typed parameters and a `goal predicate($parameter, ...);` pattern. Every parameter occurs in that goal.
- A `conclusion_kind`: factual, permission, prohibition, obligation, liability, remedy or calculation. These labels do not imply one another; absence of a prohibition never constructs permission.
- Explicit jurisdictions and forums, plus the half-open governing-law interval `law_from <= as_of < law_until`. The other three clocks retain their meanings.
- Named `requirements`, each with an exact goal and explanation, plus a source inventory, exclusions and known omissions.

See the real Seattle and IRA declarations in the pinned lawlib for full syntax. Metadata blocks use ordinary `.oath` fields and claim syntax; JSON is compiled interchange.

`analyze` reports scope as `not_declared`, `conditions_supported`, `outside_scope`, `needs_information`, or `conflicting_scope_evidence`. It derives both polarities of every scope requirement within explicit search bounds. Truncation cannot establish a missing fact. Scope supported means the declared premises have supporting arguments within that search; it does not validate their truth or establish complete legal coverage.

The ordinary `verify` API continues to check only a derivation. `verify_scoped` additionally checks a separate proof for every coverage requirement against the same fixed context, binding variables from the original goal. It rejects missing/extra condition proofs and out-of-scope goals, dates, jurisdictions and forums. It proves the conditions, not the absence of counterarguments. Reports carry the contract and frozen-context digest.

## Explicit exceptions

Within a rule, `unless exception($person);` adds an exception declaration. Applying the rule requires a checked **explicit negative claim** for each exception after all ordinary `requires` premises, in declaration order. Unknown exceptions remain unresolved. There is no negation-as-failure, inferred override or probabilistic threshold hidden in this operation.

The rule's JSON has `exceptions`; the checker, type checker and search all enforce the additional premises. Competing evidence may still support separate conditional arguments. These are not determinations of a legal winner.

## Alternative interpretations and authority

`interpretation name { issue "issue-id"; description "..."; source sourceName; pinpoint "..."; }` records a sourced reading. A rule may name it with `interpretation "name";`. The query explicitly selects a list with `interpretations ["name"];`. Unselected readings cannot be applied; selecting two readings for one issue is rejected. Rerun the same matter under the other selection to compare alternatives. The selection is part of the frozen context and task identity.

Jurisdiction ancestry continues to collect potential applicability only. Preemption and exceptions must be explicit sourced propositions and prerequisites. This release does not invent a universal ordering of courts, legal burdens, procedure or remedies.

## Frozen evaluation

Rust `freeze_task` validates host inputs, optionally checks a reference proof, and removes it from the task. The identity includes the entire context, fixed goal/root, interpretation choices and evaluation policy. `evaluate_task` requires the expected identity separately from the candidate. A candidate supplies only a proof array or `{proof, scope_proofs}`. Extra fields, replacement rules/facts/oracle policies, changed clocks, and changed identities fail.

Host errors are errors, not candidate failures. Rejected candidates earn 0; accepted proofs earn 1. `require_coverage` requires checked scope certificates; `allow_oracles` is a host task-selection policy, never a correctness score. Nothing here awards a claim about unrestricted legal correctness or an unsupported answer of "unknown".

## Signed records

The `receipts` module implements Ed25519 journals against an explicit host trust store. Events are domain-separated, hash-linked and ordered. Keys have validity/revocation intervals. Reviews bind exact artifact digests, remain separated by aspect and reviewer, and can be withdrawn. Earlier snapshots do not approve later ones. Author self-review cannot be declared independent.

Evidence receipts bind a context digest and exact fact, oracle request, response or withdrawal object. Oracle response keys must match the declared provider principal. The authentication API requires an externally pinned journal head and exposes missing receipts.

Signatures authenticate key possession under supplied trust bindings. They do not authenticate the real-world identity without an external binding, establish expertise, prove evidence truth, calibrate a classifier or provide trusted timestamps. A hash chain alone cannot prove unpublished events do not exist.

## Shared API

CLI: `oathkeeper policy request.json`. Python: `oathkeeper.assess_policy(request_json)`. WASM: `assess_policy_json(request_json)`. Each uses the same strict Rust dispatcher. Request `operation` is one of `coverage`, `verify_scoped`, `freeze`, `evaluate`, `journal`, `reviews`, `authenticate`, `signing_payload`, `digest`. The corresponding fields are defined by `crates/oathkeeper-core/src/policy.rs` and reject unknown keys.

Canonical digests use SHA-256 over compact UTF-8 JSON with recursively sorted object keys. Array order is retained. Signing bytes use `oathkeeper.receipt/1\n` followed by that canonical event without its signature. Use the Rust API to create those bytes.
