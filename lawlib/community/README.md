# Review and evidence receipts

Anyone can review the corpus. Begin with a source correction or a written review in a pull request. You do not need to be a lawyer. Describe your basis and limits; an approval is a statement of your review, not a certification of legal truth.

`current.json` lists the exact artifact digest for each module. That digest binds the compiled source graph (including coverage), interpretation notes and source inventory. `reviews.json` starts with no events, and `trusted-keys.json` starts with no keys. No real reviewer or approval has been invented.

For a signed statement, create an Ed25519 key **outside the repository**. Register only the public key through a pull request. The key entry contains `id`, `principal`, the 32-byte lowercase-hex `public_key`, RFC3339 `valid_from`, and optional `valid_until` / `revoked_at` (null when absent). Key-to-person/account binding remains an explicit maintainer assertion.

A receipt has `id`, zero-based `sequence`, `previous_sha256` (null for the first event), `signer` (key ID), `recorded_at`, `payload` and lowercase-hex `signature`. A review payload is:

```json
{
  "kind": "review",
  "artifact_sha256": "<exact digest from current.json>",
  "aspect": "formalization",
  "decision": "changes_requested",
  "independence_declared": true,
  "rationale": "Explain the exact checks, findings and remaining concerns."
}
```

Aspects: `legal_fidelity`, `formalization`, `source_accuracy`, `coverage`. Decisions: `approved`, `changes_requested`, `withdrawn`. Authors must use `independence_declared: false` for their own work. Do not copy this illustrative payload as an actual review.

Use `tools/sign_receipt.py` from the software repository with an existing Ed25519 PEM key (optional Python dependency: `cryptography`). The helper obtains canonical signing bytes from Rust, reads the key locally and writes a new signed receipt. Append that receipt to the journal, run the corpus check, and submit a PR. The Rust `digest` policy operation supplies the previous event's SHA-256; the `journal` operation checks signatures and ordering.

The signing domain is `oathkeeper.receipt/1\n` followed by UTF-8 compact JSON of the event without `signature`, recursively sorted object keys. Use the Rust `signing_payload` API instead of reimplementing canonicalization.

Signatures prove control of a host-trusted key. They do not prove expertise, truth, calibrated confidence, or trusted time. Events use signer-declared times. Hash chains detect internal edits and missing interior events; pin the expected head outside the journal to detect suffix removal or replay. An empty journal means no signed reviews.

Evidence receipts use the same envelope with `kind: evidence`, `context_sha256`, `target_kind` (`fact`, `attestation`, `oracle_policy`, `retraction`), `target`, and `payload_sha256`. They bind the complete evidence object and frozen case context. An oracle response must be signed by a key whose trusted principal matches the declared provider. Authentication reports missing receipts explicitly and never upgrades an unsigned assertion to authenticated evidence.
