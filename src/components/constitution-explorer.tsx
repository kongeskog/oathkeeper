"use client";
import Link from "next/link";
import { usePathname, useSearchParams } from "next/navigation";
import { ArrowRight, Search } from "lucide-react";
import data from "@/content/constitution.json";
import { packages, release } from "@/lib/content";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { cn } from "@/lib/utils";

type Provision = (typeof data.provisions)[number];
type Relationship = (typeof data.relationships)[number];
const statusLabels: Record<string, string> = {
  Represented: "Represented",
  InterpretationRequired: "Interpretation required",
  ExecutableSubset: "Executable subset",
};
const groups = [...new Set(data.provisions.map((p) => p.group_id))];
function groupLabel(id: string) {
  if (id === "us.const") return "Preamble";
  return (
    data.provisions.find((p) => p.group_id === id)?.label.split(",")[0] ?? id
  );
}
function endpoint(id: string) {
  return (
    data.provisions.find((p) => p.id === id)?.label ??
    data.institutions.find((i) => i.id === id)?.description ??
    id
  );
}
function queryLink(query: string) {
  const pkg = packages.find((p) => p.path === "us/federal/constitution")!;
  for (const oathModule of pkg.modules) {
    const declaration = oathModule.declarations.find(
      (d) => d.kind === "query" && `${oathModule.name}.${d.name}` === query,
    );
    if (declaration)
      return `/explorer/${pkg.path}?tab=source&file=${encodeURIComponent(oathModule.path)}&line=${declaration.line}`;
  }
  return `/explorer/${pkg.path}`;
}
export function ConstitutionExplorer() {
  const params = useSearchParams(),
    pathname = usePathname();
  const view = params.get("view") === "authority" ? "authority" : "provisions";
  const search = params.get("q") ?? "",
    group = params.get("group") ?? "",
    status = params.get("status") ?? "";
  const kind = params.get("kind") ?? "",
    actor = params.get("actor") ?? "";
  function update(values: Record<string, string>, push = false) {
    // Filters only change this client view. Read the latest URL so a second
    // quick interaction cannot restore parameters from an older render.
    const next = new URLSearchParams(window.location.search);
    for (const [key, value] of Object.entries(values)) {
      if (value) next.set(key, value);
      else next.delete(key);
    }
    window.history[push ? "pushState" : "replaceState"](
      null,
      "",
      `${pathname}${next.size ? "?" + next : ""}`,
    );
  }
  const filtered = data.provisions.filter(
    (p) =>
      (!group || p.group_id === group) &&
      (!status || p.analysis === status) &&
      `${p.label} ${p.title} ${p.text}`
        .toLowerCase()
        .includes(search.toLowerCase()),
  );
  const selected = params.has("provision")
    ? data.provisions.find((p) => p.id === params.get("provision"))
    : filtered[0];
  const relations = data.relationships.filter(
    (r) =>
      (!kind || r.kind === kind) &&
      (!actor || r.from === actor || r.to === actor),
  );
  function showProvision(id: string) {
    update(
      {
        view: "",
        provision: id,
        group: "",
        status: "",
        q: "",
        actor: "",
        kind: "",
      },
      true,
    );
  }
  function relationship(r: Relationship) {
    return (
      <article key={r.id} className="rounded-lg border p-4">
        <div className="flex flex-wrap items-center gap-2 text-sm">
          <button
            className="text-left underline underline-offset-4"
            onClick={() => showProvision(r.from)}
          >
            {endpoint(r.from)}
          </button>
          <ArrowRight
            className="size-3 shrink-0 text-muted-foreground"
            aria-hidden="true"
          />
          <Badge variant="outline">{r.kind}</Badge>
          <ArrowRight
            className="size-3 shrink-0 text-muted-foreground"
            aria-hidden="true"
          />
          {data.provisions.some((p) => p.id === r.to) ? (
            <button
              className="text-left underline underline-offset-4"
              onClick={() => showProvision(r.to)}
            >
              {endpoint(r.to)}
            </button>
          ) : (
            <button
              className="text-left underline underline-offset-4"
              onClick={() =>
                update({ view: "authority", actor: r.to, kind: "" }, true)
              }
            >
              {endpoint(r.to)}
            </button>
          )}
        </div>
        <p className="mt-3 text-sm leading-6 text-muted-foreground">
          {r.scope}
        </p>
        <p className="mt-2 text-xs leading-5 text-muted-foreground">
          {r.assessment_queries.length
            ? "Related questions have partial executable coverage."
            : "Represented relationship; no executable authority test linked."}
        </p>
      </article>
    );
  }
  function detail(p: Provision) {
    const pin = data.source_pins.find((s) => s.id === p.anchor.source_id)!;
    const audit = release.audit.sources.find(
      (s) => s.source === pin.id && s.sha256 === pin.sha256,
    );
    const linked = data.relationships.filter(
      (r) => r.from === p.id || r.to === p.id,
    );
    return (
      <section
        aria-label="Selected provision"
        className="min-w-0 rounded-xl border p-5 sm:p-6"
      >
        <p className="eyebrow">{p.label}</p>
        <h2 className="mt-3 text-xl font-semibold tracking-tight">{p.title}</h2>
        <div className="mt-3 flex flex-wrap gap-2">
          <Badge variant="outline">{statusLabels[p.analysis]}</Badge>
          <Badge variant="secondary">Unreviewed</Badge>
        </div>
        <p className="mt-4 break-all font-mono text-[10px] text-muted-foreground">
          {p.id}
        </p>
        <h3 className="mt-6 text-xs font-medium">Original provision text</h3>
        <blockquote className="mt-3 border-l-2 pl-4 text-sm leading-7">
          {p.text}
        </blockquote>
        <p className="mt-3 text-xs leading-5 text-muted-foreground">
          Editorial title above; original wording with normalized whitespace
          below the label. This is not a consolidated statement of current law.
        </p>
        <a
          href={p.url}
          target="_blank"
          rel="noreferrer"
          className="mt-3 inline-block text-xs underline underline-offset-4"
        >
          Open National Archives source
        </a>
        {p.editorial_notes.length > 0 && (
          <details className="mt-5 rounded-lg bg-muted/40 p-4">
            <summary className="cursor-pointer text-xs font-medium">
              Publisher’s history notes
            </summary>
            <ul className="mt-3 space-y-2 text-xs leading-6 text-muted-foreground">
              {p.editorial_notes.map((n) => (
                <li key={n}>{n}</li>
              ))}
            </ul>
          </details>
        )}
        <div className="mt-6 border-t pt-5">
          <h3 className="text-sm font-medium">Executable questions</h3>
          {p.executable_queries.length ? (
            <ul className="mt-3 space-y-4">
              {p.executable_queries.map((query) => {
                const fixtures = packages
                  .find((pkg) => pkg.path === "us/federal/constitution")!
                  .fixtures.filter((f) => f.request.query === query);
                return (
                  <li key={query}>
                    <Link
                      href={queryLink(query)}
                      className="break-all font-mono text-xs underline underline-offset-4"
                    >
                      {query}
                    </Link>
                    {fixtures.length > 0 && (
                      <Link
                        className="mt-1 block text-xs text-muted-foreground underline underline-offset-4"
                        href={`/explorer/us/federal/constitution?tab=scenarios&scenario=${encodeURIComponent(fixtures[0].name)}`}
                      >
                        {fixtures.length} authored scenarios
                      </Link>
                    )}
                  </li>
                );
              })}
            </ul>
          ) : (
            <p className="mt-3 text-sm text-muted-foreground">
              No executable question linked.
            </p>
          )}
          <ul className="mt-4 space-y-2 text-xs leading-6 text-muted-foreground">
            {p.unresolved.map((s) => (
              <li key={s}>{s}</li>
            ))}
          </ul>
        </div>
        <div className="mt-6 border-t pt-5">
          <h3 className="mb-3 text-sm font-medium">
            Authority and historical relationships
          </h3>
          <div className="space-y-3">
            {linked.length ? (
              linked.map(relationship)
            ) : (
              <p className="text-sm leading-6 text-muted-foreground">
                No relationships recorded for this provision yet. This does not
                establish that none exist.
              </p>
            )}
          </div>
        </div>
        <details className="mt-6 border-t pt-5 text-xs">
          <summary className="cursor-pointer font-medium">
            Source integrity and extraction
          </summary>
          <dl className="mt-4 space-y-3 break-all leading-5 text-muted-foreground">
            <div>
              <dt>Source identity</dt>
              <dd className="font-mono">{pin.id}</dd>
            </div>
            <div>
              <dt>Original document SHA-256</dt>
              <dd className="font-mono">{pin.sha256}</dd>
            </div>
            <div>
              <dt>Normalized text SHA-256</dt>
              <dd className="font-mono">{p.text_sha256}</dd>
            </div>
            <div>
              <dt>Recorded byte audit</dt>
              <dd>
                {audit?.status === "matched"
                  ? "Matched in the recorded 2026-10-19 audit"
                  : "Not checked"}
                ; source bytes are not fetched by this page.
              </dd>
            </div>
          </dl>
          <p className="mt-4 leading-6 text-muted-foreground">
            {data.segmentation}
          </p>
        </details>
      </section>
    );
  }
  const selectClass =
    "h-10 min-w-0 rounded-md border bg-background px-3 text-sm";
  return (
    <div className="mt-8">
      <Tabs
        value={view}
        onValueChange={(v) =>
          update({ view: v === "authority" ? "authority" : "" })
        }
      >
        <TabsList>
          <TabsTrigger value="provisions">Provisions</TabsTrigger>
          <TabsTrigger value="authority">Authority relationships</TabsTrigger>
        </TabsList>
      </Tabs>
      {view === "provisions" ? (
        <>
          <div className="mt-5 grid gap-3 sm:grid-cols-2">
            <div className="relative sm:col-span-2">
              <Search className="absolute top-3 left-3 size-4 text-muted-foreground" />
              <Input
                aria-label="Search constitutional provisions"
                placeholder="Search text, provision, or topic…"
                className="h-10 pl-9"
                value={search}
                onChange={(e) => update({ q: e.target.value, provision: "" })}
              />
            </div>
            <select
              aria-label="Article or amendment"
              className={selectClass}
              value={group}
              onChange={(e) => update({ group: e.target.value, provision: "" })}
            >
              <option value="">All Articles and Amendments</option>
              {groups.map((g) => (
                <option key={g} value={g}>
                  {groupLabel(g)}
                </option>
              ))}
            </select>
            <select
              aria-label="Encoding status"
              className={selectClass}
              value={status}
              onChange={(e) =>
                update({ status: e.target.value, provision: "" })
              }
            >
              <option value="">All encoding statuses</option>
              {Object.entries(statusLabels).map(([key, label]) => (
                <option key={key} value={key}>
                  {label}
                </option>
              ))}
            </select>
          </div>
          <p aria-live="polite" className="mt-4 text-xs text-muted-foreground">
            {filtered.length} provision units shown. Status describes coverage
            of the encoding.
          </p>
          {!filtered.length ? (
            <div className="mt-5 rounded-xl border border-dashed p-8 text-center">
              <h2 className="font-medium">No provisions found</h2>
              <Button
                className="mt-4"
                variant="outline"
                onClick={() =>
                  update({ q: "", group: "", status: "", provision: "" })
                }
              >
                Clear filters
              </Button>
            </div>
          ) : (
            <div className="mt-5 grid items-start gap-5 xl:grid-cols-[235px_minmax(0,1fr)]">
              <nav
                aria-label="Constitutional provisions"
                className="max-h-72 overflow-y-auto rounded-xl border p-2 xl:max-h-[70vh]"
              >
                {filtered.map((p) => (
                  <button
                    key={p.id}
                    aria-current={selected?.id === p.id ? "true" : undefined}
                    onClick={() => update({ provision: p.id }, true)}
                    className={cn(
                      "mb-1 block w-full rounded-lg px-3 py-3 text-left hover:bg-muted/50",
                      selected?.id === p.id && "bg-muted",
                    )}
                  >
                    <span className="block text-xs font-medium">{p.label}</span>
                    <span
                      className={cn(
                        "mt-1 block text-xs leading-5",
                        selected?.id === p.id
                          ? "text-foreground"
                          : "text-muted-foreground",
                      )}
                    >
                      {p.title}
                    </span>
                  </button>
                ))}
              </nav>
              {selected ? (
                detail(selected)
              ) : (
                <div className="rounded-xl border p-6">
                  <h2 className="font-medium">Unknown provision</h2>
                  <p className="mt-2 text-sm text-muted-foreground">
                    This provision ID is not present in the pinned inventory.
                  </p>
                  <Button
                    className="mt-4"
                    variant="outline"
                    onClick={() => update({ provision: "" })}
                  >
                    Return to inventory
                  </Button>
                </div>
              )}
            </div>
          )}
        </>
      ) : (
        <>
          <p className="mt-5 max-w-2xl text-sm leading-7 text-muted-foreground">
            These links describe who holds a power, what constrains it, and how
            provisions change. A graph path does not establish validity or
            automatically select a winning rule. State powers are not treated as
            ordinary delegations from Congress.
          </p>
          <div className="mt-5 grid gap-3 sm:grid-cols-2">
            <select
              aria-label="Relationship kind"
              className={selectClass}
              value={kind}
              onChange={(e) => update({ kind: e.target.value })}
            >
              <option value="">All relationships</option>
              {[...new Set(data.relationships.map((r) => r.kind))].map((k) => (
                <option key={k}>{k}</option>
              ))}
            </select>
            <select
              aria-label="Institutional actor"
              className={selectClass}
              value={actor}
              onChange={(e) => update({ actor: e.target.value })}
            >
              <option value="">All actors and provisions</option>
              {data.institutions.map((i) => (
                <option key={i.id} value={i.id}>
                  {i.description}
                </option>
              ))}
            </select>
          </div>
          {actor && (
            <p className="mt-4 text-xs leading-6 text-muted-foreground">
              {data.institutions.find((i) => i.id === actor)?.jurisdiction ??
                "Unknown institutional actor."}
            </p>
          )}
          <p aria-live="polite" className="mt-4 text-xs text-muted-foreground">
            {relations.length} relationships shown · all Unreviewed
          </p>
          <div className="mt-5 space-y-3">{relations.map(relationship)}</div>
          {!relations.length && (
            <div className="mt-5 rounded-xl border p-8 text-center">
              <p>No matching relationships.</p>
              <Button
                variant="outline"
                className="mt-4"
                onClick={() => update({ actor: "", kind: "" })}
              >
                Clear relationship filters
              </Button>
            </div>
          )}
        </>
      )}
    </div>
  );
}
