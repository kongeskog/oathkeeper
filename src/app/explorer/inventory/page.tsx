import Link from "next/link";
import inventory from "@/content/inventory.json";
import { Badge } from "@/components/ui/badge";
import { InventoryTable } from "@/components/inventory-table";

export const metadata = {
  title: "U.S. Code source inventory",
  description:
    "A pinned federal source release with separate integrity, formalization, testing, and legal-review status.",
};
export default function SourceInventory() {
  return (
    <main id="main" className="min-w-0 py-9 pb-16 lg:pl-10">
      <p className="eyebrow">Federal source collection</p>
      <h1 className="mt-5 text-3xl font-semibold tracking-tight sm:text-4xl">
        U.S. Code source inventory
      </h1>
      <p className="mt-4 max-w-2xl text-sm leading-7 text-muted-foreground">
        One reproducible source release, ready for research and incremental
        encoding. These documents are indexed source material. Importing them
        creates no executable legal rules.
      </p>
      <div className="mt-5 flex flex-wrap gap-2">
        <Badge variant="outline">OLRC release {inventory.source.release}</Badge>
        <Badge variant="secondary">Legal review: Unreviewed</Badge>
      </div>
      <div className="mt-7 grid grid-cols-2 divide-x rounded-xl border bg-muted/20 py-4">
        <div className="px-5">
          <p className="font-mono text-xl">{inventory.counts.documents}</p>
          <p className="mt-1 text-xs text-muted-foreground">XML documents</p>
        </div>
        <div className="px-5">
          <p className="font-mono text-xl">
            {inventory.counts.by_kind.section.toLocaleString("en-US")}
          </p>
          <p className="mt-1 text-xs text-muted-foreground">Section elements</p>
        </div>
      </div>
      <p className="mt-4 text-xs leading-6 text-muted-foreground">
        Includes 53 numbered titles and five appendices; Title 53 is reserved.
        Element counts include repealed, omitted, and reserved material. They do
        not measure operative laws or executable coverage.
      </p>
      <div className="mt-6 rounded-xl border p-5 text-sm">
        <p>
          Current through {inventory.source.current_through}. Retrieved{" "}
          {inventory.source.retrieved_on}.
        </p>
        <p className="mt-2 leading-6 text-muted-foreground">
          The original archive and large derived indexes live outside Git. The
          manifest pins every document and index by hash. References and
          publisher note labels are preserved; legal force and applicability
          remain questions for encoding and review.
        </p>
        <div className="mt-4 flex flex-wrap gap-x-6 gap-y-3 text-xs underline underline-offset-4">
          <a href={inventory.source.url} target="_blank" rel="noreferrer">
            Official XML archive (104 MiB)
          </a>
          <a href="/inventory/usc-119-121.json" download>
            Download pinned manifest
          </a>
          <a href="/inventory/README.md" download>
            Reproduction guide
          </a>
        </div>
      </div>
      <InventoryTable />
      <div className="mt-8 border-t pt-6 text-sm leading-7 text-muted-foreground">
        <p>
          The Code is one federal collection. Regulations, session laws outside
          this collection, cases, orders, and state, tribal, territorial and
          local law need their own inventories.
        </p>
        <p className="mt-3">
          For a concrete executable example, explore the{" "}
          <Link
            className="text-foreground underline underline-offset-4"
            href="/explorer/us/federal/procedure/frcp/6"
          >
            Rule 6(a) deadline package
          </Link>{" "}
          and its{" "}
          <Link
            className="text-foreground underline underline-offset-4"
            href="/docs/deadline-review"
          >
            pending legal review
          </Link>
          .
        </p>
      </div>
    </main>
  );
}
