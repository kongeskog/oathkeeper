"use client";
import { useState } from "react";
import { Check, Copy, FileCode2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
export function CopyButton({
  text,
  label = "Copy code",
}: {
  text: string;
  label?: string;
}) {
  const [status, setStatus] = useState<"idle" | "copied" | "failed">("idle");
  async function copy() {
    try {
      await navigator.clipboard.writeText(text);
      setStatus("copied");
    } catch {
      setStatus("failed");
    }
    setTimeout(() => setStatus("idle"), 2200);
  }
  return (
    <Button
      variant="ghost"
      size="icon-sm"
      onClick={copy}
      aria-label={status === "copied" ? "Copied" : label}
      title={
        status === "failed"
          ? "Clipboard unavailable. Select and copy the text."
          : label
      }
    >
      {status === "copied" ? <Check /> : <Copy />}
      <span role="status" className="sr-only">
        {status === "copied"
          ? "Copied to clipboard"
          : status === "failed"
            ? "Clipboard unavailable. Select and copy the text."
            : ""}
      </span>
    </Button>
  );
}
function tokens(line: string) {
  return line
    .split(
      /("(?:[^"\\]|\\.)*"|\/\/.*$|\b(?:module|use|type|enum|const|fn|query|rule|priority|scope|applies|when|yield|meta|higher|lower|let|if|else|match|and|or|not|true|false)\b)/g,
    )
    .map((t, i) => (
      <span
        key={i}
        className={
          t.startsWith("//")
            ? "code-comment"
            : t.startsWith('"')
              ? "code-string"
              : /^(module|use|type|enum|const|fn|query|rule|priority|scope|applies|when|yield|meta|higher|lower|let|if|else|match|and|or|not|true|false)$/.test(
                    t,
                  )
                ? "code-keyword"
                : undefined
        }
      >
        {t}
      </span>
    ));
}
export function CodeBlock({
  code,
  title,
  language = "oath",
  numbers = false,
  highlight,
  className,
  maxHeight,
}: {
  code: string;
  title?: string;
  language?: string;
  numbers?: boolean;
  highlight?: number;
  className?: string;
  maxHeight?: string;
}) {
  return (
    <div
      className={cn(
        "min-w-0 overflow-hidden rounded-xl border bg-muted/30 text-foreground",
        className,
      )}
    >
      <div className="flex h-11 items-center justify-between gap-3 border-b bg-muted/40 px-4">
        <span className="flex min-w-0 items-center gap-2 truncate font-mono text-xs text-muted-foreground">
          <FileCode2 className="size-3.5 shrink-0" />
          {title ?? language}
        </span>
        <CopyButton text={code} />
      </div>
      <pre
        tabIndex={0}
        aria-label={title ?? `${language} code`}
        className="code-scroll overflow-auto py-5 text-[12px] leading-relaxed sm:text-[13px]"
        style={{ maxHeight }}
      >
        <code>
          {code
            .trimEnd()
            .split("\n")
            .map((line, i) => (
              <span
                key={i}
                id={numbers ? `L${i + 1}` : undefined}
                className="code-line px-5"
                data-highlight={highlight === i + 1}
              >
                {numbers && (
                  <span
                    aria-hidden="true"
                    className="mr-5 inline-block w-7 select-none text-right text-muted-foreground"
                  >
                    {i + 1}
                  </span>
                )}
                {language === "oath" ? tokens(line) : line}
                {"\n"}
              </span>
            ))}
        </code>
      </pre>
    </div>
  );
}
