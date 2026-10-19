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
python tools/audit_lawlib.py
```

For the native Python API, install maturin in a virtual environment, run
`maturin develop`, then `python -m unittest discover -s python/tests`.

Legal modules require the review material described in [lawlib](lawlib/README.md).
Fictional examples must say they are fictional. Do not describe kernel acceptance
as proof that an encoding is legally authoritative or exhaustive. Keep oracle
assumptions, trusted facts, uncertainty, and competing interpretations explicit.

Contributions are provided under the repository's Apache 2.0 license; source
material must have compatible rights, documented separately from the code.

Review metadata is validated by `tools/review_packet.py`. Legal and formalization approval are separate, snapshot-specific records. Identify the reviewer, declare independence from authors, and record the actual review time and rationale; do not create approvals on another person's behalf. Keep approvals pending until review occurs. Run `tools/export_lawlib.py` after changing canonical sources; the audit rejects stale JSON exports and review packets.
