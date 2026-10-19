# Federal source inventory

This inventory pins the OLRC all-titles U.S. Code XML release through Public
Law 119-121 (October 9, 2026), retrieved October 19, 2026: 58 XML documents,
53 numbered titles and five appendices. Title 53 is reserved. The original ZIP
is 108,675,587 bytes; SHA-256 is in [`source.json`](usc/119-121/source.json).

[`manifest.json`](usc/119-121/manifest.json) records each document's original
digest, publication identifier, publisher metadata, structure counts and derived
index digest. There are 65,725 `section` elements, 254,302 `note` elements and
1,311,559 cross-reference occurrences. These include repealed, omitted, reserved
and appendix material. They are **not counts of operative laws, complete legal
dependencies, or formalized provisions**.

The index preserves publisher identifiers, XML IDs, hierarchy, headings, source
credits, note attributes/group labels, and unresolved references. Repeated
locators remain separate occurrences. A source locator is not a stable legal
identity; XML IDs may change between releases. A `node` is a document/element
ordinal within this snapshot only.

Raw XML in the pinned ZIP is the lossless source. JSONL indexes support discovery;
they do not reproduce all body text or formatting. Note labels such as
`statutoryNotes` and `editorialNotes` remain publisher classifications, not
determinations of legal force. No amendment, commencement, definition scope,
binding authority or reference resolution is inferred.

## Reproduce offline

Use Node 24 and pnpm 11.25.0. The compiler and interpreter remain Rust; this
optional Node tool prepares the inventory and never executes legal programs.

```powershell
cd tools/inventory
pnpm install --frozen-lockfile
pnpm test
node import-usc.mjs ../../inventory/usc/119-121/source.json C:/path/to/cache
```

Place the ZIP in the cache under its exact SHA-256 filename, without an extension.
Add `--fetch` to explicitly download missing bytes. Size and digest are checked
before parsing. Changed bytes are rejected; no pin is updated automatically.
The index directory `usc-119-121-index-v1` is created exclusively. Use another
cache for replay. The completion manifest is written last; a failed/partial
directory is not a completed inventory.

The 58 JSONL files and originals stay outside Git. Git stores the pin and small
manifest. Rows have a `kind`, original attributes, `node`, nearest indexed
`parent`, number, heading and publisher note-group label. References also retain
displayed text and `href`. Rows follow deterministic closing-element order;
parents can appear later. No absolute cache path or clock enters the output.
Original XML and derived JSONL have separate hashes.

Parsing rejects DTDs, malformed XML, invalid UTF-8, wrong root namespaces,
unexpected ZIP entries, traversal, duplicate/case-colliding entries and resource
excess. Bounds: 512 MiB ZIP, 512 MiB/XML, 2 GiB expanded input, 256 levels,
5 million elements/XML and 1 MiB captured labels. This checks well-formedness,
not full USLM XSD validity. Saxes 6 is pinned and its upstream is archived;
replace only with compatible tested behavior. This is offline tooling for
pinned official files, not an upload service.

## Coverage and updates

Source audit, executable coverage, semantic tests and legal review are separate.
An import creates no executable `.oath` rules. Existing legal packages retain
their own source editions; this import does not automatically migrate them.

The U.S. Code is one federal collection, not a complete inventory of regulations,
treaties, cases, orders, state, tribal, territorial, county, municipal or private
law. See the [OLRC guide](https://uscode.house.gov/detailed_guide.html) for enacted
text, statutory notes and editorial material.

Add a new directory and explicit pin for a new release. Audit, rebuild, review
identifier/reference changes, and then assess affected encodings. Preserve old
pins. Text changes do not by themselves establish legal effective dates.
Cross-release identity reconciliation and legal impact analysis remain future
work. Session laws, regulations and court materials need separate ingestion and
coverage histories.
