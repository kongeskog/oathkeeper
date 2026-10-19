import { Suspense } from "react";
import { ExplorerIndex } from "@/components/explorer-index";
import { counts, release } from "@/lib/content";
import { Badge } from "@/components/ui/badge";
export const metadata = {
  title: "Lawlib explorer",
  description:
    "Browse real Oathkeeper legal models, source references, coverage boundaries, and scenario fixtures.",
};
export default function Explorer() {
  return (
    <main id="main" className="min-w-0 py-9 pb-16 lg:pl-10">
      <div className="flex items-center justify-between gap-3">
        <p className="eyebrow">The legal library</p>
        <Badge variant="outline" className="font-mono text-[10px]">
          {release.version}
        </Badge>
      </div>
      <h1 className="mt-5 text-3xl font-semibold tracking-tight sm:text-4xl">
        Lawlib explorer
      </h1>
      <p className="mt-4 max-w-xl text-sm leading-7 text-muted-foreground">
        Read the rules. Follow their sources. See where the model stops. Every
        file here comes from the pinned lawlib repository.
      </p>
      <div className="mt-7 grid grid-cols-3 divide-x rounded-xl border bg-muted/20 py-4">
        {[
          [counts.packages, "Packages"],
          [counts.queries, "Queries"],
          [counts.fixtures, "Scenario fixtures"],
        ].map(([n, label]) => (
          <div key={label} className="px-4 sm:px-6">
            <p className="font-mono text-xl tracking-tight">{n}</p>
            <p className="mt-1 text-[11px] text-muted-foreground">{label}</p>
          </div>
        ))}
      </div>
      <Suspense
        fallback={
          <div className="my-8 h-40 animate-pulse rounded-lg bg-muted" />
        }
      >
        <ExplorerIndex />
      </Suspense>
      <p className="mt-7 text-xs leading-6 text-muted-foreground">
        This collection establishes an initial foundation. It does not encode
        all US law. Legal review is Unreviewed; source-byte audit results are
        dated and reported separately.
      </p>
    </main>
  );
}
