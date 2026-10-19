# Soundness scope

Oathkeeper verifies conditional derivability in a supplied finite rule environment.
It does not prove the truth of the premises, the fidelity of legal axioms, or legal
completeness. The Rust implementation has not been independently proved sound.

## Reference judgment

Let the context contain typed signed ground premises, accepted frozen oracle
assertions, eligible parameterized rules, and fixed time/forum selections. A proof
step is accepted only by one of these rules:

1. Evidence: its exact signed claim equals a visible, unwithdrawn input premise.
2. Oracle: its exact claim equals the assertion accepted by the declared protocol.
3. Comparison: exact same-kind, same-unit numeric comparison computes that polarity.
4. Application: all declared variables receive correctly typed ground terms;
   each instantiated premise equals an earlier checked step; the conclusion is
   exactly the instantiated rule conclusion; authority eligibility checks pass.

By induction on the ordered proof steps, each accepted step is derivable under
that context and these inference rules. The designated root must match the query.
Every supplied step must be in its dependency closure. The kernel rejects a proof
containing opposite signed claims, preventing inconsistent certificates; it does
not require the whole supplied context to be consistent.

This argument describes the intended calculus. It is not a machine-checked proof
that the Rust code implements it, nor a theorem about natural-language law.

## Search boundary

Search enumerates a finite active domain and candidate derivations. Every returned
candidate is checked again by the ordinary kernel. A search bug cannot turn an
invalid candidate into an accepted proof unless the checker also has a bug. The
search and checker share data types, substitution, and eligibility helpers, so
they are not independent implementations; common-mode errors remain possible.

The finite-domain completeness indicator records whether exploration reached a
budget. It is not a formally proved completeness theorem. Tests include explicit
opposition, alternative proofs, missing evidence, unit mismatches, time cutoffs,
withdrawals, oracle disputes, and truncated search. Further independent review,
property-based testing, fuzzing, and a mechanized reference calculus remain work.

## Claims deliberately outside the theorem

Source authenticity, faithful translation, completeness, factual reliability,
reviewer identity, oracle calibration, jurisdictional priority, legal burdens,
procedural admissibility, and court acceptance are not logical consequences of
kernel acceptance. Each needs a separately specified process and evidence.
