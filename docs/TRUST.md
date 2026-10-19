# What verification establishes

Computational verification means the recorded answer follows from the
caller-supplied compiled program and scenario, under the recorded semantics and
limits. Source-byte integrity and legal review are separate assurances.

`oath-verify` depends only on `oath-core`. It does not depend on or call
`oathkeeper`, its interpreter, package loader or source fetcher. The interpreter
uses graph reachability to select rules; the checker independently reconstructs
the complete rule inventory and uses topological closure. Core shares values,
primitive arithmetic, observation operations, pattern binding, wire encoding and
structural IR checks, but no rule-selection procedure.

The checker validates every consumed node against the expected expression and
environment, checks branches and arithmetic, checks all candidates and priority
conditions, reconstructs the outcome and provenance, and rejects missing, extra,
reused, cyclic or out-of-bounds trace references. The program, package, request,
limits and result are bound by domain-separated digests.

The trusted computing base includes the source-to-IR compiler, structural IR
validator, checker, exact-number implementation, serializers, hash implementation,
Rust/toolchain/runtime and platform. Compilation is not formally proved.
Structural validation rejects malformed references but is not a replacement for
the compiler's full type and dependency analysis. Public APIs expect a program
produced by compile; deserialized arbitrary IR is not a trusted program.

Calendar, monetary and legal functions in `.oath` are visible computations,
checked through the same derivations. Their algorithms and interpretations still
need tests and review. There is no hidden trusted tax or calendar oracle.

Evidence names, purported judges, completeness assertions and selected readings
are premises. They are not authenticated by their labels. Used-premise reporting
is conservative: reading a containing record can include evidence attached to its
other fields. It is not a proof of a minimal premise set.

SourceRef hashes bind the original document bytes, not legal authority or faithful
translation. HTML changes can invalidate an audit without changing underlying
law. A URL and hash do not recover a lost historical version. ReviewStatus records
a review assertion; v0.1 does not implement signatures, reviewer identity
authentication or a credential system.

Coverage records expose investigated sources, exclusions, uninvestigated areas
and unresolved interpretations. Successful checks do not demonstrate that no
other source or rule is relevant.
