## Get the source

These snapshots pair **Oathkeeper v0.1.0-dev.1** with **lawlib v0.1.0-dev.4**. Download both and extract them into the same directory. Oathkeeper and lawlib remain separate repositories; lawlib's local dependency paths expect sibling checkouts.

- [Download Oathkeeper source](/downloads/oathkeeper-v0.1.0-dev.1.zip)
- [Download lawlib source](/downloads/lawlib-v0.1.0-dev.4.zip)

```text
workspace/
  oathkeeper/
    Cargo.toml
    examples/quiet-hours/
    stdlib/
  lawlib/
    legal/
    private/household/demo/
    us/federal/constitution/
    us/federal/tax/ira/2025/
```

## Build the CLI

Install [Rust with rustup](https://rustup.rs/) and a native linker. The repository pins **Rust 1.98.1**, edition 2024. On Windows, use the Visual Studio C++ build tools with the MSVC toolchain, or a configured GNU toolchain. On Linux, the optional source-fetching client also needs OpenSSL development headers and pkg-config.

Run these commands from the extracted `oathkeeper` directory:

```sh
cargo build --workspace --locked
cargo test --workspace --locked
cargo run -p oath-cli -- check examples/quiet-hours
```

`cargo run -p oath-cli --` invokes the `oath` executable without installing it globally. Commands below use that form throughout.

## Read the policy

The tutorial asks one question: does this fictional policy permit a noise event? An ordinary rule prohibits it. An express emergency exception permits it. A priority declaration orders those two rules.

Open `examples/quiet-hours/src/main.oath`:

<!-- TUTORIAL -->

`fn` defines a closed computation. `query` declares a question. `rule` contributes a conditional answer. `priority` orders competing rules. The `meta` field connects a declaration to a typed constant; metadata does not activate a rule by itself.

This tutorial deliberately keeps time and membership out of its question. The [household lawlib package](/explorer/private/household/demo) adds source records, dated revisions, membership, findings, and separate consequences.

## Supply a scenario

The request names one query and provides an observation for each argument. This existing fixture supplies a known emergency finding:

<!-- REQUEST -->

Known `false` is different from a missing Boolean. A disputed fact retains its competing claims. Evidence identifiers label supplied support; their names do not authenticate the facts.

## Run and verify

Compute the scenario and write its certificate:

```sh
cargo run -p oath-cli -- run examples/quiet-hours --request examples/quiet-hours/scenarios/emergency.json --certificate emergency.cert.json
```

The expected outcome is **Resolved(Permitted)**. Both rules apply; the explicit priority selects the emergency rule.

Now independently check the certificate against the same package and request:

```sh
cargo run -p oath-cli -- verify examples/quiet-hours --request examples/quiet-hours/scenarios/emergency.json --certificate emergency.cert.json
```

`run` computes a derivation. `verify` checks an existing derivation. The separate `oath-verify` crate does not call the interpreter. A successful check establishes that the answer follows from this encoding and these premises; it does not establish legal correctness.

## Test a change

Run the package's authored scenario fixtures:

```sh
cargo run -p oath-cli -- test examples/quiet-hours
```

The harness checks the emergency and ordinary cases, and independently verifies their certificates. To try a missing fact, copy the request and replace the emergency observation with a `Missing` observation using the [JSON protocol](/docs/formats). Do not omit the required argument.

After editing an authored `.oath` file or manifest, explicitly update the package lock before running it again:

```sh
cargo run -p oath-cli -- lock examples/quiet-hours
cargo run -p oath-cli -- check examples/quiet-hours
cargo run -p oath-cli -- test examples/quiet-hours
```

Other commands validate locks; they do not silently rewrite them.

## Explore a legal calculation

With lawlib extracted next to Oathkeeper, run the constrained 2025 IRA scenario:

```sh
cargo run -p oath-cli -- run ../lawlib/us/federal/tax/ira/2025 --request ../lawlib/us/federal/tax/ira/2025/scenarios/pro-rata-conversion.json --format json --certificate ira.cert.json
cargo run -p oath-cli -- verify ../lawlib/us/federal/tax/ira/2025 --request ../lawlib/us/federal/tax/ira/2025/scenarios/pro-rata-conversion.json --certificate ira.cert.json
```

Its expected taxable conversion is **$6,510**: $7,000 of basis, a $7,000 conversion, and $93,000 of relevant remaining year-end balances under the package's supplied assumptions. The companion scenario with no remaining balance expects $0.

Read the [IRA package's scope, source records, and scenarios](/explorer/us/federal/tax/ira/2025). This answers a specific calculation question, with legal review marked Unreviewed. It does not decide overall transaction legality.

## Where to go next

- [Language reference](/docs/language): syntax, types, functions, rules, and priority.
- [Knowledge and outcomes](/docs/outcomes): unresolved inputs and conflicting answers.
- [Packages and CLI](/docs/formats): requests, locks, certificates, and resource limits.
- [Writing legal models](/docs/encoding): provenance, scope, and disciplined encoding.
- [Lawlib explorer](/explorer): inspect the actual source and authored fixtures.
