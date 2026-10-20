"use client";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { ChevronRight, Folder, Library, FileCode2 } from "lucide-react";
import { packages, packageHref, release } from "@/lib/content";
import { cn } from "@/lib/utils";
export function ExplorerNav() {
  const pathname = usePathname();
  return (
    <aside className="border-b lg:sticky lg:top-16 lg:h-[calc(100vh-4rem)] lg:border-r lg:border-b-0">
      <div className="py-5 lg:py-9 lg:pr-6">
        <Link
          href="/explorer"
          className={cn(
            "flex items-center gap-2 rounded-md px-3 py-2 text-sm font-medium",
            pathname === "/explorer" && "bg-muted",
          )}
        >
          <Library className="size-4" />
          All packages
          <span className="ml-auto font-mono text-xs text-muted-foreground">
            {packages.length}
          </span>
        </Link>
        <Link
          href="/explorer/inventory"
          aria-current={pathname === "/explorer/inventory" ? "page" : undefined}
          className={cn(
            "mt-2 flex items-center gap-2 rounded-md px-3 py-2 text-sm",
            pathname === "/explorer/inventory" && "bg-muted",
          )}
        >
          <Library className="size-4" />
          Source inventory
        </Link>
        {[
          ["/explorer/constitution", "Constitution and authority"],
          ["/explorer/provisions", "Provision explorer"],
          ["/explorer/changes", "Source changes"],
        ].map(([href, label]) => (
          <Link
            key={href}
            href={href}
            aria-current={pathname === href ? "page" : undefined}
            className={cn(
              "mt-2 flex items-center gap-2 rounded-md px-3 py-2 text-sm",
              pathname === href && "bg-muted",
            )}
          >
            <FileCode2 className="size-4" />
            {label}
          </Link>
        ))}
        <details open className="mt-5 hidden lg:block">
          <summary className="flex cursor-pointer list-none items-center gap-2 px-3 text-xs text-muted-foreground">
            <ChevronRight className="size-3 rotate-90" />
            <Folder className="size-3.5" />
            lawlib
          </summary>
          <nav aria-label="Package tree" className="mt-3 ml-5 border-l pl-2">
            {packages.map((p) => (
              <Link
                key={p.id}
                href={packageHref(p)}
                aria-current={pathname === packageHref(p) ? "page" : undefined}
                className={cn(
                  "mb-1 block rounded-md px-3 py-2.5 text-xs",
                  pathname === packageHref(p)
                    ? "bg-muted text-foreground"
                    : "text-muted-foreground hover:bg-muted/50 hover:text-foreground",
                )}
              >
                <span className="flex items-center gap-2">
                  <FileCode2 className="size-3.5 shrink-0" />
                  {p.title}
                </span>
                <span
                  className="mt-1.5 block truncate pl-5 font-mono text-[10px]"
                  title={p.path}
                >
                  {p.path}
                </span>
              </Link>
            ))}
          </nav>
        </details>
        <div className="mt-10 hidden border-t px-3 pt-5 lg:block">
          <p className="eyebrow">Pinned snapshot</p>
          <p className="mt-3 font-mono text-[11px]">{release.version}</p>
          <p className="mt-2 font-mono text-[10px] text-muted-foreground">
            lawlib / {release.lawlibRevision.slice(0, 7)}
          </p>
          <p className="mt-4 text-xs leading-6 text-muted-foreground">
            Directories organize content. Explicit relationships determine
            authority.
          </p>
          <Link
            href="/docs/encoding"
            className="mt-3 inline-block text-xs underline underline-offset-4"
          >
            About legal models
          </Link>
        </div>
      </div>
    </aside>
  );
}
