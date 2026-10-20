// Explicit authoring/build step. The deployed app never runs this tool.
import { readFile, writeFile, mkdir, stat, rename } from "node:fs/promises";
import { createReadStream, createWriteStream } from "node:fs";
import { Readable, Transform } from "node:stream";
import { pipeline } from "node:stream/promises";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { resolve, join } from "node:path";
const [extractor, cache, flag] = process.argv.slice(2);
if (!extractor || !cache || flag !== "--fetch")
  throw Error(
    "Usage: node scripts/fetch-provisions.mjs EXTRACTOR CACHE --fetch",
  );
const app = resolve(import.meta.dirname, ".."),
  roots = [];
await mkdir(cache, { recursive: true });
for (const release of ["119-118", "119-121"]) {
  const raw = await readFile(
      join(app, `src/content/provisions-${release}.json`),
      "utf8",
    ),
    catalog = JSON.parse(raw),
    pin = catalog.source;
  if (!/^https:\/\/uscode\.house\.gov\//.test(pin.url))
    throw Error("Unexpected archive host");
  const archive = join(cache, `${pin.sha256}.zip`);
  let exists = true;
  try {
    await stat(archive);
  } catch (e) {
    if (e.code !== "ENOENT") throw e;
    exists = false;
  }
  if (!exists) {
    console.log(`Fetching pinned official archive ${release}`);
    const response = await fetch(pin.url);
    if (!response.ok || !response.body) throw Error("Archive unavailable");
    const temp = archive + `.${process.pid}.part`,
      hash = createHash("sha256");
    let size = 0;
    await pipeline(
      Readable.fromWeb(response.body),
      new Transform({
        transform(b, _, done) {
          size += b.length;
          if (size > pin.bytes) return done(Error("Archive size limit"));
          hash.update(b);
          done(null, b);
        },
      }),
      createWriteStream(temp, { flags: "wx" }),
    );
    if (size !== pin.bytes || hash.digest("hex") !== pin.sha256)
      throw Error("Archive integrity mismatch");
    await rename(temp, archive);
  }
  const hash = createHash("sha256");
  let size = 0;
  for await (const b of createReadStream(archive)) {
    size += b.length;
    hash.update(b);
  }
  if (size !== pin.bytes || hash.digest("hex") !== pin.sha256)
    throw Error("Cached archive mismatch");
  const fingerprint = createHash("sha256")
    .update(JSON.stringify(catalog))
    .digest("hex")
    .slice(0, 12);
  const root = join(cache, `provisions-${release}-${fingerprint}`),
    pinPath = join(cache, `pin-${release}.json`);
  await writeFile(pinPath, JSON.stringify(pin));
  let ready = false;
  try {
    ready =
      JSON.stringify(
        JSON.parse(await readFile(join(root, "catalog.json"), "utf8")),
      ) === JSON.stringify(catalog);
  } catch (e) {
    if (e.code !== "ENOENT") throw e;
  }
  if (!ready)
    execFileSync(
      process.execPath,
      [resolve(extractor), pinPath, archive, root],
      { stdio: "inherit" },
    );
  roots.push(root);
}
execFileSync(
  process.execPath,
  [join(app, "scripts/sync-provisions.mjs"), ...roots, "--assets-only"],
  { stdio: "inherit" },
);
