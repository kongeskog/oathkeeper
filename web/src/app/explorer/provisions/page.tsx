import { Suspense } from "react";
import Link from "next/link";
import { ProvisionBrowser } from "@/components/provision-browser";
export const metadata = {
  title: "Provision explorer",
  description:
    "Read pinned source passages, inspect notes and references, and create an Oathkeeper encoding draft.",
};
export default function Provisions() {
  return (
    <main id="main" className="min-w-0 py-9 pb-16 lg:pl-10">
      <p className="eyebrow">From source to program</p>
      <h1 className="mt-5 text-3xl font-semibold tracking-tight sm:text-4xl">
        Provision explorer
      </h1>
      <p className="mt-4 max-w-2xl text-sm leading-7 text-muted-foreground">
        Browse extracted text from two pinned U.S. Code releases. Source units
        include sections, rules, and standalone notes. They are not a count of
        operative laws or completed encodings.
      </p>
      <div className="mt-4 flex gap-5 text-xs underline underline-offset-4">
        <Link href="/explorer/changes">Compare releases</Link>
        <Link href="/docs/source-workflow">Authoring workflow</Link>
      </div>
      <Suspense fallback={<p className="mt-6 text-sm">Loading explorer…</p>}>
        <ProvisionBrowser />
      </Suspense>
    </main>
  );
}
