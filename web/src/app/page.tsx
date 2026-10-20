import Link from "next/link";
import {
  ArrowRight,
  ArrowUpRight,
  Check,
  CircleHelp,
  FileCode2,
  Fingerprint,
  GitBranch,
  Scale,
  ShieldCheck,
  Terminal,
  Braces,
} from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { ActionLink, NextSteps } from "@/components/site-shell";
import { CodeBlock } from "@/components/code-block";
import { counts, packages, packageHref } from "@/lib/content";
const sample = `query noise_status(emergency: Bool) -> NoiseStatus {
    scope = true;
    applies = true;
}

rule ordinary for noise_status(emergency) {
    when = true;
    yield = NoiseStatus.Prohibited;
}

rule exception for noise_status(emergency) {
    when = emergency;
    yield = NoiseStatus.Permitted;
}

priority emergency_first for noise_status(emergency) {
    higher = exception;
    lower = ordinary;
    when = true;
}`;
export default function Home() {
  return (
    <main id="main">
      <section className="page-shell grid gap-12 py-16 md:py-24 lg:grid-cols-[1fr_1.07fr] lg:items-center lg:gap-16">
        <div>
          <Link
            href="/docs/getting-started"
            className="mb-8 inline-flex items-center gap-2 rounded-full border bg-muted/40 px-3 py-1.5 text-xs"
          >
            <span className="size-1.5 rounded-full bg-foreground" />
            Introducing the v0.1 developer release
            <ArrowRight className="ml-2 size-3" />
          </Link>
          <h1 className="max-w-xl text-5xl leading-[1.08] font-semibold tracking-[-.055em] sm:text-6xl lg:text-[70px]">
            Law, made
            <br />
            explicit.
          </h1>
          <p className="mt-6 max-w-md text-lg leading-relaxed text-muted-foreground">
            A programming language for legal rules you can read, run, and
            verify.
          </p>
          <p className="mt-4 max-w-md text-sm leading-7 text-muted-foreground">
            Encode the rule. Name the exception. Keep the uncertainty. Every
            answer stays connected to its program and premises.
          </p>
          <div className="mt-8 flex flex-wrap gap-3">
            <ActionLink href="/docs/getting-started">
              Get started <ArrowRight className="size-4" />
            </ActionLink>
            <ActionLink href="/explorer" secondary>
              Explore lawlib <Braces className="size-4" />
            </ActionLink>
          </div>
          <div className="mt-8 flex flex-wrap gap-4 font-mono text-[11px] text-muted-foreground">
            <span>Rust core</span>
            <span aria-hidden="true">/</span>
            <span>Functional</span>
            <span aria-hidden="true">/</span>
            <span>Exact arithmetic</span>
          </div>
        </div>
        <div className="min-w-0">
          <div className="overflow-hidden rounded-xl border shadow-sm">
            <CodeBlock
              code={sample}
              title="quiet_hours.oath · illustrative excerpt"
              className="rounded-none border-0"
            />
            <div className="flex items-center justify-between gap-4 border-t bg-background px-5 py-4">
              <div className="flex items-center gap-2 text-xs">
                <Check className="size-4" />
                <span className="font-mono">emergency = true</span>
                <ArrowRight className="size-3 text-muted-foreground" />
                <span className="font-medium">Permitted</span>
              </div>
              <Badge variant="outline" className="hidden sm:inline-flex">
                Example result
              </Badge>
            </div>
          </div>
          <p className="mt-3 text-right text-[11px] text-muted-foreground">
            Fictional policy.{" "}
            <Link
              href="/docs/getting-started#read-the-policy"
              className="underline underline-offset-4"
            >
              View the complete runnable example ↗
            </Link>
          </p>
        </div>
      </section>
      <section className="border-y bg-muted/20">
        <div className="page-shell grid grid-cols-2 divide-x divide-y md:grid-cols-4 md:divide-y-0">
          {[
            ["01", "Write", "Define questions, rules, and priorities."],
            ["02", "Supply", "Make facts and unknowns explicit."],
            ["03", "Run", "Get an answer and its derivation."],
            ["04", "Verify", "Check that derivation independently."],
          ].map(([n, t, d]) => (
            <div key={n} className="p-6 sm:p-7">
              <span className="font-mono text-xs text-muted-foreground">
                {n} /
              </span>
              <h2 className="mt-3 font-medium">{t}</h2>
              <p className="mt-2 text-xs leading-6 text-muted-foreground">
                {d}
              </p>
            </div>
          ))}
        </div>
      </section>
      <section className="page-shell py-20 md:py-24">
        <div className="grid gap-8 md:grid-cols-[1fr_1fr] md:gap-24">
          <div>
            <p className="eyebrow">Built for the difficult parts</p>
            <h2 className="mt-4 text-3xl font-semibold tracking-tight sm:text-4xl">
              An answer is only as useful
              <br className="hidden lg:block" /> as its explanation.
            </h2>
            <p className="mt-5 max-w-md text-sm leading-7 text-muted-foreground">
              Law has exceptions, changing authorities, incomplete facts, and
              human judgment. Those belong in the model.
            </p>
            <Link
              href="/docs/language"
              className="mt-6 inline-flex items-center gap-2 text-sm font-medium"
            >
              Read the language reference <ArrowRight className="size-4" />
            </Link>
          </div>
          <div className="divide-y">
            {[
              {
                Icon: GitBranch,
                title: "Exceptions are explicit.",
                text: "Rules contribute answers. Authored priorities resolve competition without relying on source order.",
              },
              {
                Icon: CircleHelp,
                title: "Unknown is a first-class answer.",
                text: "Missing facts and disputed claims stay visible. The engine never invents a negative fact.",
              },
              {
                Icon: Fingerprint,
                title: "Results have a trail.",
                text: "A certificate binds the program, scenario, limits, and derivation for an independent checker.",
              },
            ].map(({ Icon, title, text }) => (
              <div key={title} className="flex gap-4 py-5 first:pt-0">
                <Icon className="mt-1 size-5 shrink-0" />
                <div>
                  <h3 className="text-sm font-medium">{title}</h3>
                  <p className="mt-2 text-sm leading-6 text-muted-foreground">
                    {text}
                  </p>
                </div>
              </div>
            ))}
          </div>
        </div>
      </section>
      <section className="border-y bg-muted/20 py-16 md:py-20">
        <div className="page-shell">
          <div className="flex flex-col justify-between gap-5 sm:flex-row sm:items-end">
            <div>
              <p className="eyebrow">The legal library</p>
              <h2 className="mt-4 text-3xl font-semibold tracking-tight">
                Small examples. A larger foundation.
              </h2>
              <p className="mt-3 max-w-xl text-sm leading-7 text-muted-foreground">
                Lawlib is a separate, versioned repository of legal models
                written in Oathkeeper. Inspect the source and the boundaries of
                each encoding.
              </p>
            </div>
            <Link
              href="/explorer"
              className="inline-flex shrink-0 items-center gap-2 text-sm font-medium"
            >
              Browse all {counts.packages} packages{" "}
              <ArrowUpRight className="size-4" />
            </Link>
          </div>
          <div className="mt-8 grid gap-4 md:grid-cols-3">
            {packages
              .filter((p) =>
                [
                  "private/household/demo",
                  "us/federal/constitution",
                  "us/federal/tax/ira/2025",
                ].includes(p.path),
              )
              .map((p, i) => {
                const Icon = [FileCode2, Scale, Terminal][i];
                return (
                  <Link
                    key={p.id}
                    href={packageHref(p)}
                    className="group flex flex-col rounded-xl border bg-background p-6 transition-colors hover:bg-muted/50"
                  >
                    <div className="flex items-center justify-between">
                      <Icon className="size-5" />
                      <Badge variant="outline">
                        {i === 0
                          ? "Fictional policy"
                          : i === 1
                            ? "Selected provisions"
                            : "Constrained model"}
                      </Badge>
                    </div>
                    <h3 className="mt-8 text-lg font-medium tracking-tight">
                      {p.title}
                    </h3>
                    <p className="mt-3 flex-1 text-sm leading-6 text-muted-foreground">
                      {p.description}
                    </p>
                    <div className="mt-7 flex items-center justify-between border-t pt-4 text-xs text-muted-foreground">
                      <span>
                        {p.modules.length} modules · {p.fixtures.length}{" "}
                        scenarios
                      </span>
                      <ArrowRight className="size-4 transition-transform group-hover:translate-x-1" />
                    </div>
                  </Link>
                );
              })}
          </div>
          <p className="mt-5 text-xs text-muted-foreground">
            Initial legal review: Unreviewed. Source mapping and executable
            coverage are reported separately.
          </p>
        </div>
      </section>
      <section className="page-shell grid gap-10 py-20 lg:grid-cols-[1fr_1.1fr] lg:gap-24">
        <div>
          <ShieldCheck className="mb-5 size-6" />
          <h2 className="text-2xl font-semibold tracking-tight">
            Three distinct assurances.
          </h2>
          <p className="mt-4 text-sm leading-7 text-muted-foreground">
            Verification makes the computation accountable. It does not
            establish that an encoding is legally correct or that every relevant
            authority has been found.
          </p>
          <Link
            href="/docs/verification"
            className="mt-5 inline-flex items-center gap-2 text-sm font-medium"
          >
            Understand the trust boundary <ArrowRight className="size-4" />
          </Link>
        </div>
        <div className="divide-y">
          {[
            [
              "Computational verification",
              "Does the answer follow from this program and these premises?",
            ],
            [
              "Source integrity",
              "Do the inspected document bytes match the expected hash?",
            ],
            [
              "Legal review",
              "Has a reviewer assessed the encoding and its interpretation?",
            ],
          ].map(([t, d], i) => (
            <div key={t} className="flex gap-5 py-5 first:pt-0">
              <span className="font-mono text-xs text-muted-foreground">
                0{i + 1}
              </span>
              <div>
                <h3 className="text-sm font-medium">{t}</h3>
                <p className="mt-2 text-sm text-muted-foreground">{d}</p>
              </div>
            </div>
          ))}
        </div>
      </section>
      <section className="page-shell pb-20">
        <NextSteps />
      </section>
    </main>
  );
}
