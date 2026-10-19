import { Suspense } from "react";
import Link from "next/link";
import { notFound } from "next/navigation";
import { ChevronRight } from "lucide-react";
import { packages, release } from "@/lib/content";
import { Badge } from "@/components/ui/badge";
import { PackageExplorer } from "@/components/package-explorer";
export const dynamicParams = false;
export function generateStaticParams() {
  return packages.map((p) => ({ slug: p.path.split("/") }));
}
export async function generateMetadata({
  params,
}: {
  params: Promise<{ slug: string[] }>;
}) {
  const { slug } = await params;
  const pkg = packages.find((p) => p.path === slug.join("/"));
  return { title: pkg?.title, description: pkg?.description };
}
export default async function PackagePage({
  params,
}: {
  params: Promise<{ slug: string[] }>;
}) {
  const { slug } = await params;
  const pkg = packages.find((p) => p.path === slug.join("/"));
  if (!pkg) notFound();
  return (
    <main id="main" className="min-w-0 py-9 pb-16 lg:pl-10">
      <nav
        aria-label="Breadcrumb"
        className="flex flex-wrap items-center gap-2 text-[11px] text-muted-foreground"
      >
        <Link href="/explorer" className="hover:text-foreground">
          Lawlib
        </Link>
        {pkg.path.split("/").map((part, i) => (
          <span key={i} className="flex items-center gap-2">
            <ChevronRight className="size-3" />
            {part}
          </span>
        ))}
      </nav>
      <div className="mt-6 flex flex-wrap items-center gap-2">
        <Badge variant="secondary">{pkg.category}</Badge>
        <Badge variant="outline">Legal review: Unreviewed</Badge>
      </div>
      <h1 className="mt-4 text-3xl font-semibold tracking-tight sm:text-4xl">
        {pkg.title}
      </h1>
      <p className="mt-3 break-all font-mono text-[11px] text-muted-foreground">
        {pkg.id} <span className="mx-2">@</span> {pkg.version}
      </p>
      <p className="mt-4 max-w-2xl text-sm leading-7 text-muted-foreground">
        {pkg.description}
      </p>
      <Suspense
        fallback={
          <div className="my-8 h-80 animate-pulse rounded-xl bg-muted" />
        }
      >
        <PackageExplorer pkg={pkg} />
      </Suspense>
      <div className="mt-10 border-t pt-5 text-[10px] text-muted-foreground">
        Pinned lawlib snapshot{" "}
        <span className="font-mono">{release.lawlibRevision.slice(0, 12)}</span>{" "}
        ·{" "}
        <Link
          href="/docs/verification"
          className="underline underline-offset-4"
        >
          Understand verification
        </Link>
      </div>
    </main>
  );
}
