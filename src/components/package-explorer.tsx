"use client";
import Link from "next/link";
import { useEffect, useState } from "react";
import { useSearchParams, useRouter, usePathname } from "next/navigation";
import {
  ArrowRight,
  ArrowUpRight,
  Braces,
  Check,
  Download,
  FileCode2,
  Link2,
  Search,
  ShieldCheck,
} from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { buttonVariants } from "@/components/ui/button";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import { Input } from "@/components/ui/input";
import { CodeBlock, CopyButton } from "@/components/code-block";
import { type LawPackage, release } from "@/lib/content";
import { cn } from "@/lib/utils";

function pretty(value: unknown): string {
  if (value === null || value === undefined) return "No asserted value";
  if (typeof value !== "object") return String(value);
  const v = value as Record<string, unknown>;
  if (v.type === "bool" || v.type === "int" || v.type === "text")
    return String(v.value);
  if (v.type === "rational")
    return v.denominator === "1"
      ? String(v.numerator)
      : `${v.numerator} / ${v.denominator}`;
  if (v.type === "variant") return String(v.variant);
  if (v.type === "record") {
    const fields = v.fields as Record<string, { value?: unknown }>;
    if (v.name === "std.money.Money")
      return `USD ${pretty(fields.amount?.value)}`;
  }
  return JSON.stringify(value, null, 2);
}
function ListSection({ title, items }: { title: string; items: string[] }) {
  return (
    <div className="rounded-lg border p-5">
      <h3 className="text-sm font-medium">{title}</h3>
      {items.length ? (
        <ul className="mt-3 space-y-2">
          {items.map((item) => (
            <li
              key={item}
              className="flex gap-2 text-xs leading-6 text-muted-foreground"
            >
              <span className="mt-2.5 size-1 shrink-0 rounded-full bg-muted-foreground" />
              {item}
            </li>
          ))}
        </ul>
      ) : (
        <p className="mt-3 text-xs leading-6 text-muted-foreground">
          No entries declared in this record.
        </p>
      )}
    </div>
  );
}
export function PackageExplorer({ pkg }: { pkg: LawPackage }) {
  const params = useSearchParams(),
    router = useRouter(),
    pathname = usePathname();
  const tab = params.get("tab") ?? "overview";
  const file =
    pkg.modules.find((m) => m.path === params.get("file")) ??
    pkg.modules.find((m) => m.declarations.some((d) => d.kind === "query")) ??
    pkg.modules[0];
  const fixture =
    pkg.fixtures.find((f) => f.name === params.get("scenario")) ??
    pkg.fixtures[0];
  const line = Number(params.get("line")) || undefined;
  const [scenarioSearch, setScenarioSearch] = useState("");
  const [symbolSearch, setSymbolSearch] = useState("");
  function select(values: Record<string, string | undefined>) {
    const next = new URLSearchParams(params);
    for (const [key, value] of Object.entries(values)) {
      if (value) next.set(key, value);
      else next.delete(key);
    }
    router.push(`${pathname}?${next}`, { scroll: false });
  }
  useEffect(() => {
    if (tab === "source" && line)
      document
        .getElementById(`L${line}`)
        ?.scrollIntoView({ block: "nearest", inline: "nearest" });
  }, [tab, line, file.path]);
  const queries = pkg.modules.flatMap((m) =>
    m.declarations
      .filter((d) => d.kind === "query")
      .map((d) => ({ ...d, file: m.path, module: m.name })),
  );
  const filteredFixtures = pkg.fixtures.filter((f) =>
    `${f.name} ${f.request.query} ${f.expected.status}`
      .toLowerCase()
      .includes(scenarioSearch.toLowerCase()),
  );
  return (
    <Tabs
      value={
        ["overview", "source", "sources", "scenarios"].includes(tab)
          ? tab
          : "overview"
      }
      onValueChange={(value) => select({ tab: String(value) })}
      className="mt-8 gap-6"
    >
      <div className="overflow-x-auto border-b pb-2">
        <TabsList
          variant="line"
          aria-label="Package sections"
          className="gap-4"
        >
          <TabsTrigger value="overview">Overview</TabsTrigger>
          <TabsTrigger value="source">
            Source{" "}
            <span className="text-[10px] text-muted-foreground">
              {pkg.modules.length}
            </span>
          </TabsTrigger>
          <TabsTrigger value="sources">
            Citations{" "}
            <span className="text-[10px] text-muted-foreground">
              {pkg.sources.length}
            </span>
          </TabsTrigger>
          <TabsTrigger value="scenarios">
            Scenarios{" "}
            <span className="text-[10px] text-muted-foreground">
              {pkg.fixtures.length}
            </span>
          </TabsTrigger>
        </TabsList>
      </div>
      <TabsContent value="overview" className="space-y-8">
        <div className="flex gap-3 rounded-xl border bg-muted/30 p-5">
          <ShieldCheck className="mt-0.5 size-4 shrink-0" />
          <div>
            <h2 className="text-sm font-medium">
              A model with explicit boundaries
            </h2>
            <p className="mt-2 text-xs leading-6 text-muted-foreground">
              Legal review is{" "}
              <strong className="font-medium text-foreground">
                Unreviewed
              </strong>
              . The queries below answer identified questions under supplied
              premises. Source mapping, source integrity, and legal validity are
              distinct.
            </p>
          </div>
        </div>
        <section>
          <div className="mb-4 flex items-center justify-between">
            <h2 className="text-base font-medium">Executable questions</h2>
            <span className="font-mono text-xs text-muted-foreground">
              {queries.length} queries
            </span>
          </div>
          <div className="divide-y overflow-hidden rounded-xl border">
            {queries.map((q) => (
              <button
                key={`${q.module}.${q.name}`}
                onClick={() =>
                  select({ tab: "source", file: q.file, line: String(q.line) })
                }
                className="group flex w-full items-center gap-3 p-4 text-left hover:bg-muted/50"
              >
                <Braces className="size-4 shrink-0 text-muted-foreground" />
                <span className="min-w-0 flex-1">
                  <span className="block break-all font-mono text-xs">
                    {q.name}
                  </span>
                  <span className="mt-1 block break-all text-[10px] text-muted-foreground">
                    {q.module} · line {q.line}
                  </span>
                </span>
                <ArrowUpRight className="size-3.5 shrink-0 text-muted-foreground" />
              </button>
            ))}
          </div>
        </section>
        {pkg.coverage.map((c) => (
          <section key={c.id}>
            <h2 className="mb-4 text-base font-medium">Declared coverage</h2>
            <p className="mb-4 break-all font-mono text-[10px] text-muted-foreground">
              {c.id}
            </p>
            <div className="grid gap-4 sm:grid-cols-2">
              <ListSection title="Questions addressed" items={c.questions} />
              <ListSection title="Explicit exclusions" items={c.exclusions} />
              <ListSection title="Not investigated" items={c.notInvestigated} />
              <ListSection
                title="Unresolved interpretations"
                items={c.unresolved}
              />
            </div>
          </section>
        ))}
        {!pkg.coverage.length && (
          <div className="rounded-lg border p-5 text-xs leading-6 text-muted-foreground">
            This shared package supplies representation types and temporal
            examples. It does not declare substantive legal coverage. Read the{" "}
            <Link
              href="/docs/coverage"
              className="underline underline-offset-4"
            >
              coverage matrix
            </Link>{" "}
            for its scope.
          </div>
        )}
        {pkg.mappings.length > 0 && (
          <section>
            <h2 className="text-base font-medium">Constitutional source map</h2>
            <p className="mt-2 text-xs leading-6 text-muted-foreground">
              {pkg.mappings.length} provisions mapped to text. A mapped
              provision is executable only where named questions are present.
            </p>
            <div className="mt-4 grid grid-cols-2 gap-2 sm:grid-cols-3">
              {pkg.mappings.map((m) => (
                <button
                  key={m.id}
                  onClick={() =>
                    select({
                      tab: "source",
                      file: "src/sources.oath",
                      line: undefined,
                    })
                  }
                  className="rounded-lg border p-3 text-left hover:bg-muted/50"
                >
                  <span className="text-xs font-medium">{m.locator}</span>
                  <span className="mt-1 block text-[10px] text-muted-foreground">
                    {m.questions.length
                      ? `${m.questions.length} selected questions`
                      : "Source mapping only"}
                  </span>
                </button>
              ))}
            </div>
          </section>
        )}
        <details className="rounded-xl border">
          <summary className="cursor-pointer px-5 py-4 text-sm font-medium">
            Package manifest
          </summary>
          <CodeBlock
            code={pkg.manifest}
            title="oath.toml"
            language="toml"
            className="rounded-t-none border-x-0 border-b-0"
          />
        </details>
      </TabsContent>
      <TabsContent value="source" className="space-y-5">
        <div className="flex flex-col gap-3 sm:flex-row sm:items-center">
          <label htmlFor="module" className="text-xs font-medium">
            Module
          </label>
          <select
            id="module"
            value={file.path}
            onChange={(e) => select({ file: e.target.value, line: undefined })}
            className="h-9 min-w-0 flex-1 rounded-lg border bg-background px-3 font-mono text-xs"
          >
            {pkg.modules.map((m) => (
              <option key={m.path} value={m.path}>
                {m.path}
              </option>
            ))}
          </select>
          <a
            href={`/lawlib/${pkg.path}/${file.path}`}
            download
            className={buttonVariants({ variant: "outline", size: "sm" })}
          >
            <Download className="size-3.5" />
            Download .oath
          </a>
        </div>
        <div className="flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
          <FileCode2 className="size-3.5" />
          <span className="break-all font-mono">{file.name}</span>
          <span className="ml-auto">
            {file.declarations.length} declarations
          </span>
        </div>
        <CodeBlock
          code={file.content}
          title={file.path}
          numbers
          highlight={line}
          maxHeight="650px"
        />
        <div className="flex items-start gap-2 rounded-md bg-muted/40 px-3 py-2">
          <span className="pt-1.5 font-mono text-[10px] text-muted-foreground">
            SHA-256
          </span>
          <span className="min-w-0 flex-1 break-all pt-1.5 font-mono text-[10px] text-muted-foreground">
            {file.sha256}
          </span>
          <CopyButton text={file.sha256} label="Copy file hash" />
        </div>
        <section>
          <div className="mb-3 flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
            <h2 className="text-sm font-medium">Declarations in this module</h2>
            <Input
              aria-label="Filter declarations"
              placeholder="Filter declarations…"
              value={symbolSearch}
              onChange={(e) => setSymbolSearch(e.target.value)}
              className="sm:w-56"
            />
          </div>
          <div className="flex flex-wrap gap-2">
            {file.declarations
              .filter((d) => `${d.kind} ${d.name}`.includes(symbolSearch))
              .map((d) => (
                <button
                  key={d.name}
                  onClick={() => select({ line: String(d.line) })}
                  className="flex items-center gap-2 rounded-md border px-2.5 py-1.5 text-left hover:bg-muted"
                >
                  <span className="text-[10px] text-muted-foreground">
                    {d.kind}
                  </span>
                  <span className="break-all font-mono text-[11px]">
                    {d.name}
                  </span>
                </button>
              ))}
          </div>
          {!file.declarations.filter((d) =>
            `${d.kind} ${d.name}`.includes(symbolSearch),
          ).length && (
            <p className="py-4 text-xs text-muted-foreground">
              No matching declarations.
            </p>
          )}
        </section>
      </TabsContent>
      <TabsContent value="sources" className="space-y-5">
        <div>
          <h2 className="text-base font-medium">Source references</h2>
          <p className="mt-2 text-xs leading-6 text-muted-foreground">
            These records pin original document bytes. The audit below was
            recorded on {release.audit.date}; it is not a live check or a legal
            review.
          </p>
        </div>
        {pkg.sources.map((s) => {
          const matched = release.audit.sources.some(
            (a) =>
              a.source === s.id &&
              a.sha256 === s.sha256 &&
              a.status === "matched",
          );
          return (
            <div key={s.id} className="rounded-xl border p-5">
              <div className="flex flex-wrap items-start justify-between gap-3">
                <div>
                  <h3 className="text-sm font-medium">{s.citation}</h3>
                  <p className="mt-1 text-xs text-muted-foreground">
                    {s.version}
                  </p>
                </div>
                <Badge variant="outline">
                  {matched ? (
                    <>
                      <Check className="size-3" />
                      Bytes matched
                    </>
                  ) : (
                    "Not checked"
                  )}
                </Badge>
              </div>
              <p className="mt-4 break-all font-mono text-[10px] text-muted-foreground">
                {s.id}
              </p>
              <p className="mt-3 text-xs leading-6 text-muted-foreground">
                {s.retrieval}
              </p>
              <div className="mt-4 flex items-start gap-2 rounded-md bg-muted/50 p-2">
                <span className="min-w-0 flex-1 break-all pt-1.5 font-mono text-[10px] text-muted-foreground">
                  SHA-256 {s.sha256}
                </span>
                <CopyButton
                  text={s.sha256}
                  label={`Copy hash for ${s.citation}`}
                />
              </div>
              <div className="mt-4 flex flex-wrap gap-4">
                {s.locations
                  .filter((url) => url.startsWith("https://"))
                  .map((url, i) => (
                    <a
                      key={url}
                      href={url}
                      target="_blank"
                      rel="noreferrer"
                      className="inline-flex items-center gap-1.5 text-xs font-medium"
                    >
                      Open original source
                      {s.locations.length > 1 ? ` ${i + 1}` : ""}
                      <ArrowUpRight className="size-3" />
                    </a>
                  ))}
                {s.locations.some((l) => l.startsWith("fixture:")) && (
                  <span className="text-xs text-muted-foreground">
                    Local fictional fixture · included in source download
                  </span>
                )}
                <button
                  className="inline-flex items-center gap-1.5 text-xs text-muted-foreground hover:text-foreground"
                  onClick={() =>
                    select({ tab: "source", file: s.file, line: undefined })
                  }
                >
                  View source record
                  <ArrowRight className="size-3" />
                </button>
              </div>
            </div>
          );
        })}
        {!pkg.sources.length && (
          <div className="rounded-xl border border-dashed p-10 text-center">
            <Link2 className="mx-auto mb-3 size-6 text-muted-foreground" />
            <p className="text-sm font-medium">
              No legal source references in this package
            </p>
            <p className="mt-2 text-xs leading-6 text-muted-foreground">
              Shared types and illustrative temporal functions do not assert
              substantive legal authority.
            </p>
          </div>
        )}
      </TabsContent>
      <TabsContent value="scenarios" className="space-y-5">
        <div>
          <h2 className="text-base font-medium">Authored scenario fixtures</h2>
          <p className="mt-2 text-xs leading-6 text-muted-foreground">
            Inspect the supplied observations and expected outcomes from the
            repository. This browser displays fixtures; run and independently
            verify them with the Oathkeeper CLI.
          </p>
        </div>
        <div className="relative">
          <Search className="absolute top-2.5 left-3 size-3.5 text-muted-foreground" />
          <Input
            aria-label="Search scenarios"
            className="pl-9"
            placeholder="Search scenario names, queries, or outcomes…"
            value={scenarioSearch}
            onChange={(e) => setScenarioSearch(e.target.value)}
          />
        </div>
        <div className="grid gap-5 xl:grid-cols-[240px_minmax(0,1fr)]">
          <div
            className="code-scroll max-h-[540px] overflow-y-auto rounded-lg border p-1"
            role="group"
            aria-label="Scenarios"
          >
            {filteredFixtures.map((f) => (
              <button
                key={f.name}
                onClick={() => select({ scenario: f.name })}
                aria-pressed={fixture.name === f.name}
                className={cn(
                  "mb-1 block w-full rounded-md px-3 py-3 text-left",
                  fixture.name === f.name ? "bg-muted" : "hover:bg-muted/50",
                )}
              >
                <span className="block break-words text-xs font-medium">
                  {f.name.replaceAll("_", " ")}
                </span>
                <span
                  className={cn(
                    "mt-1.5 block text-[10px]",
                    fixture.name === f.name
                      ? "text-foreground"
                      : "text-muted-foreground",
                  )}
                >
                  {f.expected.status ?? f.expected.kind}
                </span>
              </button>
            ))}
            {!filteredFixtures.length && (
              <p className="p-4 text-xs leading-6 text-muted-foreground">
                No matching scenarios. Try a query name or outcome.
              </p>
            )}
          </div>
          <div className="min-w-0 space-y-4">
            <div className="rounded-xl border p-5">
              <div className="mb-3 flex flex-wrap items-center justify-between gap-2">
                <p className="eyebrow">Expected outcome</p>
                <Badge variant="secondary" className="capitalize">
                  {fixture.expected.status ?? fixture.expected.kind}
                </Badge>
              </div>
              <h3 className="break-words text-sm font-medium">
                {fixture.name.replaceAll("_", " ")}
              </h3>
              {fixture.expected.value !== undefined && (
                <p className="mt-4 whitespace-pre-wrap break-all font-mono text-xl tracking-tight">
                  {pretty(fixture.expected.value)}
                </p>
              )}
              <p className="mt-3 break-all font-mono text-[10px] leading-5 text-muted-foreground">
                {fixture.request.query}
              </p>
            </div>
            <CodeBlock
              code={JSON.stringify(fixture.expected, null, 2)}
              title="Expected result assertions"
              language="json"
              maxHeight="250px"
            />
            <CodeBlock
              code={JSON.stringify(fixture.request, null, 2)}
              title="Scenario request"
              language="json"
              maxHeight="400px"
            />
            <Link
              className="inline-flex items-center gap-2 text-xs font-medium"
              href="/docs/formats#commands"
            >
              Run fixtures with oath test
              <ArrowRight className="size-3" />
            </Link>
          </div>
        </div>
      </TabsContent>
    </Tabs>
  );
}
