# Initial developer validation — 2026-10-18

| Check | Completed result |
|---|---|
| Rust workspace tests, Windows GNU 1.98.1 | 46 passed |
| Rust workspace tests, WSL Ubuntu / Linux 1.98.1 | 46 passed |
| Clippy, workspace/all targets, warnings denied | Passed on Windows GNU |
| rustfmt check | Passed |
| Standalone standard-library package | Locked snapshot compiles |
| Oathkeeper CLI tutorial fixtures | 2 passed; certificates independently checked |
| lawlib fixtures, Windows | 65 passed; certificates independently checked |
| lawlib fixtures, Linux | 65 passed; certificates independently checked |
| Windows certificate checked on Linux | Passed |
| Independently emitted Windows/Linux IRA certificates | Byte-identical |
| Distinct original source snapshots audited | 13 matched expected SHA-256 |
| Constitutional source-map inventory | 7 Articles + 27 Amendments |
| Checker crate boundary | oath-verify directly depends only on oath-core |
| Wire schemas | 7 generated schemas; strict reader/roundtrip tests pass |
| Parser/certificate-reader mutation smoke | 512 deterministic mutations each per test run |
| Linux libFuzzer target builds and smoke runs | Both built; 2,000 inputs each |

The library property tests run 48 cases each for civil-date roundtrips, rational
arithmetic/normalization and canonical exact-value serialization. Authored
calendar cases include century/leap-day behavior, negative epoch days, invalid
dates and both month-overflow policies.

Adversarial checks cover changed inputs/results, forged arithmetic, omitted
candidates/unknown exceptions/priorities, invented premises, cyclic and malformed
traces, wrong program/request hashes, forged resource budgets and malformed IR.
Additional checks cover inactive intermediate priority rules, declaration order,
empty/no-support outcomes, distinct missing/absent/disputed facts, CLI exit codes,
source mismatch/unavailability, lock portability and dependency changes.

The local libFuzzer smoke binaries used stable Rust without coverage or sanitizer
instrumentation. They are bounded smoke runs, not coverage-guided campaigns.
The separate nightly workflow is configured for instrumented campaigns but has
not been run on a hosting service. Hosted Windows MSVC CI has also not been run.

Windows native tests used the installed GNU Rust 1.98.1 toolchain and an external
LLVM-MinGW dlltool in the workspace's scratch toolchain directory. Linux tests
used Rust 1.98.1 under WSL with an independent target directory.

No external source retrieval is part of run or verify. Source auditing used the
external cache plus the fictional household fixture; its initial results are
recorded in lawlib/docs/source-audit.initial.json. That audit is not legal review.
All lawlib encodings remain Unreviewed, and the computational checks establish
no claim of comprehensive US-law coverage.
