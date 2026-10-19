# Oathkeeper lawlib

Community-maintained, source-backed legal formalizations for [Oathkeeper](https://github.com/kongeskog/oathkeeper). Canonical law is authored in `.oath`; JSON files are generated interchange and review artifacts.

This repository owns the legal modules and their review history. The software repository vendors an immutable snapshot for offline use, pinned by commit and a complete SHA-256 inventory. Changes here do not silently change existing proofs or applications.

## Current collection

- **Seattle / Washington / U.S.:** selected historical parking provisions and Long/Timbs holdings. One actual Long matter, five historical proof paths and two incomplete evidence contexts.
- **2025 federal IRA:** bounded pro-rata allocation estimates, plus two historical proof paths from one actual Shank matter.

Both pilots remain drafts awaiting independent community review. Coverage declarations are explicit limits, not certificates of completeness. Read each module's README, interpretation notes and source lock.

## Contribute

Anyone may propose corrections, formalizations, tests or reviews. Professional credentials are not a contribution requirement. See [CONTRIBUTING](CONTRIBUTING.md), [GOVERNANCE](GOVERNANCE.md) and [the review protocol](community/README.md).

Install the Oathkeeper 0.5.0 Python binding from the software repository, then run `python tools/check.py`. CI pins the engine release. The package manifest is `oath-package.json`; changes to language or supported legal scope require explicit versioning and replay.

Original code, formalizations and annotations are Apache-2.0. Source documents retain their own rights, recorded in each `sources.lock.json`; this license does not relicense quoted or reproduced legal materials.
