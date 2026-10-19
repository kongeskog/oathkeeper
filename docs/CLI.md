# CLI reference

Run these commands from the repository root with a Rust toolchain installed.

## Analyze a real record

```sh
cargo run -- analyze lawlib/us/wa/seattle/cases/long-parking-evidence-review.case.oath
```

Searches supporting and opposing arguments, checks certificates, lists evidence gaps, and reruns selected inputs for sensitivity. Unresolved evidence is not a negative finding. This command also accepts a downloaded 0.4 context JSON file.

## Verify a supplied proof

```sh
cargo run -- verify lawlib/us/wa/seattle/cases/long-excessive-fines.case.oath
```

Success means derivation_verified under the supplied assumptions. Rejection returns a nonzero exit status. No scalar verifiability score is returned.

## Compile and export

```sh
cargo run -- compile lawlib/us/wa/seattle/parking.oath
cargo run -- normalize lawlib/us/wa/seattle/cases/long-excessive-fines.case.oath
```

Compile returns source metadata and typed IR. Normalize requires a valid supplied proof and exports its assembled bundle. For an unresolved case, use compile or the application's context export.

## Module and case JSON

```sh
cargo run -- verify-case module.json case.json
cargo run -- normalize-case module.json case.json
cargo run -- compare-arguments supporting.json opposing.json
```

Comparison requires two valid opposing proofs in the same frozen context. It does not resolve legal priority.

## Audit, review, and evaluate

```sh
python tools/audit_lawlib.py
python tools/review_packet.py --output /tmp/review-packet.json
python tools/benchmark.py --output /tmp/baseline.json
python tools/compare_snapshots.py old-module.json new-module.json --report old-report.json
```

The audit verifies pinned bytes and corpus consistency. Review generation does not perform human review. Change impact marks affected dependencies for replay and new argument search; it does not decide whether a difference represents an amendment or a corrected interpretation.

## Bounded federal tax assessment

```sh
cargo run -- tax-backdoor-roth lawlib/us/tax/scenarios/synthetic-pro-rata.json
cargo run -- verify lawlib/us/tax/shank-2018.case.oath
python tools/audit_tax.py
```

The first command returns `calculated`, `needs_information`, or `outside_scope`. A calculated result contains a replayable proof certificate and an allocation estimate, not tax owed. See the [tax module guide](lawlib/us/tax/README.md).
