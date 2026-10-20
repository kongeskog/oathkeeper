import snapshot from "@/content/snapshot.json";
export type Declaration = { kind: string; name: string; line: number };
export type OathModule = {
  path: string;
  name: string;
  content: string;
  sha256: string;
  declarations: Declaration[];
};
export type Source = {
  id: string;
  citation: string;
  version: string;
  locations: string[];
  sha256: string;
  publication: string;
  retrieval: string;
  file: string;
};
export type Coverage = {
  id: string;
  questions: string[];
  exclusions: string[];
  notInvestigated: string[];
  unresolved: string[];
  sources: string[];
  file: string;
};
export type Fixture = {
  name: string;
  path: string;
  request: { query: string; [key: string]: unknown };
  expected: {
    kind: string;
    status?: string;
    value?: unknown;
    [key: string]: unknown;
  };
};
export type LawPackage = {
  id: string;
  path: string;
  title: string;
  category: string;
  description: string;
  version: string;
  review: string;
  manifest: string;
  modules: OathModule[];
  sources: Source[];
  coverage: Coverage[];
  mappings: {
    id: string;
    source: string;
    locator: string;
    questions: string[];
  }[];
  fixtures: Fixture[];
};
export const packages = snapshot.packages as LawPackage[];
export const release = {
  version: snapshot.version,
  coreRevision: snapshot.coreRevision,
  lawlibRevision: snapshot.lawlibRevision,
  audit: snapshot.audit,
};
export const documents = snapshot.documents;
export const tutorial = snapshot.tutorial;
export const tutorialRequest = snapshot.tutorialRequest;
export const counts = {
  packages: packages.length,
  modules: packages.reduce((n, p) => n + p.modules.length, 0),
  queries: packages.reduce(
    (n, p) =>
      n +
      p.modules.reduce(
        (m, f) => m + f.declarations.filter((d) => d.kind === "query").length,
        0,
      ),
    0,
  ),
  fixtures: packages.reduce((n, p) => n + p.fixtures.length, 0),
  sources: new Set(packages.flatMap((p) => p.sources.map((s) => s.id))).size,
};
export const packageHref = (p: Pick<LawPackage, "path">) =>
  `/explorer/${p.path}`;
export const slugify = (s: string) =>
  s
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-|-$/g, "");
export const docPages = [
  {
    slug: "constitutional-authority",
    title: "Constitutional authority",
    group: "Lawlib",
    description:
      "Provision inventory, institutional powers, lawmaking requirements and explicit interpretive gaps.",
  },
  {
    slug: "source-workflow",
    title: "Source to verified program",
    group: "Lawlib",
    description:
      "Pinned passages, encoding drafts, explicit source dependencies, and release impact.",
  },
  {
    slug: "wawd-answer",
    title: "W.D. Washington answer example",
    group: "Lawlib",
    description:
      "A concrete, bounded 2026 answer-service calculation and its review questions.",
  },
  {
    slug: "getting-started",
    title: "Getting started",
    group: "Introduction",
    description:
      "Build the Rust core, run your first policy, and verify the result.",
  },
  {
    slug: "language",
    title: "Language reference",
    group: "Language",
    description:
      "The complete declaration syntax, type system, and execution semantics.",
  },
  {
    slug: "outcomes",
    title: "Knowledge & outcomes",
    group: "Language",
    description:
      "Model missing facts, disputed claims, and competing answers explicitly.",
  },
  {
    slug: "time",
    title: "Dates & legal change",
    group: "Language",
    description:
      "Calendar arithmetic, delayed commencement, and explicit historical context.",
  },
  {
    slug: "formats",
    title: "Packages & CLI",
    group: "Tooling",
    description:
      "Locked packages, scenario JSON, command-line operations, and limits.",
  },
  {
    slug: "verification",
    title: "Verification & trust",
    group: "Tooling",
    description:
      "What a certificate proves, what it assumes, and what remains outside it.",
  },
  {
    slug: "encoding",
    title: "Writing legal models",
    group: "Lawlib",
    description:
      "Source references, authority, findings, and disciplined legal encoding.",
  },
  {
    slug: "coverage",
    title: "Coverage & limitations",
    group: "Lawlib",
    description:
      "A transparent inventory of the initial examples and uninvestigated areas.",
  },
  {
    slug: "deadlines",
    title: "Federal civil deadlines",
    group: "Lawlib",
    description:
      "Bounded Rule 6(a) date computation and supplied legal findings.",
  },
  {
    slug: "deadline-review",
    title: "Deadline review packet",
    group: "Lawlib",
    description:
      "The pending legal assessment, source mapping, and review questions.",
  },
];
function section(text: string, title: string) {
  return text.split(`## ${title}\n`)[1]?.split("\n## ")[0] ?? "";
}
export function docContent(slug: string) {
  if (slug === "outcomes")
    return section(documents.language, "Knowledge and outcomes");
  if (slug === "time")
    return (
      section(documents.language, "Calendars and effects") +
      "\n\n## Legal time is authored\n\nAdoption, enactment, commencement, and applicability are distinct. A policy adopted in 2026 may commence in 2029; a later amendment may preserve earlier conduct through a savings clause. No current date or latest-version selection is implicit.\n\nScenario event dates, an explicit historical-authority cutoff, and the pinned program snapshot answer different questions. Unknown triggering events remain unknown.\n\nExplore the [household policy](/explorer/private/household/demo) and the [Twentieth Amendment examples](/explorer/us/federal/constitution) to inspect these mechanisms in real `.oath` files.\n\nA household has membership and location relationships. Directory nesting does not make it a legal descendant of a city. Authority is encoded explicitly.\n"
    );
  return (
    documents[slug as keyof typeof documents]?.replace(/^# .*\n/, "") ?? ""
  );
}
export function filterPackages(search: string, category: string) {
  const needle = search.trim().toLowerCase();
  return packages.filter(
    (p) =>
      (category === "All" || p.category === category) &&
      (!needle ||
        [
          p.id,
          p.title,
          p.description,
          ...p.modules.flatMap((m) => [m.name, m.content]),
          ...p.fixtures.map((f) => f.name),
        ]
          .join(" ")
          .toLowerCase()
          .includes(needle)),
  );
}
