"use client";
import Link from "next/link";
import { useSearchParams, useRouter } from "next/navigation";
import { useEffect, useState } from "react";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { DraftBuilder } from "./draft-builder";
import registry from "@/content/encodings.json";
import {
  catalogs,
  checkedContent,
  provisionHref,
  type ProvisionIndex,
  type Provision,
} from "@/lib/provisions";

function referenceTarget(href: string) {
  const m = href.match(/^\/us\/usc\/t(\d+)(a?)(?:\/|$)/i);
  if (!m) return null;
  const title = Number(m[1]);
  return (
    catalogs["119-121"].documents.find((d) =>
      new RegExp(
        `^usc${String(title).padStart(2, "0")}${m[2] ? "a" : ""}\\.xml$`,
        "i",
      ).test(d.file),
    )?.file ?? null
  );
}
export function ProvisionBrowser() {
  const params = useSearchParams(),
    router = useRouter();
  const release = params.get("release") ?? "119-121",
    file = params.get("file") ?? "usc28a.xml",
    selected = params.get("key"),
    target = params.get("ref");
  const catalog = catalogs[release],
    document = catalog?.documents.find((d) => d.file === file);
  const [loaded, setLoaded] = useState<{
      id: string;
      index: ProvisionIndex;
    } | null>(null),
    [detail, setDetail] = useState<{ id: string; unit: Provision } | null>(
      null,
    ),
    [failure, setFailure] = useState<{ id: string; message: string } | null>(
      null,
    ),
    [filter, setFilter] = useState({ context: "", search: "", page: 0 }),
    [retry, setRetry] = useState(0);
  const indexId = release + "/" + file,
    detailId = indexId + "/" + selected;
  const filterContext = indexId + "|" + (target ?? "");
  const search = filter.context === filterContext ? filter.search : "";
  const page = filter.context === filterContext ? filter.page : 0;
  const setSearch = (value: string) =>
    setFilter({ context: filterContext, search: value, page: 0 });
  const setPage = (value: number) =>
    setFilter((current) => ({
      context: filterContext,
      search: current.context === filterContext ? current.search : "",
      page: value,
    }));
  const index = loaded?.id === indexId ? loaded.index : null,
    unit = detail?.id === detailId ? detail.unit : null;
  const error =
    failure?.id === indexId || failure?.id === detailId
      ? failure.message
      : null;
  useEffect(() => {
    if (!document) return;
    const abort = new AbortController();
    checkedContent<ProvisionIndex>(release, document.index, abort.signal)
      .then((index) => {
        if (abort.signal.aborted) return;
        if (index.source.sha256 !== document.sha256)
          throw Error("Index/source identity mismatch");
        setLoaded({ id: indexId, index });
      })
      .catch((e) => {
        if (!abort.signal.aborted)
          setFailure({ id: indexId, message: String(e) });
      });
    return () => abort.abort();
  }, [release, document, indexId, retry]);
  useEffect(() => {
    if (!index || !selected) return;
    const row = index.units.find((u) => u.key === selected),
      ref = index.chunks.find((c) => c.path === row?.chunk);
    if (!row || !ref) return;
    const abort = new AbortController();
    checkedContent<Provision[]>(release, ref, abort.signal)
      .then((units) => {
        if (abort.signal.aborted) return;
        const unit = units.find((u) => u.key === selected);
        if (!unit || unit.content_sha256 !== row.content_sha256)
          throw Error("Provision/index mismatch");
        setDetail({ id: detailId, unit });
      })
      .catch((e) => {
        if (!abort.signal.aborted)
          setFailure({ id: detailId, message: String(e) });
      });
    return () => abort.abort();
  }, [index, selected, release, detailId, retry]);
  if (!catalog || !document)
    return (
      <p role="alert">
        Unknown pinned release or document.{" "}
        <Link href={provisionHref("119-121", "usc28a.xml")}>
          Open the current collection
        </Link>
        .
      </p>
    );
  const needle = search.trim().toLowerCase();
  const targetMatches =
    target && index
      ? index.units.filter(
          (u) =>
            u.locator &&
            (target === u.locator || target.startsWith(u.locator + "/")),
        )
      : [];
  const longest = Math.max(0, ...targetMatches.map((u) => u.locator!.length));
  const rows = (
    target
      ? targetMatches.filter((u) => u.locator!.length === longest)
      : (index?.units ?? [])
  ).filter(
    (u) =>
      !needle ||
      [u.number, u.heading, u.locator, u.kind]
        .join(" ")
        .toLowerCase()
        .includes(needle),
  );
  const links = unit
    ? registry.encodings.filter((e) =>
        e.sources.some(
          (s) => s.release === release && s.file === file && s.key === unit.key,
        ),
      )
    : [];
  const select = (key: string) =>
    router.push(provisionHref(release, file, key), { scroll: false });
  return (
    <>
      <div className="mt-7 grid gap-4 sm:grid-cols-[9rem_1fr]">
        <label className="text-xs">
          Pinned release
          <select
            aria-label="Pinned release"
            className="mt-2 h-10 w-full rounded-md border bg-background px-3 text-sm"
            value={release}
            onChange={(e) => {
              setSearch("");
              setPage(0);
              router.push(provisionHref(e.target.value, file));
            }}
          >
            {Object.keys(catalogs).map((r) => (
              <option key={r}>{r}</option>
            ))}
          </select>
        </label>
        <label className="min-w-0 text-xs">
          Source document
          <select
            aria-label="Source document"
            className="mt-2 h-10 w-full min-w-0 rounded-md border bg-background px-3 text-sm"
            value={file}
            onChange={(e) => {
              setSearch("");
              setPage(0);
              router.push(provisionHref(release, e.target.value));
            }}
          >
            {catalog.documents.map((d) => (
              <option value={d.file} key={d.file}>
                {d.title || d.file}
              </option>
            ))}
          </select>
        </label>
      </div>
      <div className="mt-4 flex flex-wrap gap-2">
        <Badge variant="outline">
          {catalog.count.toLocaleString("en-US")} source units
        </Badge>
        <Badge variant="secondary">Legal review: Unreviewed</Badge>
      </div>
      {error ? (
        <div role="alert" className="mt-6 rounded-md border p-4 text-sm">
          <p>{error}</p>
          <Button
            className="mt-3"
            variant="outline"
            onClick={() => {
              setFailure(null);
              setRetry(retry + 1);
            }}
          >
            Retry source data
          </Button>
        </div>
      ) : !index ? (
        <p role="status" className="mt-6 text-sm text-muted-foreground">
          Loading and checking source index…
        </p>
      ) : null}
      {index && (
        <>
          {target && (
            <p className="mt-5 text-sm leading-6">
              Reference target: <code className="break-all">{target}</code>.{" "}
              {targetMatches.length
                ? "Choose a matching source unit below; this is a navigation match, not a legal dependency."
                : "No source unit in this snapshot matches. The reference remains unresolved."}{" "}
              <Link className="underline" href={provisionHref(release, file)}>
                Clear reference
              </Link>
            </p>
          )}
          <label className="mt-6 block text-xs">
            Find by number, heading, or locator
            <Input
              type="search"
              className="mt-2"
              value={search}
              onChange={(e) => {
                setSearch(e.target.value);
                setPage(0);
              }}
              placeholder="e.g. Rule 6 or /Civil/rule12"
            />
          </label>
          <p className="mt-3 text-xs text-muted-foreground">
            {rows.length.toLocaleString("en-US")} matches · metadata search
            within this document
          </p>
          <div
            className="mt-4 max-h-80 overflow-auto rounded-lg border"
            aria-label="Source units"
          >
            {rows.slice(page * 30, (page + 1) * 30).map((row) => (
              <button
                key={row.key}
                onClick={() => select(row.key)}
                aria-pressed={selected === row.key}
                className={`block w-full border-b px-4 py-3 text-left text-sm last:border-0 hover:bg-muted ${selected === row.key ? "bg-muted" : ""}`}
              >
                <span className="font-medium">
                  {row.number || row.kind}{" "}
                  {row.heading ||
                    (!row.number ? "— standalone source material" : "")}
                </span>
                <span className="mt-1 block break-all font-mono text-[10px] text-muted-foreground">
                  {row.locator ?? `Snapshot key: ${row.key}`}
                </span>
              </button>
            ))}
            {!rows.length && (
              <p className="p-5 text-sm text-muted-foreground">
                No matching source units.
              </p>
            )}
          </div>
          <div className="mt-3 flex items-center justify-between">
            <Button
              variant="outline"
              size="sm"
              disabled={page === 0}
              onClick={() => setPage(page - 1)}
            >
              Previous
            </Button>
            <span className="text-xs text-muted-foreground">
              Page {page + 1} of {Math.max(1, Math.ceil(rows.length / 30))}
            </span>
            <Button
              variant="outline"
              size="sm"
              disabled={(page + 1) * 30 >= rows.length}
              onClick={() => setPage(page + 1)}
            >
              Next
            </Button>
          </div>
          {selected && !index.units.some((u) => u.key === selected) && (
            <p role="alert" className="mt-6 text-sm">
              This snapshot key is not in the selected document.
            </p>
          )}
          {selected &&
            !unit &&
            index.units.some((u) => u.key === selected) &&
            !error && (
              <p role="status" className="mt-6 text-sm">
                Loading and checking provision text…
              </p>
            )}
          {unit && (
            <article
              aria-label="Selected provision"
              className="mt-9 min-w-0 border-t pt-7"
            >
              <p className="eyebrow">Pinned source passage</p>
              <h2 className="mt-3 text-2xl font-semibold tracking-tight">
                {unit.number} {unit.heading || unit.kind}
              </h2>
              <p className="mt-3 break-all font-mono text-xs text-muted-foreground">
                {unit.locator ?? "No publisher locator; use the snapshot key"}
              </p>
              <p className="mt-3 text-xs leading-6 text-muted-foreground">
                {unit.hierarchy
                  .map((h) => [h.number, h.heading].filter(Boolean).join(" "))
                  .filter(Boolean)
                  .join(" / ")}
              </p>
              <details className="mt-4 rounded-lg border p-4">
                <summary className="cursor-pointer text-xs">
                  Source version and integrity
                </summary>
                <dl className="mt-3 space-y-3 text-xs leading-6">
                  <div>
                    <dt>Original XML SHA-256</dt>
                    <dd className="break-all font-mono">{document.sha256}</dd>
                  </div>
                  <div>
                    <dt>Extracted content SHA-256</dt>
                    <dd className="break-all font-mono">
                      {unit.content_sha256}
                    </dd>
                  </div>
                  <div>
                    <dt>Snapshot key</dt>
                    <dd>{unit.key}</dd>
                  </div>
                </dl>
                <a
                  href={catalog.source.url}
                  className="mt-4 inline-block text-xs underline"
                  target="_blank"
                  rel="noreferrer"
                >
                  Original official archive
                </a>
                <p className="mt-3 text-xs text-muted-foreground">
                  Index and text shard hashes checked in this browser. Original
                  archive integrity was checked during extraction. Neither check
                  establishes legal force.
                </p>
              </details>
              <div className="mt-6 whitespace-pre-wrap break-words text-sm leading-7">
                {unit.body ||
                  "This source unit consists of notes or credits shown below."}
              </div>
              <details className="mt-6 rounded-lg border p-4">
                <summary className="cursor-pointer text-sm">
                  Notes ({unit.notes.length})
                </summary>
                <div className="mt-4 space-y-6">
                  {unit.notes.map((n, i) => (
                    <section key={i}>
                      <p className="text-xs text-muted-foreground">
                        {[n.group, n.topic, n.role]
                          .filter(Boolean)
                          .join(" · ") || "Publisher note"}
                      </p>
                      <p className="mt-2 whitespace-pre-wrap break-words text-sm leading-7">
                        {n.text}
                      </p>
                    </section>
                  ))}
                </div>
                <p className="mt-3 text-xs text-muted-foreground">
                  Publisher labels are preserved. A note can contain enacted
                  law; its placement alone does not decide its authority.
                </p>
              </details>
              <details className="mt-3 rounded-lg border p-4">
                <summary className="cursor-pointer text-sm">
                  Source credits ({unit.credits.length})
                </summary>
                {unit.credits.map((c, i) => (
                  <p
                    key={i}
                    className="mt-4 whitespace-pre-wrap text-sm leading-7"
                  >
                    {c}
                  </p>
                ))}
              </details>
              <details className="mt-3 rounded-lg border p-4">
                <summary className="cursor-pointer text-sm">
                  Cross-references ({unit.references.length})
                </summary>
                <ul className="mt-4 max-h-96 space-y-4 overflow-auto text-sm">
                  {unit.references.map((r, i) => {
                    const targetFile = referenceTarget(r.href);
                    return (
                      <li key={i}>
                        <span className="mr-2 text-xs text-muted-foreground">
                          {r.context}
                        </span>
                        {targetFile ? (
                          <Link
                            className="underline"
                            href={
                              provisionHref(release, targetFile) +
                              "&ref=" +
                              encodeURIComponent(r.href)
                            }
                          >
                            {r.text || r.href}
                          </Link>
                        ) : (
                          <span>
                            {r.text || r.href}{" "}
                            <span className="text-xs text-muted-foreground">
                              — not resolved in this collection
                            </span>
                          </span>
                        )}
                        <code className="mt-1 block break-all text-[10px] text-muted-foreground">
                          {r.href}
                        </code>
                      </li>
                    );
                  })}
                </ul>
              </details>
              <section className="mt-6 rounded-lg border p-5">
                <h3 className="font-medium">Linked encodings</h3>
                {links.length ? (
                  links.map((e) => (
                    <div key={e.id} className="mt-4">
                      <Link
                        className="text-sm underline"
                        href={"/explorer/" + e.package}
                      >
                        {e.id}
                      </Link>
                      <p className="mt-2 text-xs">
                        {e.status.implementation} · {e.status.legal_review} ·{" "}
                        {e.tests.length} authored fixtures
                      </p>
                      <ul className="mt-2 list-disc pl-5 text-xs leading-6 text-muted-foreground">
                        {e.unresolved.map((u, i) => (
                          <li key={i}>{u}</li>
                        ))}
                      </ul>
                    </div>
                  ))
                ) : (
                  <p className="mt-3 text-sm leading-6 text-muted-foreground">
                    No encoding is registered for this passage and snapshot.
                    Executable coverage has not been assessed. This is not a
                    reviewed exclusion.
                  </p>
                )}
              </section>
              <DraftBuilder
                key={detailId}
                catalog={catalog}
                document={document}
                unit={unit}
              />
            </article>
          )}
        </>
      )}
    </>
  );
}
