import { readFile, writeFile, readdir, mkdir } from "node:fs/promises";
import { resolve, relative, join } from "node:path";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";

const app = resolve(import.meta.dirname, "..");
const lawlib = resolve(app, "../lawlib");
const core = resolve(app, "../oathkeeper");
const read = async (p) => (await readFile(p, "utf8")).replace(/\r\n/g, "\n");
const json = async (p) => JSON.parse(await read(p));
const sha = (text) => createHash("sha256").update(text).digest("hex");
const git = (cwd, ...args) =>
  execFileSync("git", args, { cwd, encoding: "utf8" }).trim();
const walk = async (p) =>
  (
    await Promise.all(
      (await readdir(p, { withFileTypes: true }))
        .sort((a, b) => a.name.localeCompare(b.name, "en"))
        .map(async (e) =>
          e.isDirectory() ? walk(join(p, e.name)) : [join(p, e.name)],
        ),
    )
  ).flat();

// This is a browsing index, not a compiler or an authority resolver. Source text
// remains verbatim. Balanced extraction ignores braces inside strings.
function records(text, name) {
  const found = [];
  const pattern = new RegExp("\\b" + name + "\\s*\\{", "g");
  for (const m of text.matchAll(pattern)) {
    let depth = 1,
      quoted = false,
      escaped = false,
      i = m.index + m[0].length;
    const start = i;
    for (; i < text.length && depth; i++) {
      const c = text[i];
      if (quoted) {
        if (escaped) escaped = false;
        else if (c === "\\") escaped = true;
        else if (c === '"') quoted = false;
      } else if (c === '"') quoted = true;
      else if (c === "{") depth++;
      else if (c === "}") depth--;
    }
    found.push(text.slice(start, i - 1));
  }
  return found;
}
const stringField = (text, name) => {
  const m = text.match(
    new RegExp("\\b" + name + '\\s*:\\s*("(?:[^"\\\\]|\\\\.)*")'),
  );
  return m ? JSON.parse(m[1]) : "";
};
const listField = (text, name) => {
  const m = text.match(new RegExp("\\b" + name + "\\s*:\\s*(\\[[^\\]]*\\])"));
  return m ? JSON.parse(m[1]) : [];
};
const labels = {
  "us/federal/courts/wawd/answer/2026": [
    "W.D. Washington · answer service",
    "United States",
    "A bounded 2026 Rule 12 answer-service date with Washington holidays and explicit procedural findings.",
  ],
  "drafts/us/federal/commerce/9801": [
    "Draft · commerce definitions",
    "Draft",
    "An unimplemented source-linked question from an earlier release, flagged for reassessment by the source comparison.",
  ],
  legal: [
    "Shared legal models",
    "Shared",
    "Jurisdiction-neutral records for institutions, findings, legal effects, and change over time.",
  ],
  "private/household/demo": [
    "Household policy",
    "Private",
    "A fictional quiet-hours policy with emergency permission, delayed commencement, and amendments.",
  ],
  "us/authority": [
    "US authority",
    "United States",
    "Explicit, limited models of binding force, citation permission, and persuasive relevance.",
  ],
  "us/federal/constitution": [
    "US Constitution",
    "United States",
    "137 constitutional provision units, scoped institutional authority, conditional lawmaking requirements and selected precedent.",
  ],
  "us/federal/tax/ira/2025": [
    "IRA · tax year 2025",
    "United States",
    "Constrained contribution and Roth conversion questions, exact basis allocation, and selected reporting.",
  ],
  "us/federal/procedure/frcp/6": [
    "Federal civil deadlines",
    "United States",
    "Conditional Rule 6(a) date computation with explicit holiday findings, clerk accessibility, and coverage limits.",
  ],
};
for (const root of [core, lawlib]) {
  if (git(root, "status", "--porcelain", "--untracked-files=no")) {
    throw new Error(
      "Content sync requires committed source repositories so the snapshot and downloads match their pinned revisions.",
    );
  }
}
const audit = await json(join(lawlib, "docs/source-audit.2026-10-19.json"));
const packages = [];
await mkdir(join(app, "src/content"), { recursive: true });
await mkdir(join(app, "public/downloads"), { recursive: true });
for (const path of await json(join(lawlib, "packages.json"))) {
  const root = join(lawlib, path),
    manifest = await read(join(root, "oath.toml"));
  const id = manifest.match(/name\s*=\s*"([^"]+)"/)[1];
  const modules = [];
  const sources = [],
    coverage = [],
    mappings = [];
  for (const file of (await walk(join(root, "src"))).filter((f) =>
    f.endsWith(".oath"),
  )) {
    const content = await read(file),
      filePath = relative(root, file).replaceAll("\\", "/");
    const name = content.match(/^module\s+([\w.]+);/m)?.[1] ?? filePath;
    const declarations = [
      ...content.matchAll(
        /^(type|enum|const|fn|query|rule|priority)\s+([\w]+)/gm,
      ),
    ].map((m) => ({
      kind: m[1],
      name: m[2],
      line: content.slice(0, m.index).split("\n").length,
    }));
    modules.push({
      path: filePath,
      name,
      content,
      sha256: sha(content),
      declarations,
    });
    for (const s of records(content, "SourceRef"))
      sources.push({
        id: stringField(s, "stable_id"),
        citation: stringField(s, "citation"),
        version: stringField(s, "edition_or_version"),
        locations: listField(s, "locations"),
        sha256: stringField(s, "expected_sha256"),
        publication: stringField(s, "publication_metadata"),
        retrieval: stringField(s, "retrieval_metadata"),
        file: filePath,
      });
    for (const c of records(content, "CoverageRecord"))
      coverage.push({
        id: stringField(c, "id"),
        questions: listField(c, "questions"),
        exclusions: listField(c, "exclusions"),
        notInvestigated: listField(c, "not_investigated"),
        unresolved: listField(c, "unresolved_interpretations"),
        sources: listField(c, "investigated_sources"),
        file: filePath,
      });
    for (const m of records(content, "SourceMapping"))
      mappings.push({
        id: stringField(m, "provision_id"),
        source: stringField(m, "source_id"),
        locator: stringField(m, "locator"),
        questions: listField(m, "executable_questions"),
      });
    const dest = join(app, "public/lawlib", path, filePath);
    await mkdir(resolve(dest, ".."), { recursive: true });
    await writeFile(dest, content);
  }
  const fixtures = [];
  for (const file of (await walk(join(root, "tests"))).filter((f) =>
    f.endsWith(".json"),
  )) {
    fixtures.push({
      path: relative(root, file).replaceAll("\\", "/"),
      ...(await json(file)),
    });
  }
  const [title, category, description] = labels[path];
  packages.push({
    id,
    path,
    title,
    category,
    description,
    version: "0.1.0",
    review: "Unreviewed",
    manifest,
    modules,
    sources,
    coverage,
    mappings,
    fixtures,
  });
}
const documents = {};
for (const [id, path] of Object.entries({
  language: "docs/LANGUAGE.md",
  formats: "docs/FORMATS.md",
  verification: "docs/TRUST.md",
  release: "docs/RELEASE.md",
}))
  documents[id] = await read(join(core, path));
documents.coverage = await read(join(lawlib, "docs/COVERAGE.md"));
documents["constitutional-authority"] = await read(
  join(lawlib, "us/federal/constitution/README.md"),
);
documents.encoding = await read(join(lawlib, "docs/ENCODING.md"));
documents["source-workflow"] = await read(
  join(lawlib, "docs/SOURCE-WORKFLOW.md"),
);
documents["wawd-answer"] = await read(
  join(lawlib, "us/federal/courts/wawd/answer/2026/README.md"),
);
documents.deadlines = await read(
  join(lawlib, "us/federal/procedure/frcp/6/README.md"),
);
documents["deadline-review"] = await read(
  join(lawlib, "us/federal/procedure/frcp/6/REVIEW.md"),
);
const inventory = await read(
  join(lawlib, "inventory/usc/119-121/manifest.json"),
);
await writeFile(join(app, "src/content/inventory.json"), inventory);
await mkdir(join(app, "public/inventory"), { recursive: true });
const constitution = await read(
  join(lawlib, "inventory/constitution/provisions.json"),
);
await writeFile(join(app, "src/content/constitution.json"), constitution);
await writeFile(join(app, "public/inventory/constitution.json"), constitution);
await writeFile(join(app, "public/inventory/usc-119-121.json"), inventory);
await writeFile(
  join(app, "public/inventory/README.md"),
  await read(join(lawlib, "inventory/README.md")),
);
for (const [repo, path] of [
  ["oathkeeper", core],
  ["lawlib", lawlib],
]) {
  execFileSync(
    "git",
    [
      "archive",
      "--format=zip",
      `--prefix=${repo}/`,
      "-o",
      join(
        app,
        "public/downloads",
        `${repo}-${repo === "lawlib" ? "v0.1.0-dev.4" : "v0.1.0-dev.1"}.zip`,
      ),
      "HEAD",
    ],
    { cwd: path },
  );
}
const snapshot = {
  version: "0.1.0-dev.4",
  coreRevision: git(core, "rev-parse", "HEAD"),
  lawlibRevision: git(lawlib, "rev-parse", "HEAD"),
  audit,
  packages,
  documents,
  tutorial: await read(join(core, "examples/quiet-hours/src/main.oath")),
  tutorialRequest: await json(
    join(core, "examples/quiet-hours/scenarios/emergency.json"),
  ),
};
await writeFile(
  join(app, "src/content/snapshot.json"),
  JSON.stringify(snapshot, null, 2) + "\n",
);
console.log(
  `Indexed ${packages.length} packages, ${packages.reduce((n, p) => n + p.modules.length, 0)} modules, ${packages.reduce((n, p) => n + p.fixtures.length, 0)} scenarios. Source archives refreshed.`,
);
