import Link from "next/link";
import { ArrowLeft } from "lucide-react";
export default function NotFound() {
  return (
    <main
      id="main"
      className="page-shell flex min-h-[65vh] flex-col items-center justify-center text-center"
    >
      <p className="eyebrow">404 · Not found</p>
      <h1 className="mt-5 text-3xl font-semibold tracking-tight">
        This page is outside our scope.
      </h1>
      <p className="mt-4 text-sm text-muted-foreground">
        The page or package you requested does not exist in this snapshot.
      </p>
      <Link
        href="/explorer"
        className="mt-7 inline-flex items-center gap-2 text-sm font-medium"
      >
        <ArrowLeft className="size-4" />
        Back to lawlib explorer
      </Link>
    </main>
  );
}
