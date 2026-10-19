# Module review contract

Each new 0.5 module needs:

- A bounded legal question, jurisdiction, forum, event/law/authority/recording times, purpose, and explicit scope.
- Canonical typed `.oath` sources with content-pinned imports, primary source editions and hashes, pinpoint citations, and generated JSON interchange.
- A premise-by-premise interpretation mapping with known omissions, alternative readings, procedural limitations, and assumptions that may hide disputed judgments.
- Real case provenance. Synthetic engineering mutations must be labeled and kept separate from actual judicial findings and precedents.
- A coverage inventory with an honest denominator (unknown when no authoritative inventory exists).
- Separate legal and formalization review records bound to the exact compiled snapshot. Approval requires an identified independent reviewer, timestamp, and rationale; metadata does not authenticate identity or expertise.
- Reproducible positive, negative, missing-evidence, conflict, boundary, oracle, and temporal tests appropriate to the module.
- A deterministic evaluation baseline and unmeasured community evaluation outcomes recorded as unknown rather than zero.

Use the Seattle pilot manifests as a starting structure. The pilot itself has not completed either independent review. Recompile exports with `tools/export_lawlib.py`, regenerate the packet with `tools/review_packet.py`, and run the corpus audit after source changes. Never copy another module's approval to a new snapshot.

Professional credentials are not a contribution requirement. Reviews use exact artifact digests and the signed journal protocol in `community/README.md`; no approvals are inferred from CI.
