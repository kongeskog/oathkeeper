import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { Children, isValidElement, type ReactNode } from "react";
import { CodeBlock } from "@/components/code-block";
import { slugify } from "@/lib/content";
function textOf(node: ReactNode): string {
  return Children.toArray(node)
    .map((n) =>
      typeof n === "string" || typeof n === "number"
        ? String(n)
        : isValidElement<{ children?: ReactNode }>(n)
          ? textOf(n.props.children)
          : "",
    )
    .join("");
}
const rewrite = (href: string) =>
  href === "REVIEW.md"
    ? "/docs/deadline-review"
    : href.includes("schemas/v0.1")
      ? "/downloads/oathkeeper-v0.1.0-dev.1.zip"
      : href.endsWith("LANGUAGE.md")
        ? "/docs/language"
        : href.endsWith("FORMATS.md")
          ? "/docs/formats"
          : href.endsWith("TRUST.md")
            ? "/docs/verification"
            : href.endsWith("COVERAGE.md")
              ? "/docs/coverage"
              : href.endsWith("ENCODING.md")
                ? "/docs/encoding"
                : href;
export function Markdown({ content }: { content: string }) {
  return (
    <div className="prose-doc">
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        components={{
          h2: ({ children }) => (
            <h2 id={slugify(textOf(children))}>{children}</h2>
          ),
          h3: ({ children }) => (
            <h3 id={slugify(textOf(children))}>{children}</h3>
          ),
          pre: ({ children }) => {
            const child = Children.toArray(children)[0];
            const lang = isValidElement<{ className?: string }>(child)
              ? child.props.className?.replace("language-", "")
              : "text";
            return (
              <CodeBlock
                code={textOf(children).trimEnd()}
                language={lang ?? "text"}
              />
            );
          },
          table: ({ children }) => (
            <div className="overflow-x-auto rounded-lg border">
              <table>{children}</table>
            </div>
          ),
          a: ({ href, children }) => (
            <a
              href={rewrite(href ?? "")}
              {...(href?.startsWith("https://")
                ? { target: "_blank", rel: "noreferrer" }
                : {})}
            >
              {children}
            </a>
          ),
        }}
      >
        {content}
      </ReactMarkdown>
    </div>
  );
}
