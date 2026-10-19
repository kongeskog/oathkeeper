# Seattle law library

This is the first **real-authority** module in Oathkeeper. It contains nine
parameterized rules derived from Seattle Ordinance 120573, *City of Seattle v.
Long*, and *Timbs v. Indiana*, plus five executable reconstructions of issues in
the actual Long record. It is a machine-assisted formalization awaiting separate
legal and formal review, not a complete or current Seattle law certification.

```sh
cargo run -- verify-case lawlib/us/wa/seattle/parking.module.json lawlib/us/wa/seattle/cases/long-excessive-fines.case.json
python tools/audit_lawlib.py
```

The shared [module](parking.module.json) is independent of the individual case
files. A case supplies evidence, its query, and its proof; it cannot redefine
the library's rules or authority assessments. Each rule includes parameters,
source digest, pinpoint, holding/text role, and an explicit scope premise.

## Sources and historical scope

| Authority | Encoded material | Source |
| --- | --- | --- |
| Seattle Ordinance 120573 § 1, SMC 11.72.440 | Ordinary 72-hour branch and the impoundment framework, with scope conditions | [City Clerk scan](https://clerk.seattle.gov/~archives/Ordinances/Ord_120573.pdf) |
| *City of Seattle v. Long*, 198 Wn.2d 136, 493 P.3d 94 (2021), No. 98824-2 | Selected majority holdings about fines, ability to pay, and homestead protections | [Court slip opinion](https://www.courts.wa.gov/opinions/pdf/988242.pdf), [opinion-text mirror](https://law.justia.com/cases/washington/supreme-court/2021/98824-2.html) |
| *Timbs v. Indiana*, 586 U.S. 146 (2019), No. 17-1091 | Incorporation of the Excessive Fines Clause | [Supreme Court PDF](https://www.supremecourt.gov/opinions/18pdf/17-1091_5536.pdf) |

The legal snapshot is **August 13, 2021, 00:00 UTC**, following Long's filing on
August 12 at 08:00 Pacific. Recording/ingestion timestamps are in October 2026.
This separation is intentional: a source downloaded today can support a
historical reconstruction, but the formalizer must not backdate its ingestion.

The ordinance signature page (scan PDF p. 4) shows passage October 22, approval
October 29, and filing October 30, 2001. Section 2 specifies thirty days after
approval; the modeled commencement is November 28, 2001, at Seattle local midnight.
The Timbs decision timestamp is normalized to midnight UTC on its stated decision
date; no exact release time is asserted. These are explicit modeling conventions.

The court-hosted Long PDF states that the slip text was revised for official
publication. Our pinned Long evidence is the original judicial text extracted
from the mirror's `noframes` element, with whitespace normalized. It is **not**
represented as the revised official reporter text or as the bytes of the court
PDF. The court PDF was consulted during research; direct executor downloading
was blocked. Editorial summaries and website markup are excluded from the
committed text. The separate concurrence remains labeled and is not encoded as
a majority holding. Reconciliation against the final official text is a review
task, explicitly outstanding.

[sources.lock.json](sources.lock.json) records exact snapshot hashes, retrieval
URLs, official URLs, transformations, and source status. The ordinance scan and
Timbs PDF are pinned original PDF bytes. `tools/audit_lawlib.py` verifies those
bytes and every source reference before running all listed proofs. A separate
replay tool reproduces downloads into a new directory and rejects changed bytes:

```sh
python tools/refetch_sources.py lawlib/us/wa/seattle/sources.lock.json /tmp/oathkeeper-source-replay
```

## What the proofs establish

| Case file | Checked result and limits |
| --- | --- |
| `long-statutory-framework.case.json` | An established SMC violation is subject to Chapter 11.30's impoundment framework. This does not prove satisfaction of every procedural or constitutional prerequisite. |
| `long-excessive-fines.case.json` | Reconstructs the majority's excessive-fines result using the actual recorded gross-disproportionality judgment, rather than inventing a numeric test. |
| `long-ability-to-pay.case.json` | Reconstructs Long's requirement to consider ability to pay in its excessiveness analysis. This is not attributed to Timbs as an independent nationwide holding. |
| `long-homestead.case.json` | Long's occupied-personal-property situation does not require an additional homestead declaration. It does not establish a universal exemption from impoundment. |
| `long-no-automatic-impoundment-bar.case.json` | Reconstructs the majority's distinction between homestead protection at debt enforcement and independent parking impoundment authority. It does not endorse an unconstitutional tow. |

The ordinary 72-hour rule is reusable but is not used to reconstruct Long's exact
parking duration from incomplete timing evidence. The historical proofs instead
use the violation reported as established in the case. The ordinary-branch rule
requires all of the following inputs:

1. An established ordinary-branch scope: no shorter posted limit, no contrary
   legal permission, no authorized-vehicle exclusion on municipal property, and
   no junk/abandoned-junk-vehicle exclusion under subsections C–D.
2. Parking on a city street or other municipal property.
3. An established duration in whole hours and a checked strict comparison to 72.

Exactly 72 hours does not satisfy the strict comparison. Whole-hour integer
duration is a restricted representation; fractional durations require a finer
time unit in a future rule version. The other statutory branches must be added
separately. A scope assertion is not inferred from silence or from a favorable
classification label.

## Court findings, oracle dependence, and future cases

Every historical fact has a source, pinpoint, explanation, and basis kind:
`record_fact`, `judicial_finding`, or `scope_assumption`. The current five case
files use reported record facts and judicial findings. They do not fabricate
classifier outputs, human agreements, or confidence values for a real court.

In particular, the excessive-fines proof assumes the majority's recorded
gross-disproportionality finding. Its oracle-count score is therefore 1.0, while
the report exposes that judicial finding and the unreviewed authority assessments.
That score says there are no additional oracle attestations in this reconstruction;
it does not mean a subjective legal judgment has become a mathematical fact.
This is a proof of a derivation from a recorded judgment, not an independent
automated decision of the merits.

For a new case, bind `matter` to its own stable ID, supply actual evidence,
establish each scope condition, and obtain a justified disproportionality judgment
through the agreed oracle protocol when necessary. No original Long finding
automatically transfers to that person. New legal times/forums require new
authority assessments. Reports distinguish binding/persuasive assessments and
machine-assisted/unreviewed versus reviewed inputs; the kernel does not certify
the assessments themselves.

## Coverage and expansion

[coverage.json](coverage.json) lists exactly what exists and what is missing.
The full Seattle inventory is incomplete, so no invented coverage percentage is
reported. The approach to citywide coverage is in
[SEATTLE_PLAN.md](../../../../docs/SEATTLE_PLAN.md). The current logic still
needs extension for many legal constructs; this module does not imply every
Seattle provision can already be expressed faithfully.

Code and formalizations use the repository license. Source artifacts retain their
own legal status; the code license does not relicense archival attachments or
third-party material. See the rights notes in the source lock.
