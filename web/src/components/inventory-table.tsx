"use client";
import { useState } from "react";
import Link from "next/link";
import { provisionHref } from "@/lib/provisions";
import inventory from "@/content/inventory.json";
import { Input } from "@/components/ui/input";

export function InventoryTable() {
  const [search, setSearch] = useState("");
  const documents = inventory.documents.filter((d) =>
    [d.document_title, d.heading, d.file, d.identifier]
      .join(" ")
      .toLowerCase()
      .includes(search.trim().toLowerCase()),
  );
  return (
    <section className="mt-8" aria-labelledby="inventory-heading">
      <h2 id="inventory-heading" className="text-lg font-medium">
        Documents and status
      </h2>
      <label
        htmlFor="inventory-search"
        className="mt-4 block text-xs text-muted-foreground"
      >
        Search titles, filenames, or identifiers
      </label>
      <Input
        id="inventory-search"
        className="mt-2 max-w-md"
        value={search}
        onChange={(event) => setSearch(event.target.value)}
        placeholder="For example: Title 26 or judiciary"
      />
      <p aria-live="polite" className="mt-3 text-xs text-muted-foreground">
        {documents.length} of {inventory.documents.length} documents
      </p>
      <div className="mt-4 overflow-x-auto rounded-xl border">
        <table className="w-full min-w-[680px] text-left text-xs">
          <caption className="sr-only">
            Source integrity, formalization, semantic tests, and legal review
            are separate assessments.
          </caption>
          <thead className="border-b bg-muted/40">
            <tr>
              {[
                "Document",
                "Source bytes",
                "Executable coverage",
                "Semantic tests",
                "Legal review",
              ].map((label) => (
                <th key={label} scope="col" className="px-4 py-3 font-medium">
                  {label}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {documents.map((d) => (
              <tr key={d.file} className="border-b last:border-0">
                <th scope="row" className="max-w-80 px-4 py-4 font-normal">
                  <p className="font-medium">{d.document_title}</p>
                  <Link
                    className="mt-2 inline-block underline underline-offset-4"
                    href={provisionHref("119-121", d.file)}
                  >
                    Browse provisions
                  </Link>
                  <p className="mt-1 leading-5 text-muted-foreground">
                    {d.heading}
                  </p>
                  <details className="mt-2 text-muted-foreground">
                    <summary className="cursor-pointer text-[11px]">
                      Identifiers and hashes
                    </summary>
                    <dl className="mt-2 space-y-2 text-[10px]">
                      <div>
                        <dt>Publisher locator</dt>
                        <dd className="break-all font-mono">{d.identifier}</dd>
                      </div>
                      <div>
                        <dt>XML file</dt>
                        <dd className="font-mono">{d.file}</dd>
                      </div>
                      <div>
                        <dt>Original XML SHA-256</dt>
                        <dd className="break-all font-mono">{d.sha256}</dd>
                      </div>
                      <div>
                        <dt>Derived index SHA-256</dt>
                        <dd className="break-all font-mono">
                          {d.index.sha256}
                        </dd>
                      </div>
                    </dl>
                  </details>
                </th>
                <td className="px-4 py-4 align-top">
                  {d.status.source_audit}
                  <span className="mt-1 block text-[10px] text-muted-foreground">
                    {inventory.source.retrieved_on}
                  </span>
                </td>
                <td className="px-4 py-4 align-top text-muted-foreground">
                  {d.status.executable_coverage === "NotAssessed"
                    ? "Not assessed"
                    : d.status.executable_coverage}
                </td>
                <td className="px-4 py-4 align-top text-muted-foreground">
                  {d.status.semantic_tests === "NotAssessed"
                    ? "Not assessed"
                    : d.status.semantic_tests}
                </td>
                <td className="px-4 py-4 align-top">{d.status.legal_review}</td>
              </tr>
            ))}
          </tbody>
        </table>
        {documents.length === 0 && (
          <p className="p-6 text-sm text-muted-foreground">
            No documents match this search.
          </p>
        )}
      </div>
      <p className="mt-3 text-xs leading-6 text-muted-foreground">
        “Not assessed” is an investigation status for each imported document,
        not a finding that no executable model exists. Existing packages retain
        their own source editions and scope.
      </p>
    </section>
  );
}
