import { readFile, writeFile, mkdir, copyFile } from "node:fs/promises";
import { join } from "node:path";
import { createHash } from "node:crypto";
import { gunzipSync } from "node:zlib";
import { app, lawlib } from "./source-roots.mjs";
const roots = process.argv.slice(2).filter((arg) => arg !== "--assets-only");
if (roots.length !== 2)
  throw Error(
    "Usage: pnpm provisions:sync PREVIOUS_EXTRACTION CURRENT_EXTRACTION",
  );
const sha = (b) => createHash("sha256").update(b).digest("hex");
for (const [i, release] of ["119-118", "119-121"].entries()) {
  const pin = await readFile(
    join(app, `src/content/provisions-${release}.json`),
  );
  if (
    JSON.stringify(
      JSON.parse(await readFile(join(roots[i], "catalog.json"), "utf8")),
    ) !== JSON.stringify(JSON.parse(pin))
  )
    throw Error("Extraction does not match pinned catalog");
  const catalog = JSON.parse(pin),
    out = join(app, "public/provisions", release);
  await mkdir(out, { recursive: true });
  async function copy(ref) {
    if (!/^[A-Za-z0-9.-]+\.json\.gz$/.test(ref.path))
      throw Error("Invalid shard path");
    const b = await readFile(join(roots[i], ref.path));
    if (b.length !== ref.bytes || sha(b) !== ref.sha256)
      throw Error("Shard integrity failure: " + ref.path);
    await copyFile(join(roots[i], ref.path), join(out, ref.path));
    return b;
  }
  for (const d of catalog.documents) {
    const index = JSON.parse(
      gunzipSync(await copy(d.index), { maxOutputLength: 128 * 1024 ** 2 }),
    );
    if (index.source.sha256 !== d.sha256)
      throw Error("Index document mismatch");
    for (const c of index.chunks) await copy(c);
  }
  await writeFile(join(out, "catalog.json"), pin);
  console.log(
    `Prepared and hash-checked ${release}: ${catalog.count} source units`,
  );
}
// Small, versioned authoring inputs are tracked; large derived text stays external.
if (process.argv.includes("--assets-only")) process.exit(0);
await copyFile(
  join(lawlib, "tools/inventory/draft.mjs"),
  join(app, "src/lib/draft.mjs"),
);
await copyFile(
  join(lawlib, "inventory/encodings.json"),
  join(app, "src/content/encodings.json"),
);
const report = JSON.parse(
  await readFile(
    join(lawlib, "inventory/impact/119-118--119-121.json"),
    "utf8",
  ),
);
await writeFile(
  join(app, "public/inventory/impact-119-118--119-121.json"),
  JSON.stringify(report, null, 2) + "\n",
);
const compact = {
  ...report,
  changes: report.changes.map((c) => ({
    ...c,
    before: c.before.map((u) => ({
      key: u.key,
      number: u.number,
      heading: u.heading,
    })),
    after: c.after.map((u) => ({
      key: u.key,
      number: u.number,
      heading: u.heading,
    })),
  })),
};
await writeFile(
  join(app, "src/content/impact.json"),
  JSON.stringify(compact, null, 2) + "\n",
);
