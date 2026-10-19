# Packages, requests, results and CLI

## Package snapshot

`oath.toml` declares package identity, exact language version, entry under
`src/`, and local dependencies with exact versions. `oath lock PACKAGE`
explicitly creates/updates `oath.lock`. All other commands require the lock to
match the resolved snapshot and never rewrite it. Dependencies are local paths;
there is no registry, network package resolution or implicit latest selection.

Digests include normalized manifest information, all authored `src/**/*.oath`
paths/bytes and dependency digests. Dependency paths are resolution hints and are
excluded from identity. CRLF is normalized to LF. Symlinked source entries are
rejected. Source references are ordinary constants inside hashed source files.
Scenario fixtures, original documents, caches, build products and absolute
checkout locations are excluded from program/package identity.

The compiled program digest additionally binds typed IR, source spans,
included-module inventory and semantics version. Changing source formatting can
change a digest even if a legal answer is unchanged.

## JSON protocol

Generated schemas are in [schemas/v0.1](../schemas/v0.1). Regenerate with:

```sh
cargo run -p oathkeeper --example export_schemas
```

Schemas describe structure; the typed reader additionally checks canonical
numbers, nominal types, arity, limits, duplicate keys, reference integrity and
semantics versions. The manifest schema describes the TOML data model in JSON.

A request identifies exactly one root query and exactly its named arguments.
Unknown arguments must be present as Missing observations:

```json
{
  "version": "0.1.0",
  "query": "demo.noise_status",
  "arguments": {
    "emergency": {
      "state": "known",
      "value": {"type": "bool", "value": true},
      "evidence": ["finding:emergency.1"]
    }
  },
  "metadata": {}
}
```

Metadata is a string-to-string extension map and does not participate in legal
conditions. Findings, interpretation profiles, cutoff dates and completeness
assertions used by a program belong in typed arguments. They are bound by the
request digest.

Int uses `{"type":"int","value":"7000"}`; Rational uses
`{"type":"rational","numerator":"7","denominator":"100"}`.
Records use `name`, `type_args`, and observation-valued `fields`; variants
also identify their `variant` and ordered observation-valued payload fields.
Generic type arguments use the schema's `Ty` representation.

All object keys are unique, unknown semantic fields are rejected, and integer
strings have no leading zeros, plus sign, exponent or negative zero. JSON numeric
tokens are only for safe-integer protocol counters; exact application numbers
are strings. Canonical encoding uses RFC 8785 UTF-16 key order and its integer-only
subset, with SHA-256 domain prefixes `oathkeeper:DOMAIN:v1\0`. Application
values are normalized before result comparison and certificate emission.

Each result has one outcome, rule support, used premise IDs, metadata references,
and the included closure's declared coverage references. These coverage records
are declarations, not a claim that every entry applies to the root query.
The certificate's postorder trace is the explanation graph; it is not duplicated
in a second graph field. Nodes identify expression spans, input environments,
calculations, branches, dependencies and complete rule/priority inventories.

## Commands

```text
oath run PACKAGE --request FILE --format text|json --certificate FILE [--limits FILE]
oath test PACKAGE [--json]
oath verify PACKAGE --request FILE --certificate FILE [--limits FILE] [--format text|json]
oath lock PACKAGE
oath check PACKAGE
oath coverage PACKAGE [--json]
oath sources check PACKAGE [--fetch] [--cache DIRECTORY]
```

The certificate option is optional for run; the Rust API always returns one.
JSON run output is the QueryResult; the separate certificate file contains
the result and derivation. Verify receives the caller's locked package and
request and cannot let a certificate select either.

Fixture JSON files under `tests/` contain `name`, `request`, `expected` and
optional `limits`. Expectations choose either an operational error or an
outcome with optional value/stage/issue/support assertions. The harness calls the
same run API and then independently verifies every successful derivation,
including unresolved and conflicted outcomes.

Source auditing reads optional cache entries named by raw-byte SHA-256 or
`fixture:relative/path` references. Only `--fetch` permits HTTPS retrieval.
Matched, Mismatched and Unavailable are audit results; NotChecked is reserved for
records not submitted to auditing. Run and computational verify never read these
documents. A missing archive can therefore prevent an audit without changing an
answer or certificate.

## Resource profile

Defaults: 1,000,000 evaluated expressions/query entries, 250,000 trace nodes,
100,000 collection entries, call depth 128, 65,536 bits per exact component,
64 MiB per input/certificate and JSON nesting 128.

An execution-limit file contains the complete ExecutionLimits object. Verification
requires the same profile as the certificate. A successful trace contains one
node per evaluation step. The checker separately enforces its own work bound.
Overflowing any bound is operational failure, never a legal answer.

v0.1 permits configuring work/size budgets but fixes safety ceilings of 128 for
call depth and nesting. Depth counts expression evaluation as well as function
and query calls. Compiler expression/type nesting is also bounded. Workers
reserve a 32 MiB stack so host thread defaults do not change these guarantees.
Limits bound logical work and stored structures; they are not a hard process
RSS cap or a wall-clock timeout. Deploy hostile workloads with process isolation.
