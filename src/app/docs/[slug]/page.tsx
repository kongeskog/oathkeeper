import { readFile } from "node:fs/promises";
import path from "node:path";
import Link from "next/link";
import { notFound } from "next/navigation";
import { ArrowLeft, ArrowRight } from "lucide-react";
import { Markdown } from "@/components/markdown";
import { Badge } from "@/components/ui/badge";
import {
  docPages,
  docContent,
  slugify,
  tutorial,
  tutorialRequest,
  release,
} from "@/lib/content";
export const dynamicParams = false;
export function generateStaticParams() {
  return docPages.map((p) => ({ slug: p.slug }));
}
export async function generateMetadata({
  params,
}: {
  params: Promise<{ slug: string }>;
}) {
  const { slug } = await params;
  const doc = docPages.find((p) => p.slug === slug);
  return { title: doc?.title, description: doc?.description };
}
export default async function DocPage({
  params,
}: {
  params: Promise<{ slug: string }>;
}) {
  const { slug } = await params;
  const index = docPages.findIndex((p) => p.slug === slug);
  if (index < 0) notFound();
  const doc = docPages[index];
  const content =
    slug === "getting-started"
      ? (
          await readFile(
            path.join(process.cwd(), "src/content/getting-started.md"),
            "utf8",
          )
        )
          .replace("<!-- TUTORIAL -->", "```oath\n" + tutorial + "\n```")
          .replace(
            "<!-- REQUEST -->",
            "```json\n" + JSON.stringify(tutorialRequest, null, 2) + "\n```",
          )
      : docContent(slug);
  const toc = [...content.matchAll(/^## (.+)$/gm)].map((m) => ({
    title: m[1],
    id: slugify(m[1]),
  }));
  return (
    <main id="main" className="min-w-0 py-9 lg:pl-10">
      <div className="grid gap-10 xl:grid-cols-[minmax(0,1fr)_150px]">
        <article className="min-w-0 pb-16">
          <div className="mb-8">
            <p className="mb-4 text-xs text-muted-foreground">
              Documentation <span className="mx-2">/</span> {doc.group}
            </p>
            <div className="mb-4">
              <Badge variant="outline" className="font-mono text-[10px]">
                {release.version}
              </Badge>
            </div>
            <h1 className="text-3xl font-semibold tracking-tight sm:text-4xl">
              {doc.title}
            </h1>
            <p className="mt-4 text-base leading-7 text-muted-foreground">
              {doc.description}
            </p>
          </div>
          <Markdown content={content} />
          <div className="mt-12 grid gap-4 border-t pt-6 sm:grid-cols-2">
            {index > 0 ? (
              <Link
                className="rounded-lg border p-4 text-sm hover:bg-muted/50"
                href={`/docs/${docPages[index - 1].slug}`}
              >
                <span className="mb-2 flex items-center gap-2 text-xs text-muted-foreground">
                  <ArrowLeft className="size-3" />
                  Previous
                </span>
                {docPages[index - 1].title}
              </Link>
            ) : (
              <div />
            )}
            {index < docPages.length - 1 && (
              <Link
                className="rounded-lg border p-4 text-right text-sm hover:bg-muted/50"
                href={`/docs/${docPages[index + 1].slug}`}
              >
                <span className="mb-2 flex items-center justify-end gap-2 text-xs text-muted-foreground">
                  Next
                  <ArrowRight className="size-3" />
                </span>
                {docPages[index + 1].title}
              </Link>
            )}
          </div>
        </article>
        <aside className="hidden xl:block">
          <div className="sticky top-24">
            <p className="mb-4 text-xs font-medium">On this page</p>
            <nav aria-label="On this page" className="space-y-3">
              {toc.map((t) => (
                <a
                  key={t.id}
                  href={`#${t.id}`}
                  className="block text-xs leading-5 text-muted-foreground hover:text-foreground"
                >
                  {t.title}
                </a>
              ))}
            </nav>
            <div className="mt-8 border-t pt-5 text-[11px] leading-5 text-muted-foreground">
              Documentation for a pinned developer snapshot.
              <br />
              <span className="font-mono">
                {release.coreRevision.slice(0, 7)}
              </span>
            </div>
          </div>
        </aside>
      </div>
    </main>
  );
}
