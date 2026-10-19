"use client";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { useState } from "react";
import { Search, ChevronDown, ArrowUpRight } from "lucide-react";
import { docPages } from "@/lib/content";
import { Input } from "@/components/ui/input";
import { cn } from "@/lib/utils";
export function DocsNav() {
  const pathname = usePathname();
  const [search, setSearch] = useState("");
  const [open, setOpen] = useState(false);
  return (
    <aside className="border-b lg:sticky lg:top-16 lg:h-[calc(100vh-4rem)] lg:border-r lg:border-b-0">
      <button
        onClick={() => setOpen(!open)}
        aria-expanded={open}
        className="flex w-full items-center justify-between py-4 text-sm lg:hidden"
      >
        Documentation menu
        <ChevronDown className={cn("size-4", open && "rotate-180")} />
      </button>
      <div
        className={cn(
          "pb-8 lg:block lg:py-9 lg:pr-7",
          open ? "block" : "hidden",
        )}
      >
        <div className="relative mb-7">
          <Search className="absolute top-2.5 left-3 size-3.5 text-muted-foreground" />
          <Input
            className="pl-9"
            placeholder="Find a topic…"
            aria-label="Find a documentation topic"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
          />
        </div>
        <nav aria-label="Documentation">
          {["Introduction", "Language", "Tooling", "Lawlib"].map((group) => {
            const items = docPages.filter(
              (d) =>
                d.group === group &&
                `${d.title} ${d.description}`
                  .toLowerCase()
                  .includes(search.toLowerCase()),
            );
            return (
              items.length > 0 && (
                <div key={group} className="mb-6">
                  <p className="mb-2 px-3 text-xs font-medium">{group}</p>
                  {items.map((d) => (
                    <Link
                      key={d.slug}
                      href={`/docs/${d.slug}`}
                      onClick={() => setOpen(false)}
                      aria-current={
                        pathname === `/docs/${d.slug}` ? "page" : undefined
                      }
                      className={cn(
                        "mb-0.5 block rounded-md px-3 py-2 text-[13px] transition-colors",
                        pathname === `/docs/${d.slug}`
                          ? "bg-muted font-medium text-foreground"
                          : "text-muted-foreground hover:bg-muted/50 hover:text-foreground",
                      )}
                    >
                      {d.title}
                    </Link>
                  ))}
                </div>
              )
            );
          })}
          {!docPages.some((d) =>
            `${d.title} ${d.description}`
              .toLowerCase()
              .includes(search.toLowerCase()),
          ) && (
            <p className="px-3 text-xs text-muted-foreground">
              No matching topics. Try “rules” or “CLI”.
            </p>
          )}
        </nav>
        <Link
          href="/explorer"
          className="mx-3 mt-7 flex items-center justify-between border-t pt-5 text-xs text-muted-foreground"
        >
          Open lawlib explorer
          <ArrowUpRight className="size-3.5" />
        </Link>
      </div>
    </aside>
  );
}
