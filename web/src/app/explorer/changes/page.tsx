import Link from "next/link";
import report from "@/content/impact.json";
import { provisionHref } from "@/lib/provisions";
import { Badge } from "@/components/ui/badge";
export const metadata = { title: "Source changes and encoding impact" };
export default function Changes() {
  const identified = report.changes.filter(
    (c) => c.kind !== "IdentityReviewRequired",
  );
  const uncertain = report.changes.filter(
    (c) => c.kind === "IdentityReviewRequired",
  );
  return (
    <main id="main" className="min-w-0 py-9 pb-16 lg:pl-10">
      <p className="eyebrow">Pinned releases</p>
      <h1 className="mt-5 text-3xl font-semibold tracking-tight sm:text-4xl">
        Source changes
      </h1>
      <p className="mt-4 text-sm leading-7 text-muted-foreground">
        U.S. Code {report.before.release} → {report.after.release}. Text
        differences identify passages to reassess. They do not establish
        enactment, commencement, repeal, or a changed legal answer.
      </p>
      <div className="mt-5 flex flex-wrap gap-2">
        <Badge variant="outline">
          {identified.length} identified provision changes
        </Badge>
        <Badge variant="outline">{uncertain.length} identity reviews</Badge>
        <Badge variant="secondary">
          {report.affected.length} registered draft affected
        </Badge>
      </div>
      <section className="mt-8">
        <h2 className="text-xl font-semibold">Encodings to reassess</h2>
        {report.affected.map((e) => (
          <div key={e.id} className="mt-4 rounded-xl border p-5">
            <Link
              className="font-medium underline"
              href={"/explorer/" + e.package}
            >
              {e.id}
            </Link>
            <p className="mt-3 text-sm leading-6 text-muted-foreground">
              This unimplemented draft pins the earlier source. Its fixture
              verifies that it remains unresolved; it does not test substantive
              law. Review the change before writing rules or updating its source
              pin.
            </p>
            <p className="mt-3 break-all font-mono text-xs">{e.query}</p>
            <p className="mt-2 text-xs">
              Affected fixtures: {e.tests.join(", ")}
            </p>
          </div>
        ))}
        <p className="mt-4 text-sm leading-7 text-muted-foreground">
          The registered Rule 6 and W.D. Washington source passages are
          unchanged between these releases. This comparison does not check
          changes to local rules, Washington law, cases, or unregistered
          dependencies.
        </p>
      </section>
      <section className="mt-8">
        <h2 className="text-xl font-semibold">Changed passages</h2>
        <div className="mt-4 divide-y rounded-xl border">
          {identified.map((c, i) => (
            <div key={i} className="p-5">
              <Badge variant="outline">
                {c.kind === "AddedText" ? "Added text" : "Changed text"}
              </Badge>
              <h3 className="mt-3 text-sm font-medium">
                {c.after[0]?.number ?? c.before[0]?.number}{" "}
                {c.after[0]?.heading ?? c.before[0]?.heading}
              </h3>
              <p className="mt-2 break-all font-mono text-[11px] text-muted-foreground">
                {c.locator}
              </p>
              <p className="mt-2 text-xs text-muted-foreground">
                Changed components:{" "}
                {c.changed_parts.join(", ") || "new source unit"}
              </p>
              <div className="mt-4 flex gap-5 text-xs underline">
                {c.before[0] && (
                  <Link
                    href={provisionHref(
                      report.before.release,
                      c.file,
                      c.before[0].key,
                    )}
                  >
                    Earlier passage
                  </Link>
                )}
                {c.after[0] && (
                  <Link
                    href={provisionHref(
                      report.after.release,
                      c.file,
                      c.after[0].key,
                    )}
                  >
                    Later passage
                  </Link>
                )}
              </div>
            </div>
          ))}
        </div>
      </section>
      <details className="mt-6 rounded-xl border p-5">
        <summary className="cursor-pointer text-sm">
          Identity review ({uncertain.length} document groups)
        </summary>
        <p className="mt-4 text-sm leading-7 text-muted-foreground">
          These groups contain unlocated or ambiguous source material, including
          release notes. Positional keys are not durable legal identities. Group
          differences are retained without asserting which provision changed.
        </p>
        <ul className="mt-4 grid grid-cols-2 gap-2 text-xs sm:grid-cols-3">
          {uncertain.map((c, i) => (
            <li key={i}>
              <Link
                className="underline"
                href={provisionHref(report.after.release, c.file)}
              >
                {c.file}
              </Link>
            </li>
          ))}
        </ul>
      </details>
      <ul className="mt-6 list-disc space-y-2 pl-5 text-xs leading-6 text-muted-foreground">
        {report.limitations.map((l) => (
          <li key={l}>{l}</li>
        ))}
      </ul>
      <a
        className="mt-6 inline-block text-sm underline"
        href="/inventory/impact-119-118--119-121.json"
        download
      >
        Download complete impact report
      </a>
    </main>
  );
}
