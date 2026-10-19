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
test("snapshot preserves the complete package and fixture inventory", () => {
  assert.equal(snapshot.packages.length, 5);
  assert.equal(
    snapshot.packages.reduce((n, p) => n + p.fixtures.length, 0),
    65,
  );
  assert.equal(
    snapshot.packages.reduce((n, p) => n + p.modules.length, 0),
    20,
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
  assert.equal(new Set(sources.map((s) => s.id)).size, 13);
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
