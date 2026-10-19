# Contributing

Use small changes that keep the trusted kernel understandable. A semantic change
needs a specification update, positive and adversarial cases, and a language
version decision. Keep proof search, model execution, and source ingestion outside
the verifier. Do not add unchecked escape hatches or silently accept unknown fields.

Run from the repository root:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -- verify examples/hoa.oath
python tools/lawlib_package.py verify
python tools/audit_lawlib.py
python tools/audit_tax.py
python lawlib/tools/check.py
```

For the native Python API, install maturin in a virtual environment, run
`maturin develop`, then `python -m unittest discover -s python/tests`.

Canonical legal contributions belong in [oathkeeper-lawlib](https://github.com/kongeskog/oathkeeper-lawlib). Community reviews are open to everyone; professional credentials are not required. The local `lawlib/` directory is a generated snapshot.
Fictional examples must say they are fictional. Do not describe kernel acceptance
as proof that an encoding is legally authoritative or exhaustive. Keep oracle
assumptions, trusted facts, uncertainty, and competing interpretations explicit.

Contributions are provided under the repository's Apache 2.0 license; source
material must have compatible rights, documented separately from the code.

Review metadata is validated by `tools/review_packet.py`. Legal and formalization approval are separate, snapshot-specific records. Identify the reviewer, declare independence from authors, and record the actual review time and rationale; do not create approvals on another person's behalf. Keep approvals pending until review occurs. Run `tools/export_lawlib.py` after changing canonical sources; the audit rejects stale JSON exports and review packets.

To update the offline corpus: clone the canonical lawlib repository, review an exact commit, then run `python tools/lawlib_package.py vendor --from-checkout /path/to/oathkeeper-lawlib --revision <40-character-SHA>`. Regenerate application snapshots, replay proofs and analyses, and commit the changed lock and vendored bytes together.

Property tests include a separate propositional reference model. For fuzzing, install `cargo-fuzz` and nightly Rust, then run `cargo +nightly fuzz run parse_verify -- -max_total_time=60` from the repository root. A scheduled/manual CI smoke run exercises this harness; passing it is not a soundness proof.
