"use client";
import Link from "next/link";
import { useSearchParams, useRouter, usePathname } from "next/navigation";
import {
  ArrowUpRight,
  Search,
  FolderOpen,
  FileCode2,
  Beaker,
  Braces,
  X,
} from "lucide-react";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { filterPackages, packageHref } from "@/lib/content";
export function ExplorerIndex() {
  const params = useSearchParams();
  const router = useRouter();
  const pathname = usePathname();
  const search = params.get("q") ?? "";
  const category = params.get("category") ?? "All";
  const filtered = filterPackages(search, category);
  function update(key: string, value: string) {
    const next = new URLSearchParams(params);
    if (value && value !== "All") next.set(key, value);
    else next.delete(key);
    router.replace(`${pathname}${next.size ? "?" + next : ""}`, {
      scroll: false,
    });
  }
  return (
    <>
      <div className="relative mt-8">
        <Search className="absolute top-3 left-3.5 size-4 text-muted-foreground" />
        <Input
          aria-label="Search lawlib"
          placeholder="Search packages, rules, sources, or scenarios…"
          className="h-10 pr-10 pl-10"
          value={search}
          onChange={(e) => update("q", e.target.value)}
        />
        {search && (
          <Button
            variant="ghost"
            size="icon-sm"
            aria-label="Clear search"
            className="absolute top-1.5 right-1.5"
            onClick={() => update("q", "")}
          >
            <X />
          </Button>
        )}
      </div>
      <div
        className="mt-4 flex flex-wrap items-center gap-2"
        role="group"
        aria-label="Filter jurisdiction"
      >
        {["All", "United States", "Private", "Shared"].map((c) => (
          <Button
            key={c}
            variant={category === c ? "secondary" : "ghost"}
            size="sm"
            aria-pressed={category === c}
            onClick={() => update("category", c)}
          >
            {c}
          </Button>
        ))}
        <span
          className="ml-auto text-xs text-muted-foreground"
          aria-live="polite"
        >
          {filtered.length} package{filtered.length === 1 ? "" : "s"}
        </span>
      </div>
      <div className="mt-6 grid gap-4">
        {filtered.map((p) => (
          <Link
            key={p.id}
            href={packageHref(p)}
            className="group rounded-xl border p-5 transition-colors hover:bg-muted/40 sm:p-6"
          >
            <div className="flex gap-4">
              <span className="flex size-10 shrink-0 items-center justify-center rounded-lg border bg-muted/30">
                <FolderOpen className="size-4" />
              </span>
              <div className="min-w-0 flex-1">
                <div className="flex flex-wrap items-center gap-2">
                  <h2 className="font-medium tracking-tight">{p.title}</h2>
                  <Badge variant="outline" className="text-[10px]">
                    {p.category}
                  </Badge>
                  <ArrowUpRight className="ml-auto size-4 text-muted-foreground transition-transform group-hover:-translate-y-0.5 group-hover:translate-x-0.5" />
                </div>
                <p className="mt-1.5 break-all font-mono text-[11px] text-muted-foreground">
                  {p.id}
                </p>
                <p className="mt-3 max-w-2xl text-sm leading-6 text-muted-foreground">
                  {p.description}
                </p>
                <div className="mt-5 flex flex-wrap items-center gap-x-5 gap-y-2 text-[11px] text-muted-foreground">
                  <span className="flex items-center gap-1.5">
                    <FileCode2 className="size-3" />
                    {p.modules.length} modules
                  </span>
                  <span className="flex items-center gap-1.5">
                    <Braces className="size-3" />
                    {p.modules.reduce(
                      (n, m) =>
                        n +
                        m.declarations.filter((d) => d.kind === "query").length,
                      0,
                    )}{" "}
                    queries
                  </span>
                  <span className="flex items-center gap-1.5">
                    <Beaker className="size-3" />
                    {p.fixtures.length} scenarios
                  </span>
                  <span className="ml-auto">Legal review: Unreviewed</span>
                </div>
              </div>
            </div>
          </Link>
        ))}
      </div>
      {!filtered.length && (
        <div className="mt-6 rounded-xl border border-dashed px-6 py-16 text-center">
          <Search className="mx-auto mb-4 size-6 text-muted-foreground" />
          <h2 className="font-medium">No packages found</h2>
          <p className="mt-2 text-sm text-muted-foreground">
            Try “emergency”, “constitution”, or “basis”, or clear the filters.
          </p>
          <Button
            variant="outline"
            className="mt-5"
            onClick={() => router.replace(pathname)}
          >
            Clear all filters
          </Button>
        </div>
      )}
    </>
  );
}
