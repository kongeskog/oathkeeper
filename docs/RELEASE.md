# 0.1.0 developer release

The initial implementation includes the four-crate workspace, functional
compiler/interpreter, independent checker, standard library, local packages and
locks, CLI/API, schemas, conformance/adversarial/property tests, fuzz targets and
separate lawlib integration.

The publishing targets are `kongeskog/oathkeeper` and
`kongeskog/oathkeeper-lawlib`. The website lives in Oathkeeper's `web/` directory.
Local validation and hosted workflow results are recorded separately.

## Pinning and CI

Oathkeeper builds and tests without lawlib. Its integration job uses
`integration/lawlib.json`, an exact commit from `kongeskog/oathkeeper-lawlib`.
The repository variable `LAWLIB_REPOSITORY` can override that location for forks.

Lawlib's `compatibility.json` pins an exact Oathkeeper candidate commit. A
subsequent Oathkeeper integration-pin commit changes no runtime code, avoiding a
circular moving-branch dependency. Lawlib CI defaults to `kongeskog/oathkeeper`,
with an optional `OATHKEEPER_REPOSITORY` override. Existing content locks pin the
std snapshot.

Each repository has an independent version/release history. Release snapshots
should remain immutable. For an amendment or encoding correction, update source
and rationale, keep historical revisions, add fixtures, explicitly relock affected
packages, and make a new release.

## Verification

The test suite includes independently authored expected outcomes, evaluator/checker
agreement, forged and omitted certificate premises, malformed traces, exact
arithmetic properties, calendar/rounding boundaries, portability and bounded
parser/reader mutation. Lawlib has separate scenario fixtures; each successful
run is independently checked. Source audits are separate from deterministic tests.

Native Windows GNU and WSL Linux are the local validation targets. The CI matrix
also describes Windows MSVC and hosted Ubuntu. Local GNU success does not claim
that hosted MSVC CI has already run. See the generated initial
`VALIDATION.md` for actual completed checks.

## Deliberate v0.1 limits

- Interpreted typed IR, no optimizer/native backend, recursion or dynamic dispatch.
- Single-root requests and conservative blocking on every unknown candidate guard
  or priority. No universal legal decision procedure.
- Local dependency resolution; external archives and authenticated review/signature
  infrastructure are not implemented.
- JSON schemas describe wire shape; reader/typechecker enforce semantic invariants.
- Source metadata validation recognizes the standard SourceRef/SourceAnchor schemas;
  arbitrary private legal ID graphs still require authored validation/review.
- Source-to-IR correctness is tested, not formally proved. The checker is independent
  implementation, not a theorem-prover kernel.
- Trace graphs are deliberately explicit and can be large. Resource errors can
  occur before nominal collection limits due to certificate-size constraints.
- Metadata attaches directly to queries/rules/priorities. Other declarations use
  ordinary provenance constants and explicit symbol references.

The expansion chapter of the accepted plan is an ongoing roadmap. General legal
impact analysis, full historical authority discovery and comprehensive legal
coverage are not delivered by these initial examples.
