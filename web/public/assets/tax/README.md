# U.S. federal IRA pilot · 2025 · module 0.1.0

The canonical authoring language is `.oath`. JSON files are generated interchange and proof artifacts. The module uses language and kernel 0.5, including explicit coverage contracts and checked quantity arithmetic. Older engines reject these new declarations.

`backdoor-roth-2025.oath` models the bounded federal income allocation for an ordinary traditional-IRA-to-Roth conversion completed in calendar 2025, using final 2025 IRS guidance. It checks regular contribution dollar and own-compensation limits, aggregates the owner's traditional/SEP/SIMPLE balances, subtracts following-year contributions from available conversion-year basis, allocates basis pro rata, and carries forward unused basis. It has no MAGI input: a direct Roth contribution income limit is not a conversion income limit. This does not establish the eligibility of any separately supplied direct Roth contribution.

Only 2025 is supported. A 2025-designated contribution made in early 2026 is distinct from a 2026 conversion, which needs the 2026 module. This is a historical 2025 scenario tool, not a claim that 2025 contributions can still be made today.

## Run

```sh
cargo run -- tax-backdoor-roth lawlib/us/tax/scenarios/synthetic-pro-rata.json
cargo run -- verify lawlib/us/tax/shank-2018.case.oath
python tools/audit_tax.py
```

```python
import json, oathkeeper
from pathlib import Path
scenario = Path('lawlib/us/tax/scenarios/synthetic-pro-rata.json').read_text()
result = json.loads(oathkeeper.assess_backdoor_roth(scenario))
# Independent certificate replay using the general kernel:
report = json.loads(oathkeeper.verify(json.dumps(result['certificate']), 'json'))
```

Money is integer USD cents. Blank/null is unknown, never zero. All input fields, a source-era-or-later `recorded_at` timestamp, and a positive 2025 conversion are required for calculation. `purpose` defaults to `new_matter_assessment`; fixtures explicitly use `synthetic_test`. Requests cannot supply rules or replace the embedded canonical module. Missing information produces `needs_information`; unsupported scenarios produce `outside_scope`, both without amounts or certificates. Negative, fractional, oversized, unknown-field, and inconsistent inputs are rejected. Each monetary input is capped at 1,000,000,000,000 cents; this is an engineering bound, not a legal limit.

`scope_confirmed` acknowledges **every** condition in `tax::SCOPE`, not merely that the user wants a calculation. In particular, other regular IRA contributions must be independently eligible Roth contributions; deductible traditional contributions and their worksheet interactions are excluded. The model excludes other distributions, withholding, RMD obligations, rollovers, repayments, QCD/HSA transfers, excess corrections, recharacterizations, inherited IRAs, SIMPLE conversions, spousal compensation, annuity adjustments, disaster relief, nonresident/treaty and community-property issues. All own traditional, traditional SEP and traditional SIMPLE year-end values are included; spouse IRAs and employer-plan balances are not.

## Arithmetic and assurance

The nontaxable estimate is `conversion × min(available_basis, aggregate_pool) / aggregate_pool`, rounded once to cents with ties up. Numerator and denominator are also returned. Form 8606 line 10 permits a decimal rounded to at least three places; this pilot intentionally preserves the exact fraction until its final cent rounding. Its output is an **allocation estimate**, not completed form line entries. Filing-rounding conventions can produce different amounts. The taxable amount is income allocation, not tax owed; rates, deductions, credits, penalties, future withdrawals, and state taxes are outside scope.

The dedicated Rust builder constructs arithmetic witnesses and rule applications, then the general kernel independently replays the certificate. It does not claim exhaustive search, optimal tax planning, or absence of opposing legal arguments. The browser calls the same Rust implementation through WASM. There are no classifier calls in ordinary arithmetic. A zero oracle count does not authenticate supplied basis, completeness of account aggregation, or the law translation. Missing/disputed basis should remain unresolved until evidence is established; no artificial confidence score converts uncertainty into fact.

IRS forms and publications are `administrative_guidance`, not statutes. Their model authority assessments are explicitly persuasive and machine-assisted/unreviewed. The module uses their specific 2025 instructions, cites the statutory context in those sources (§§ 219, 408, 408A), and does not pretend to encode those sections in full. Community legal-fidelity and formalization reviews remain pending and pinned to module hashes in `review-packet.json`.

## Real precedent: Shank

`shank-basis.oath` and two `.case.oath` files reconstruct **one actual matter**, *Shank v. Commissioner*, T.C. Memo. 2018-33, No. 1752-17 (March 20, 2018), using the official Tax Court PDF retrieved through DAWSON. The court found $4,760 of basis and $22,985 taxable from a $27,745 distribution in 2014 (slip pp. 5–7). Credibility, initial contributions, and no later contributions are explicit judicial findings. Arithmetic alone does not establish them.

The old returns were unavailable. The opinion does **not** establish that Shank never filed Form 8606. It allowed basis supported by credible testimony and records; it did not accept arbitrary unsupported basis. This was an IRA liquidation, not a Roth conversion. No current citator result or general backdoor Roth endorsement is claimed. The hypothetical scenarios in `scenarios/` are labeled engineering regressions, not additional real cases.

## Provenance and regeneration

`sources.lock.json` pins five official PDFs, original URLs, retrieval metadata, and SHA-256 digests. `oath.lock.json` pins canonical source graphs. `interpretations.json` records translation choices. `python tools/audit_tax.py --write` regenerates module IR, actual-case reports, synthetic regressions, and the pending review packet. The audit fails on stale exports or modified source bytes. Legal review is not inferred from passing engineering tests.
