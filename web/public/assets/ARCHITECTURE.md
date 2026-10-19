# Architecture and trust boundaries

Oathkeeper 0.5 checks conditional legal arguments. It does not certify legal truth.

## Components

The `.oath` compiler parses readable declarations, resolves content-pinned imports, links sources, and emits typed JSON. The Rust kernel independently validates this JSON and each supplied proof step. Python, CLI, and WebAssembly call the same core.

Bounded finite-domain search lives outside the trusted derivation checker. It proposes supporting and opposing certificates and replays each through the kernel. Search exhaustion cannot create a negative fact. Reported completeness describes the implemented bounded exploration, not a mechanized completeness theorem or completeness of law.

The browser uses a worker to run the actual compiled Rust engine. It never submits evidence to an LLM. Saved fixture walkthroughs remain distinguishable from live analysis.

## Trust boundaries

The kernel trusts supplied legal rules, facts, interpretation assumptions, authority assessments, and oracle protocol agreements. Hashes bind exact bytes; they do not authenticate provenance or prove legal fidelity. A court finding supplied in a reconstruction remains an input.

Legal review and formalization review are separate records tied to an exact snapshot. The workflow validates recorded metadata and rejects stale approvals. Reviewer identity, qualifications, independence, and actual review require external verification. Both pilot reviews are pending.

The kernel checks signed claims, nominal types, units, substitutions, explicit premises, scope, forum treatment, selected times, evidence withdrawals, and frozen oracle responses. It reports dependencies and assurance dimensions. It never combines these into a correctness probability.

## Scope and time

A jurisdiction graph collects potentially applicable rules. Geographic depth does not resolve preemption, priority, or conflicts. Existing authority assessments are supplied policy inputs for an exact forum and authority instant, not a universal legal hierarchy.

Separate event, law, authority, and recording times preserve historical replay. The caller selects law and authority times and explains the relationship; retroactivity is not inferred. Query purpose distinguishes historical reconstruction, new-matter assessment, and synthetic tests.

## Evidence and counterarguments

Support and opposition can coexist as independently consistent arguments. The application exposes both without choosing a winner. Withholding an input reruns the search for alternative proofs; this is a sensitivity experiment, not a new factual finding.

Oracles supply frozen external responses under named procedures. Unanimity requires all agreed providers and matching polarity. Abstention or disagreement leaves the request unusable. Provider IDs and evidence hashes are checked for consistency; optional Ed25519 receipt journals authenticate exact responses and procedures against caller-supplied keys. Real-world identity, evidence truth, expertise and trusted timestamps remain outside that guarantee.

See [SPEC-0.5](SPEC-0.5.md) and [SOUNDNESS](SOUNDNESS.md) for the precise experimental contract.

## Independent lawlib releases

Canonical legal sources and community review events live in `kongeskog/oathkeeper-lawlib`. The software vendors a complete offline snapshot under `lawlib/`, pinned by `lawlib.lock.json` to an immutable Git commit and SHA-256 inventory. Updating that lock is explicit; runtime compilation never downloads moving source.

Coverage and review policies are separate from conditional derivation. `verify_scoped` checks the main certificate and all scope obligations. `evaluation::evaluate_task` accepts only proof material under a host-pinned task identity. The common policy dispatcher exposes the same functionality through CLI, Python and WASM.
