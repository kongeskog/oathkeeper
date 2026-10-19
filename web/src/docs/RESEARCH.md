# Research and design decisions for 0.4

Research date: October 18, 2026. These are engineering decisions informed by primary
sources, not independent legal approval of the library.

## Legal computation and review

[Catala's exception documentation](https://book.catala-lang.org/en/2-2-conditionals-exceptions.html)
demonstrates explicit exception priorities and identifies choosing those priorities
as legal interpretation. Oathkeeper therefore does not silently select a winner
using geographic depth or a general "more specific wins" heuristic. It exposes
arguments and unresolved conflicts while the pilot uses explicit qualifications.

[OpenFisca's documentation](https://openfisca.org/doc/) describes executable
tax/benefit models and human collaboration in translating legislation. This
supports a bounded pilot and comparison against existing rule-based methods.
The 0.4 benchmark runs deterministic search before proposing any RL experiment.

[LegalRuleML 1.0](https://docs.oasis-open.org/legalruleml/legalruleml-core-spec/v1.0/os/legalruleml-core-spec-v1.0-os.html)
models defeasibility, normative operators, authority, and temporal information.
Oathkeeper does not claim to implement that standard or a complete legal logic.
Permissions, obligations, burdens, and conflict-resolution policies need explicit
semantics and source-specific review before additional language constructs are
given those meanings.

[OECD, Cracking the Code](https://www.oecd.org/en/publications/cracking-the-code_3afe6ba5-en.html)
frames machine-consumable government rules as a wider institutional undertaking.
The library's review, maintenance, and provenance workflows are therefore part of
the implementation, rather than relying on parser correctness as certification.

## Primary Seattle sources

[Seattle Ordinance 120573](https://clerk.seattle.gov/~archives/Ordinances/Ord_120573.pdf),
section 1, scan page 3, lines 32–42, supplies the historical 72-hour branch and the
authorized/junk-vehicle exclusions. It requires longer than 72 hours, with posted
limit and other-law qualifications. Version 0.4 uses exact millisecond units and
explicit evidence for qualifications; it does not infer absence from silence.

[City of Seattle v. Long, original official slip opinion](https://www.courts.wa.gov/opinions/pdf/988242.pdf)
was retrieved directly and pinned in 0.4. The opinion reports the 2016 parking
episode, while the decision is dated August 12, 2021. These are now separate
clocks. Its opening notice says a revised reporter opinion replaces the slip
opinion. The [official reporter portal](https://www.lexisnexis.com/clients/wareports/)
was consulted but did not yield the revised text through the research fetch.
Final-text reconciliation remains an explicit review item; no claim of current
good-law status is made. The older mirrored slip text remains archived for replay.

The historical reconstructions preserve recorded judicial findings. The new
parking evidence-review case withholds the recorded violation conclusion and
uses only separately described premises from the real record. Missing exception
evidence leaves the query unresolved. No new verdict or oracle response is invented.

## Court-facing scope

[Washington ER 702](https://www.courts.wa.gov/court_rules/pdf/ER/GA_ER_07_02_00.pdf)
addresses qualified expert testimony and assistance to the factfinder. A declared
oracle agreement is not a substitute for applicable evidentiary or procedural
requirements. The current pilot targets auditable arguments and evidence gaps;
court deployment requires a separately reviewed procedural specification.

## Decisions and unresolved research

- Retire the scalar oracle-count score. Counts depend on question granularity and
  premise classification; they are not reliability estimates.
- Preserve missing, conflicting, and abstaining oracle responses. Agreement is a
  checked input protocol; identity, calibration, and evidence quality remain external.
- Track source editions, interpretation changes, evidence withdrawals, and review
  snapshots separately. A byte digest is not a legal-validity certificate.
- Keep all authentic benchmark records from Long in one related-matter group.
  No train/test generalization result is possible from this one litigation matter.
- Collect independent legal/formal review and authentic additional matters before
  corpus expansion. Practitioner efficiency and checklist comparisons remain unmeasured.
