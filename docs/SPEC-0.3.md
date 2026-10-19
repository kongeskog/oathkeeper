# Oathkeeper 0.3: authoring language and typed proof IR

Status: experimental. `oathkeeper/0.3` extends the [0.2 calculus](SPEC-0.2.md).
The Rust compiler and kernel are version 0.3.0. Versions 0.1 and 0.2 remain
readable, including the original TOML `.oath` tutorial. New `.oath` sources begin
with `oathkeeper "0.3";`. JSON is the interchange representation.

## What a proof establishes

The judgment is: under the supplied law snapshot, case facts, oracle protocol,
authority assessments, forum, and time context, this explicit derivation reaches
this signed ground proposition. A legal `rule` is a sourced, trusted legal axiom;
it is not a theorem about the natural-language source. Predicate declarations
introduce vocabulary, not evidence. Successful compilation is not legal review
or proof acceptance.

The logic remains finite, explicit Horn-style rule application with explicit
negative claims. There is no negation from missing information, automatic
priority, closed-world completion, or implicit quantifier/proof search. The
checker does not assign burdens of proof. Definitions, reusable proved lemmas,
`unless`, `overrides`, tactics, arithmetic expressions, and general quantifiers
are deliberately unsupported. A future definition must be conservative and a
lemma must carry a checked derivation; neither may create an unchecked rule.

## A real authoring source

The canonical Seattle source is [parking.oath](https://github.com/kongeskog/oathkeeper-lawlib/blob/main/us/wa/seattle/parking.oath).
Its subsection C rule has the following inference body (the linked source also
contains the complete authority, date, and citation metadata):

```oath
rule seattle.parking.authorized-vehicle-exclusion(matter: Matter, vehicle: Vehicle) @ "1" {
  // Authority metadata omitted from this excerpt; required in a complete rule.
  requires scope.smc72.historical-text($matter);
  requires parking.vehicle($matter, $vehicle);
  requires parking.on_municipal_property($matter);
  requires parking.vehicle_authorized_on_municipal_property($matter, $vehicle);
  ensures parking.smc72_c_excluded($matter);
}
```

A second rule derives `not parking.violates_smc72($matter)` from that exclusion.
This concerns only that section. It does not grant permission under other laws.
The ordinary branch requires explicit negative evidence for each qualification
and exclusion. Missing negative evidence leaves that argument unsupported.

## Syntax

Files are UTF-8. `//` introduces a line comment. Quoted strings use JSON escapes.
Identifiers use ASCII letters, digits, `_`, `.`, `/`, and `-`; delimiters such as
`:`, `$`, and `@` have syntax roles. Statements end with `;`; declaration blocks
end with `}`. Metadata fields inside blocks also end with `;`.

Each source has exactly one header:

```oath
oathkeeper "0.3";
module lawlib.us.wa.seattle.parking.long @ "0.3.0";
```

A case uses `case <name> @ "<version>";` instead. Imports are explicit:
`import "../long.oath" sha256 "<64 lowercase hexadecimal characters>";`.
Copy an actual pinned import from the source; the angle-bracket text is a grammar
placeholder, not an accepted digest.

Declarations:

| Form | Meaning |
| --- | --- |
| `type Matter;` | A nominal domain type |
| `predicate parking.vehicle(Matter, Vehicle);` | An exact predicate signature |
| `constant long.98824-2: Matter;` | A named ground symbol with one declared sort |
| `source name { citation "..."; uri "..."; sha256 "..."; }` | A reusable source reference |
| `jurisdiction id { name "..."; parents [us]; }` | An applicability node |
| `authority id { ... }` | The existing authority fields |
| `rule id(matter: Matter) @ "1" { ... }` | A sourced legal axiom, with `requires` and one `ensures` |
| `assessment id { ... }` | The existing forum/time-specific authority assessment fields |
| `fact id: predicate(arguments) { ... }` | A ground fact with `recorded_at`, `kind`, `source`, `pinpoint`, and `explanation` |
| `policy id: predicate(arguments) { ... }` | An agreed oracle request; the declaration supplies `oracle`, `atom`, and arguments |
| `attestation id: predicate(arguments) { ... }` | A frozen response with the existing attestation fields |
| `query { ... }` | Jurisdiction, forum, `as_of`, `known_at`, `goal`, and `root` |
| `proof { step id: claim by reason; ... }` | Ordered explicit proof steps |

A claim is `predicate(term, ...)` or `not predicate(term, ...)`. In rule bodies,
`$matter` refers to a declared parameter. Ground terms are declared constant
names or signed 64-bit integer literals. Sorts are `integer`, `symbol`, or a
declared domain name. The corpus uses `Matter`, `Vehicle`, and `integer`.

Metadata uses the corresponding IR field names. `source name;` embeds the named
source; `evidence name;` embeds its digest. Facts derive their evidence digest
from the supplied provenance source. Unknown fields, repeated fields, and
conflicting declarations are errors. Enum metadata may be bare or quoted.
Lists use commas; nested objects use `{ field value; }`.

Rules require `jurisdiction`, `effective_from`, `recorded_at`, `source`, and
`authority { authority id; role text-or-holding; pinpoint "..."; scope claim; };`.
`effective_until null;` means no supplied end date, not certification of present
validity. The authority scope must be one of the explicit `requires` clauses.
Authority and assessment metadata follow the 0.2 specification exactly.

Proof reasons are:

```text
fact <fact-id>
oracle <attestation-id>
compare greater_than | less_than | equal
apply <rule-id> @ "<version>"(parameter = term, ...) using [earlier-step, ...]
```

For example, the actual Long reconstruction ends with:

```oath
step violation: constitutional.excessive_fines_violation(long.98824-2)
  by apply wa.long.gross-disproportionality @ "1"(matter = long.98824-2)
  using [federal-scope, fine, disproportionality-finding];
```

Cases may declare only constants, source references, facts, policies,
attestations, a query, and a proof. They import one root module, which may itself
import multiple modules. Cases cannot redefine types, predicates, rules,
jurisdictions, or authority assessments. A module cannot contain case evidence.

## Kernel-enforced types

The 0.3 JSON IR adds `types`, `predicates`, and `constants` to modules/bundles.
Cases may add constants. For example:

```json
{
  "types": ["Matter", "Vehicle"],
  "predicates": [{"id": "parking.vehicle", "arguments": [{"domain": "Matter"}, {"domain": "Vehicle"}]}],
  "constants": [{"id": "long.98824-2", "sort": {"domain": "Matter"}}]
}
```

A rule parameter's `sort` can also be `{"domain":"Matter"}`. Every predicate
must be declared, and every occurrence must have the exact arity and sorts.
Every symbolic constant must be declared once. Every rule variable must have a
declared sort consistent with all its occurrences. Ground substitutions must
match parameter sorts and exact premise claims, including the matter identity.
No subtype coercions or implicit conversions exist.

The kernel checks facts, oracle requests/responses, rules, query goals, proof
claims, and substitutions independently of the compiler. This includes unused
input declarations. Direct JSON cannot bypass typing. Version 0.1/0.2 bundles
cannot carry domain types or signature tables. `oathkeeper.integer.gt`, `.lt`,
and `.eq` have fixed `(integer, integer)` signatures and can only be established
by checked comparisons. The `oathkeeper.` predicate namespace is reserved.

## Imports, compilation artifacts, and source maps

`compile_file(path)` reads local relative imports, verifies each edge's SHA-256,
and flattens the declaration graph. It rejects cycles, different files claiming
the same package/version, and namespace collisions. A shared dependency is loaded
once; every incoming hash is still checked. An import is not a legal authority
or applicability edge. Importing a rule cannot bypass jurisdiction, forum, date,
scope, or treatment checks.

The `oathkeeper-compilation/1` artifact contains:

- `module` and optional `case`: the explicit typed IR.
- `lock`: every source's kind, package, version, and exact byte digest, including
  the entry source. Inline pins transitively commit to dependencies.
- `snapshot_sha256`: a digest of the module, case, and sorted lock, excluding
  filesystem locations. It differs from the verified bundle fingerprint.
- `source_map`: declaration, premise, and proof-step file/line/column locations.
- `compiler_version`: the generating compiler version.

Compilation checks syntax, name collisions, and signatures. The kernel checks
full legal eligibility and derivations during `verify`; a compiled module or
pending application is not a verified proof. Parse and proof-step diagnostics
include source locations. Artifacts contain no executable Python/Rust callbacks.
The frontend never fetches sources or invokes a model.

Imports are local filesystem reads; a host compiling untrusted submissions must
provide its own filesystem sandbox. Per-file size is bounded at 2 MiB, the import
graph at 128 files/64 levels, metadata nesting at 32, and lexer tokens at 250,000.
These frontend limits are not a general hosted-kernel resource policy.

The committed Seattle `oath.lock.json` pins its flattened module and transitive
source graph. `tools/audit_lawlib.py` checks it, the separate primary-source lock,
all reference digests, coverage, five historical proofs, and the expected missing
oracle response. Source maps identify authoring locations; source citations and
pinpoints identify legal passages. Neither proves that the translation is faithful.

## Unknown results, counterarguments, and oracles

`verify` remains a strict proof checker: unsupported inputs return errors, not a
negative legal conclusion. `E_NO_PROOF` means no derivation was supplied.
`E_MISSING_EVIDENCE` identifies a referenced fact or attestation that is absent.
Wrong premises, false comparisons, and malformed derivations are verification
errors. None establishes the opposite proposition.

`compare_arguments(left, right)` requires opposite goals under identical laws,
evidence, oracle inputs, and query context. It independently checks both
submitted proofs and reports `conflicting_arguments`. It neither searches for
counterarguments nor resolves them. Binding and persuasive labels are reported;
no unstated priority policy selects a winner. A single proof still cannot contain
contradictory claims or unused steps.

In 0.3 a frozen context contains at most one attestation per oracle request ID.
Conflicting or repeated responses are rejected before proof search can select a
favorable one. A host must resolve aggregation, agreement, abstention, and
provider authentication before freezing the context; the kernel does not invent
that procedure. Abstention supplies no proposition. An absent response cannot
satisfy an oracle step. Confidence is checked against the agreed threshold and
is never composed into a probability of legal correctness.

Reports retain `oracle-count-v1`: `1/(1+N)` for distinct used attestation IDs.
They additionally expose `oracle_requests_used`, `record_fact_count`, and
`judicial_finding_count` alongside existing scope assumptions and authority
review counts. Question bundling and reclassification can change the score;
it is not an invariant measure of subjective legal dependence. RL hosts should
reward a valid derivation under frozen context, with oracle count as a diagnostic.

## Time, review, and compatibility

Time semantics remain 0.2: rule effective intervals plus `recorded_at`, and query
`as_of` plus `known_at`. Authority assessments require the exact forum and legal
instant, with latest visible recording controlling. This is a pinned-snapshot
model, not append-only transaction-time revisions. Conduct time and later
adjudication/precedent availability are not yet separate first-class clocks.

The five Long proofs retain their original judgments and provenance. Their
ordinary parking branch was never used to reconstruct a precise duration from
incomplete evidence. The new explicit branch is version 2, with a new recording
time; subsection C adds two rules. The pending oracle reassessment removes the
court's gross-disproportionality finding as an input and deliberately supplies
no replacement attestation. It is a new assessment task over a real record, not
a fabricated court or classifier outcome.

Separate legal and formalization review states remain pending in `review.json`.
Compiler success and test success do not change them. Legacy JSON modules and
cases remain regression fixtures; the `.oath` files are the authoring sources
for version 0.3.
