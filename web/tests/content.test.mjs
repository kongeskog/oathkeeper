import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { createHash } from "node:crypto";
const snapshot = JSON.parse(
  await readFile(
    new URL("../src/content/snapshot.json", import.meta.url),
    "utf8",
  ),
);

test("constitutional inventory, downloaded data, sources and query links agree", async () => {
  const raw = await readFile(
    new URL("../src/content/constitution.json", import.meta.url),
    "utf8",
  );
  assert.equal(
    raw,
    await readFile(
      new URL("../public/inventory/constitution.json", import.meta.url),
      "utf8",
    ),
  );
  const inventory = JSON.parse(raw);
  assert.equal(inventory.provisions.length, 137);
  assert.equal(inventory.relationships.length, 37);
  const pkg = snapshot.packages.find(
    (p) => p.path === "us/federal/constitution",
  );
  assert.equal(pkg.fixtures.length, 69);
  for (const pin of inventory.source_pins)
    assert.ok(
      pkg.sources.some((s) => s.id === pin.id && s.sha256 === pin.sha256),
    );
  for (const p of inventory.provisions) {
    assert.equal(
      createHash("sha256").update(p.text).digest("hex"),
      p.text_sha256,
    );
    for (const query of p.executable_queries) {
      assert.ok(
        pkg.modules.some((m) =>
          m.declarations.some(
            (d) => d.kind === "query" && `${m.name}.${d.name}` === query,
          ),
        ),
      );
      assert.ok(pkg.fixtures.some((f) => f.request.query === query));
    }
  }
});
test("inventory download matches the pinned manifest and preserves distinct coverage states", async () => {
  const content = await readFile(
    new URL("../src/content/inventory.json", import.meta.url),
    "utf8",
  );
  assert.equal(
    await readFile(
      new URL("../public/inventory/usc-119-121.json", import.meta.url),
      "utf8",
    ),
    content,
  );
  const inventory = JSON.parse(content);
  assert.equal(inventory.source.release, "119-121");
  assert.equal(inventory.documents.length, 58);
  assert.equal(inventory.counts.by_kind.section, 65725);
  assert.equal(
    inventory.documents.reduce((n, d) => n + (d.counts.section ?? 0), 0),
    65725,
  );
  for (const d of inventory.documents) {
    assert.match(d.sha256, /^[a-f0-9]{64}$/);
    assert.match(d.index.sha256, /^[a-f0-9]{64}$/);
    assert.equal(d.status.source_audit, "Matched");
    assert.equal(d.status.executable_coverage, "NotAssessed");
    assert.equal(d.status.semantic_tests, "NotAssessed");
    assert.equal(d.status.legal_review, "Unreviewed");
  }
  const deadlines = snapshot.packages.find(
    (p) => p.path === "us/federal/procedure/frcp/6",
  );
  assert.equal(deadlines.fixtures.length, 40);
  assert.equal(
    deadlines.fixtures.find((f) => f.name === "closure_and_holiday").expected
      .value.fields.day.value.value,
    "20522",
  );
});
test("snapshot preserves the complete package and fixture inventory", () => {
  assert.equal(snapshot.packages.length, 8);
  assert.equal(
    snapshot.packages.reduce((n, p) => n + p.fixtures.length, 0),
    191,
  );
  assert.equal(
    snapshot.packages.reduce((n, p) => n + p.modules.length, 0),
    30,
  );
  assert.equal(
    snapshot.packages.find((p) => p.path === "us/federal/constitution").mappings
      .length,
    34,
  );
  assert.match(snapshot.coreRevision, /^[a-f0-9]{40}$/);
  assert.match(snapshot.lawlibRevision, /^[a-f0-9]{40}$/);
});
test("all indexed source files match downloadable bytes and declaration locations", async () => {
  for (const p of snapshot.packages)
    for (const m of p.modules) {
      const raw = await readFile(
        new URL(`../public/lawlib/${p.path}/${m.path}`, import.meta.url),
        "utf8",
      );
      assert.equal(raw, m.content);
      assert.equal(createHash("sha256").update(raw).digest("hex"), m.sha256);
      for (const d of m.declarations)
        assert.ok(
          raw.split("\n")[d.line - 1].startsWith(`${d.kind} ${d.name}`),
        );
    }
});
test("source integrity is matched by source identity AND exact digest", () => {
  const sources = snapshot.packages.flatMap((p) => p.sources);
  assert.equal(new Set(sources.map((s) => s.id)).size, 17);
  for (const s of sources) {
    assert.ok(
      snapshot.audit.sources.some(
        (a) =>
          a.source === s.id && a.sha256 === s.sha256 && a.status === "matched",
      ),
    );
    assert.match(s.sha256, /^[a-f0-9]{64}$/);
  }
});
test("scenario requests refer to included query declarations and retain review boundaries", () => {
  const allQueries = new Set(
    snapshot.packages.flatMap((p) =>
      p.modules.flatMap((m) =>
        m.declarations
          .filter((d) => d.kind === "query")
          .map((d) => `${m.name}.${d.name}`),
      ),
    ),
  );
  for (const p of snapshot.packages) {
    assert.equal(p.review, "Unreviewed");
    for (const f of p.fixtures)
      assert.ok(allQueries.has(f.request.query), f.request.query);
  }
  const ira = snapshot.packages.find(
    (p) => p.path === "us/federal/tax/ira/2025",
  );
  assert.ok(
    ira.coverage
      .flatMap((c) => c.exclusions)
      .includes("Overall backdoor Roth legality"),
  );
});
