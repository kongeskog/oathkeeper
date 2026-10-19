# Source, builds and evidence

The repository contains what contributors author. Reviewable changes should show a changed legal premise, citation, interpretation or test input, without thousands of generated lines or a PDF replacement.

```text
us/                       .oath rules and cases; authored metadata and scenarios
community/                signed review events and trusted public keys
tools/                    compiler, audit and evidence-cache commands
tests/                    storage and reproducibility regression tests
oath-package.json          package and exact engine version
.build/                   generated JSON, proofs and review packets (ignored)
```

## Generated artifacts

Install the Python binding for the exact `engine_version` in `oath-package.json`. Run `python tools/build.py`. Compilation uses local `.oath` inputs and never downloads evidence. The output mirrors the source paths under `.build/`; `build-manifest.json` records every authored input hash, every output hash, the engine version and the evidence digests. It explicitly reports that compilation did not inspect the evidence bytes.

`python tools/build.py --check` independently rebuilds and compares all output bytes, detecting stale or edited products. Paths, build timestamps and machine timings do not enter the artifacts. `python tools/build.py --archive /tmp/oathkeeper-lawlib.tar.gz` produces a reproducible release archive. CI attaches this directory to each successful validation run; release archives are published separately from Git source.

Authored JSON is still appropriate for manifests, source provenance, scenario inputs and signed review events. It is not a second authoring language for legal rules. CI rejects compiled exports, PDFs, archives and files larger than 1 MiB in the authored tree.

## Evidence objects

Each `sources.lock.json` entry pins a SHA-256 digest, exact byte count, media type, logical filename, original provenance and HTTPS mirrors serving those exact bytes. `file` is a display/export name, not a path to a tracked file. The `uri` remains the cited authority; a storage mirror never becomes legal authority.

The existing nine snapshots are archived as individual assets in the [evidence-2026-10 release](https://github.com/kongeskog/oathkeeper-lawlib/releases/tag/evidence-2026-10), named `<sha256>.pdf` or `<sha256>.txt`. Original PDFs and the previously extracted Long text retain their original bytes and recorded rights. The cache layout is:

```text
$OATHKEEPER_CACHE_DIR/sha256/<first-two-hex>/<full-sha256>
```

The default is `$XDG_CACHE_HOME/oathkeeper`, or `~/.cache/oathkeeper`. Different modules and checkouts reuse the same object. Fetching one module does not download the whole corpus:

```sh
python tools/sources.py list --module seattle
python tools/sources.py fetch --module seattle
python tools/sources.py fetch --source shank2018
python tools/sources.py verify              # cached bytes only; never network
python tools/check.py --fetch               # explicit full evidence audit
python tools/check.py                       # repeat the audit offline
```

Every cache read checks size and hash again. Downloads stream to temporary files and enter the cache only after verification; truncated, oversized, changed or unavailable evidence fails visibly. The default per-object download limit is 128 MiB, configurable with `--max-bytes`. A corrupt cache object is rejected; remove that object explicitly before refetching. Missing evidence does not invalidate a conditional derivation, but it prevents reporting a successful evidence audit.

For offline environments, copy the cache or use `sources.py import --source ID --file /path/to/snapshot.pdf`. Imports perform the same checks. To change storage providers, add exact-byte HTTPS mirrors (for example object storage or an institutional archive). No Git LFS service or specific cloud SDK is required. Hashes identify bytes, not authenticity or legal correctness. Keep more than one independently operated mirror when possible; an artifact host can still become unavailable.

## Releases and history

New documents are uploaded as new content-addressed assets; never replace an existing asset to represent an amendment. New editions receive new hashes and explicit provenance. Do not run `gh release upload --clobber` on evidence. Preserve original rights and do not publish private records.

Version 0.1.0 and its historical Git objects remain intact because released software pins that exact revision. Version 0.2.0 removes generated products and source documents from the current source tree and stops that growth. A shallow clone or a GitHub source archive downloads only the current authored snapshot. Rewriting historical tags would break existing revision pins and is not part of this migration.
