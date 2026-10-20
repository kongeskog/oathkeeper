import { Suspense } from "react";
import Link from "next/link";
import { ConstitutionExplorer } from "@/components/constitution-explorer";
import inventory from "@/content/constitution.json";
import { Badge } from "@/components/ui/badge";

export const metadata = {
  title: "Constitution and authority",
  description:
    "Explore constitutional provisions, institutional powers, amendments and executable authority questions.",
};
export default function ConstitutionPage() {
  return (
    <main id="main" className="min-w-0 py-9 pb-16 lg:pl-10">
      <p className="eyebrow">Start at the foundation</p>
      <h1 className="mt-5 text-3xl font-semibold tracking-tight sm:text-4xl">
        Constitution and authority
      </h1>
      <p className="mt-4 max-w-2xl text-sm leading-7 text-muted-foreground">
        Follow the text into institutions, powers, limits and procedures.
        Original provisions remain visible alongside selected amendment links.
        Every executable question has an explicit boundary.
      </p>
      <div className="mt-5 flex flex-wrap gap-2">
        <Badge variant="outline">
          {inventory.provisions.length} provision units
        </Badge>
        <Badge variant="outline">7 Articles · 27 Amendments</Badge>
        <Badge variant="outline">
          {inventory.relationships.length} scoped relationships
        </Badge>
        <Badge variant="secondary">Legal review: Unreviewed</Badge>
      </div>
      <div className="mt-5 flex flex-wrap gap-x-6 gap-y-3 text-xs underline underline-offset-4">
        <Link href="/docs/constitutional-authority">
          Model and coverage guide
        </Link>
        <Link href="/explorer/us/federal/constitution">
          Oath source and scenarios
        </Link>
        <a href="/inventory/constitution.json" download>
          Download inventory JSON
        </a>
      </div>
      <Suspense
        fallback={
          <div className="my-8 h-40 animate-pulse rounded-lg bg-muted" />
        }
      >
        <ConstitutionExplorer />
      </Suspense>
    </main>
  );
}
