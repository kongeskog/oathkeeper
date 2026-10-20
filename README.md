# Oathkeeper

Oathkeeper is a statically typed functional language for executable rules with
explicit uncertainty, provenance, and independently checked derivations. Its Rust
core is separate from [lawlib](https://github.com/kongeskog/oathkeeper-lawlib), the repository of legal models
and examples. This is the initial **0.1.0 developer implementation**.

An answer follows from a particular program and its supplied premises. A valid
certificate does not prove that the encoding is legally correct or that all
relevant law has been found.

## Build and try it

Rust **1.98.1**, edition 2024, and a native linker are required. On Linux, install
the OpenSSL development headers and pkg-config for the optional source-fetching
client. Computation itself uses no network.

```sh
cargo build --workspace --locked
cargo test --workspace --locked
cargo run -p oath-cli -- check stdlib
cargo run -p oath-cli -- run examples/quiet-hours --request examples/quiet-hours/scenarios/emergency.json --certificate emergency.cert.json
cargo run -p oath-cli -- verify examples/quiet-hours --request examples/quiet-hours/scenarios/emergency.json --certificate emergency.cert.json
cargo run -p oath-cli -- test examples/quiet-hours
```

On this Windows workspace, use `./scripts/cargo.ps1` in place of `cargo`.
It selects the installed GNU toolchain only when its reported version is 1.98.1.
Its local LLVM dlltool fallback lives outside this repository; other GNU
installations can set `OATH_DLLTOOL`. Normal MSVC installations use the pinned
toolchain directly. Pass an argument separator to the PowerShell wrapper as
`'--'`.

Clone the repositories as siblings, using `lawlib` as the local directory name:

```sh
git clone https://github.com/kongeskog/oathkeeper.git
git clone https://github.com/kongeskog/oathkeeper-lawlib.git lawlib
```

From the `oathkeeper` directory:

```sh
cargo run -p oath-cli -- run ../lawlib/us/federal/tax/ira/2025 --request ../lawlib/us/federal/tax/ira/2025/scenarios/pro-rata-conversion.json --format json --certificate ira.cert.json
cargo run -p oath-cli -- verify ../lawlib/us/federal/tax/ira/2025 --request ../lawlib/us/federal/tax/ira/2025/scenarios/pro-rata-conversion.json --certificate ira.cert.json
```

That scenario produces an exact USD taxable conversion amount of **6510** under
the package's constrained 2025 model. The companion scenario with no remaining
balance produces **0**. Neither query decides overall transaction legality.

## What is implemented

- Handwritten lexer, declaration parser, Pratt expressions, static typing,
  nominal records/variants, generics, immutable bindings and finite collections.
- Exact integers/rationals; calendar, rounding, money, sets and maps in `.oath`.
- Explicit query scope/applicability, complete imported rule inventories,
  conditional priorities, unresolved inputs and conflicting answers.
- Deterministic, bounded interpretation; source spans and a derivation graph.
- Independent certificate checking in `oath-verify`, without an interpreter
  dependency or shared rule-selection algorithm.
- Local package locks, versioned JSON schemas, scenario fixtures, coverage
  reports and optional source-byte audits.

The public operations are `run`, `test`, and `verify`. Authoring tools are
`lock`, `check`, `coverage`, and `sources check`. Legal outcomes, including
prohibitions, uncertainty and conflicts, exit successfully; operational failures
exit 1 and invalid CLI usage exits 2.

## Developer map

The landing page, documentation, and explorer live in [web/](web/README.md).
Run its pnpm commands from that directory. The Rust workspace builds independently
of the website, and lawlib remains a separate repository.

| Location | Responsibility |
|---|---|
| `crates/oath-core` | Values, IR, wire formats, arithmetic, structural validation |
| `crates/oath-verify` | Independent derivation checker |
| `crates/oathkeeper` | Compiler, runtime, package loader, audits, fixture harness |
| `crates/oath-cli` | `oath` executable |
| `stdlib` | Versioned standard library written in Oathkeeper |
| `schemas/v0.1` | Generated JSON Schema 2020-12 documents |
| `fuzz` | Parser and certificate-reader fuzz targets |

Read the [language specification](docs/LANGUAGE.md), [wire and CLI guide](docs/FORMATS.md),
[trust boundary](docs/TRUST.md), and [release/verification notes](docs/RELEASE.md).
The [accepted implementation plan](docs/IMPLEMENTATION_PLAN.md) preserves the
long-term design; release notes distinguish delivered mechanisms from future work.

There is no recursion, mutable state, language I/O, automatic legal research,
automatic holding extraction, theorem-prover kernel, or training pipeline in v0.1.
Unknown candidate conditions conservatively block a query even if a more
advanced analysis could prove them irrelevant.
