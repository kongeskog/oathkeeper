# Oathkeeper

Oathkeeper is an open language and Rust engine for **conditional, inspectable legal arguments**. A checked derivation proves what follows from the supplied formalization and evidence. It does not prove that the formalization is the law, that evidence is true, or that a court would accept the conclusion.

The 0.5 research prototype includes readable `.oath` code, typed JSON interchange, a deterministic proof checker, bounded argument search, Python bindings, CLI, and the same Rust engine compiled to WebAssembly. The application provides lawlib exploration, a live evidence workbench, source inspection, review packets, and code documentation.

**Live application:** <https://oathkeeper-sable.vercel.app/>

The application source lives in [`web/`](web/README.md). With the native binding, Rust WASM target and wasm-bindgen CLI installed, run `python tools/build_web.py --fetch-sources`, then `npm ci --prefix web` and `npm run dev --prefix web` (Node.js 22+). See the web README for setup. Generated app data, source documents and WASM live in ignored `web/.generated/`. Core and application code are versioned together in this repository.

## Start with real law

The bounded Seattle pilot contains 14 rules from SMC 11.72.440, *City of Seattle v. Long*, and *Timbs v. Indiana*. Five historical proof paths concern **one actual litigation matter**. Two additional contexts expose missing parking evidence and a pending subjective assessment. Independent legal and formalization reviews remain pending. This is not all Seattle law or a statement of current law.

```sh
cargo run -- verify lawlib/us/wa/seattle/cases/long-excessive-fines.case.oath
cargo run -- analyze lawlib/us/wa/seattle/cases/long-parking-evidence-review.case.oath
cargo run -- compile lawlib/us/wa/seattle/parking.oath
cargo test --workspace --locked
```

Read the [language specification](docs/SPEC-0.5.md), [trust boundaries](docs/ARCHITECTURE.md), [soundness discussion](docs/SOUNDNESS.md), and [research](docs/RESEARCH.md). The [pilot](docs/SEATTLE_PLAN.md) defines acceptance criteria and openly records unfinished work.

## What changed in 0.4

* Reports say `derivation_verified` and expose separate assurance dimensions. The normalized oracle-count score has been removed, including for legacy inputs.
* Argument search exposes support, opposition, both, or neither, with independently checked certificates and explicit search limits. Missing evidence never proves a negative.
* Exact integer quantities carry units. The Seattle threshold uses milliseconds and a strict greater-than comparison.
* Event time, governing-law time, authority time, and recording cutoff are separate. Evidence withdrawals are replayable events.
* Oracle agreements distinguish observation, classification, legal assessment, and institutional determination; single-provider and unanimous protocols handle abstention, disagreement, and missing responses. Confidence is optional and requires a declared meaning when used.
* Review records bind to an exact module snapshot, separate legal from formalization review, and reject stale approvals. Source mappings expose assumptions and omissions.

## Python

```sh
python -m venv .venv
. .venv/bin/activate
pip install 'maturin>=1.7,<2'
maturin develop --locked
python -m unittest discover -s python/tests
python tools/audit_lawlib.py
python tools/review_packet.py --output /tmp/review-packet.json
python tools/benchmark.py --output /tmp/baseline.json
```

```python
import json
import oathkeeper
report = json.loads(oathkeeper.verify_file(
    'lawlib/us/wa/seattle/cases/long-excessive-fines.case.oath'))
analysis = json.loads(oathkeeper.analyze_file(
    'lawlib/us/wa/seattle/cases/long-parking-evidence-review.case.oath'))
assert report['status'] == 'derivation_verified'
assert analysis['status'] == 'neither'  # Missing explicit exception evidence.
```

## Library and training

Jurisdictions form an applicability graph. Federal, state, municipal, and private-covenant authority do not have an automatic geographic override order. Forum-specific treatment and scope remain explicit. International expansion needs legal-system-specific authority policies, source conventions, and community review of each formalization.

[Training guidance](docs/TRAINING.md) separates formalization, evidence assessment, oracle judgment, and proof construction. A frozen-context proof task earns a binary verification reward without a penalty for honestly declared oracles. The deterministic baseline has 11 engineering tasks, one real matter, and synthetic boundary tests; it is not a legal-accuracy benchmark or an RL training result.

Code is Apache-2.0. Source artifacts retain their documented provenance and rights. Contributions must preserve uncertainty and must not invent reviewers, evidence, or legal coverage.

### Bounded federal tax pilot

The [2025 backdoor Roth module](lawlib/us/tax/README.md) contains canonical `.oath` rules, pinned final IRS guidance, exact checked integer arithmetic, and an official-source reconstruction of *Shank v. Commissioner*. Run `cargo run -- tax-backdoor-roth lawlib/us/tax/scenarios/synthetic-pro-rata.json`, or Python `oathkeeper.assess_backdoor_roth(input_json)`. Missing basis remains unknown. The output is a conditional allocation estimate with a replayable certificate, not a tax return or a tax-owed calculation. Independent tax and formalization reviews are pending. Kernel 0.5.0 exposes the same implementation to Python and WASM.

## Community lawlib

Canonical legal modules live in [kongeskog/oathkeeper-lawlib](https://github.com/kongeskog/oathkeeper-lawlib). This repository contains a revision-pinned authored snapshot under `lawlib/`. Compiled JSON, proofs and review packets are generated under `.build/lawlib/`; evidence documents are fetched by SHA-256 into a shared cache outside Git. Compilation remains offline; an evidence audit requires the pinned bytes. Run `python tools/lawlib_package.py verify` to check every authored file against the lock, `python tools/build_lawlib.py` to generate artifacts, and `python lawlib/tools/sources.py fetch --module seattle` to retrieve only the selected evidence. See [source and storage design](lawlib/docs/STORAGE.md).

Version 0.5 adds explicit coverage and scope proofs, exception prerequisites, alternative interpretations, Ed25519 review/evidence receipts and frozen evaluation tasks. See [the specification](docs/SPEC-0.5.md). Community contributions and reviews are open to everyone; both legal pilots remain unreviewed until actual statements are submitted.
