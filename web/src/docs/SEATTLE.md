# Seattle: parking and Long

The 0.5 module has 14 formalized rules, five historical proof reconstructions from one real matter, and two unresolved assessment contexts. It imports Washington and federal rules through content-pinned `.oath` sources.

## Real sources

The authorities are Seattle Ordinance 120573 (SMC 11.72.440), *City of Seattle v. Long*, and *Timbs v. Indiana*. `sources.lock.json` pins four artifacts, including the official original Long slip PDF and an archival mirror. Final reporter reconciliation remains pending.

## Explore uncertainty

```sh
cargo run -- analyze lawlib/us/wa/seattle/cases/long-parking-evidence-review.case.oath
cargo run -- analyze lawlib/us/wa/seattle/cases/long-oracle-reassessment.case.oath
```

The parking assessment deliberately omits the court's violation finding. The available record does not establish all explicit negative exception premises, so its ordinary-branch query remains unresolved. The oracle context awaits an agreed subjective response. No probabilities or expert approvals were invented.

The duration branch uses exact milliseconds. Exactly 72 hours fails the strict greater-than comparison; 72 hours plus one millisecond satisfies that premise alone. Missing exceptions are never inferred false.

## Review and evaluation

`review-packet.json` maps every rule to its source and lists scope, omissions, and review questions. `review.json` records independent legal and formalization review as pending. `interpretations.json`, `pilot.json`, `coverage.json`, and `baseline.json` expose current assumptions and measured engineering results. The baseline's synthetic boundary tasks are explicitly labeled and are not new real precedents.

The module is a historical research formalization. Read the [pilot plan](../../../../docs/SEATTLE_PLAN.md) before extending it or treating a derivation as a legal conclusion.
