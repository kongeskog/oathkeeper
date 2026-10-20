"use client";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { useState } from "react";
import { ThemeProvider, useTheme } from "next-themes";
import {
  ArrowUpRight,
  BookOpen,
  Library,
  Menu,
  Moon,
  ShieldCheck,
  Sun,
} from "lucide-react";
import { Button, buttonVariants } from "@/components/ui/button";
import {
  Sheet,
  SheetTrigger,
  SheetContent,
  SheetHeader,
  SheetTitle,
  SheetDescription,
} from "@/components/ui/sheet";
import { cn } from "@/lib/utils";
export function Logo() {
  return (
    <span className="inline-flex items-center gap-2.5 font-semibold tracking-tight">
      <span className="flex size-7 items-center justify-center rounded-lg bg-foreground text-background">
        <ShieldCheck className="size-[18px]" strokeWidth={1.8} />
      </span>
      oathkeeper
      <span className="ml-1 hidden rounded border px-1.5 py-0.5 font-mono text-[10px] font-normal text-muted-foreground sm:inline">
        v0.1
      </span>
    </span>
  );
}
function ThemeToggle() {
  const { resolvedTheme, setTheme } = useTheme();
  return (
    <Button
      variant="ghost"
      size="icon"
      aria-label="Toggle color theme"
      onClick={() => setTheme(resolvedTheme === "dark" ? "light" : "dark")}
    >
      <Sun className="dark:hidden" />
      <Moon className="hidden dark:block" />
    </Button>
  );
}
export function SiteShell({ children }: { children: React.ReactNode }) {
  const pathname = usePathname();
  const [open, setOpen] = useState(false);
  const links = [
    { href: "/docs/getting-started", label: "Getting started" },
    { href: "/docs/language", label: "Documentation" },
    { href: "/explorer", label: "Lawlib explorer" },
  ];
  return (
    <ThemeProvider
      attribute="class"
      defaultTheme="light"
      enableSystem
      disableTransitionOnChange
    >
      <a
        href="#main"
        className="fixed left-4 top-2 z-[100] -translate-y-20 rounded-md bg-primary px-4 py-2 text-primary-foreground focus:translate-y-0"
      >
        Skip to content
      </a>
      <header className="sticky top-0 z-40 border-b bg-background/95 backdrop-blur-md">
        <div className="page-shell flex h-16 items-center justify-between gap-5">
          <Link href="/">
            <Logo />
            <span className="sr-only"> home</span>
          </Link>
          <nav
            aria-label="Main navigation"
            className="ml-auto hidden items-center gap-7 text-sm md:flex"
          >
            {links.map((l) => (
              <Link
                key={l.href}
                href={l.href}
                className={cn(
                  "transition-colors hover:text-foreground",
                  pathname === l.href ||
                    (l.href === "/explorer" &&
                      pathname.startsWith("/explorer/"))
                    ? "text-foreground"
                    : "text-muted-foreground",
                )}
              >
                {l.label}
              </Link>
            ))}
          </nav>
          <div className="flex items-center gap-2 md:ml-4">
            <ThemeToggle />
            <Sheet open={open} onOpenChange={setOpen}>
              <SheetTrigger
                render={
                  <Button
                    variant="ghost"
                    size="icon"
                    className="md:hidden"
                    aria-label="Open navigation"
                  />
                }
              >
                <Menu />
              </SheetTrigger>
              <SheetContent>
                <SheetHeader>
                  <SheetTitle>Oathkeeper</SheetTitle>
                  <SheetDescription>
                    Language, documentation, and lawlib.
                  </SheetDescription>
                </SheetHeader>
                <nav
                  aria-label="Mobile navigation"
                  className="flex flex-col gap-1 px-4"
                >
                  {links.map((l) => (
                    <Link
                      key={l.href}
                      href={l.href}
                      onClick={() => setOpen(false)}
                      className="rounded-md p-3 hover:bg-muted"
                    >
                      {l.label}
                    </Link>
                  ))}
                </nav>
              </SheetContent>
            </Sheet>
          </div>
        </div>
      </header>
      {children}
      <footer className="border-t">
        <div className="page-shell flex flex-col gap-8 py-10 sm:flex-row sm:items-center sm:justify-between">
          <div>
            <Link href="/">
              <Logo />
              <span className="sr-only"> home</span>
            </Link>
            <p className="mt-3 text-xs text-muted-foreground">
              A language for explicit rules and verifiable reasoning.
            </p>
          </div>
          <div className="flex flex-wrap gap-6 text-xs text-muted-foreground">
            <Link className="hover:text-foreground" href="/docs/verification">
              Trust & verification
            </Link>
            <Link className="hover:text-foreground" href="/docs/coverage">
              Coverage
            </Link>
            <a
              className="inline-flex items-center gap-1 hover:text-foreground"
              href="/downloads/oathkeeper-v0.1.0-dev.1.zip"
              download
            >
              Source code <ArrowUpRight className="size-3" />
            </a>
          </div>
        </div>
      </footer>
    </ThemeProvider>
  );
}
export function NextSteps() {
  return (
    <div className="grid gap-4 sm:grid-cols-2">
      <Link
        href="/docs/getting-started"
        className="group rounded-xl border p-6 transition-colors hover:bg-muted/50"
      >
        <BookOpen className="mb-5 size-5" />
        <p className="flex items-center justify-between font-medium">
          Write your first policy <ArrowUpRight className="size-4" />
        </p>
        <p className="mt-2 text-sm text-muted-foreground">
          Build, run, and independently verify an answer.
        </p>
      </Link>
      <Link
        href="/explorer"
        className="group rounded-xl border p-6 transition-colors hover:bg-muted/50"
      >
        <Library className="mb-5 size-5" />
        <p className="flex items-center justify-between font-medium">
          Read the lawlib <ArrowUpRight className="size-4" />
        </p>
        <p className="mt-2 text-sm text-muted-foreground">
          Follow a question from its rules to its sources.
        </p>
      </Link>
    </div>
  );
}
export function ActionLink({
  href,
  children,
  secondary = false,
}: {
  href: string;
  children: React.ReactNode;
  secondary?: boolean;
}) {
  return (
    <Link
      href={href}
      className={cn(
        buttonVariants({
          variant: secondary ? "outline" : "default",
          size: "lg",
        }),
        "h-10 gap-2 px-4",
      )}
    >
      {children}
    </Link>
  );
}
