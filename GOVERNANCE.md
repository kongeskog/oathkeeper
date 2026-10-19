# Oathkeeper governance

The software repository owns the language, kernel, bindings, application and engineering tests. [oathkeeper-lawlib](https://github.com/kongeskog/oathkeeper-lawlib) owns canonical legal modules, sources, interpretation notes and community review history.

All contributors may participate. Professional credentials are not required. Changes to semantics need a specification/version decision and adversarial tests. Keep automated search, source ingestion, model inference and legal review outside the trusted derivation checker.

The `lawlib/` directory here is a generated, revision-pinned snapshot, checked by `tools/lawlib_package.py verify`. Propose canonical law changes upstream, then explicitly vendor a reviewed commit and replay affected certificates and analyses. Software release approval does not establish legal fidelity of the corpus.

Maintainers explain compatibility and trust-boundary decisions in pull requests. Original contributions are Apache-2.0. Corpus source documents retain their separately documented rights. No number of signatures, tests or reviewers is a probability of legal correctness.
