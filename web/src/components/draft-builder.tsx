"use client";
import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { renderDraft } from "@/lib/draft.mjs";
import type { Catalog, SourceDocument, Provision } from "@/lib/provisions";
export function DraftBuilder({
  catalog,
  document,
  unit,
}: {
  catalog: Catalog;
  document: SourceDocument;
  unit: Provision;
}) {
  const [name, setName] = useState("draft.my_question"),
    [question, setQuestion] = useState(""),
    [stdPath, setStdPath] = useState("../oathkeeper/stdlib"),
    [files, setFiles] = useState<Record<string, string> | null>(null),
    [error, setError] = useState("");
  function create() {
    try {
      setFiles(
        renderDraft(
          { catalog, document, unit },
          { packageName: name, moduleName: name, stdPath, question },
        ),
      );
      setError("");
    } catch (e) {
      setError(String(e));
      setFiles(null);
    }
  }
  function download() {
    if (!files) return;
    const url = URL.createObjectURL(
      new Blob(
        [JSON.stringify({ format: "lawlib-draft-bundle/1", files }, null, 2)],
        { type: "application/json" },
      ),
    );
    const a = window.document.createElement("a");
    a.href = url;
    a.download = "oath-draft.json";
    a.click();
    setTimeout(() => URL.revokeObjectURL(url), 1000);
  }
  return (
    <section
      aria-label="Create encoding draft"
      className="mt-8 rounded-xl border p-5"
    >
      <h3 className="font-semibold">Start an encoding</h3>
      <p className="mt-2 text-sm leading-6 text-muted-foreground">
        Pin this passage and write the question you want to formalize. The draft
        includes citations, coverage, and an unresolved scenario. You author the
        legal rules.
      </p>
      <div className="mt-5 grid gap-4 sm:grid-cols-2">
        <label className="text-xs">
          Package and module name
          <Input
            className="mt-2"
            value={name}
            onChange={(e) => {
              setName(e.target.value);
              setFiles(null);
            }}
          />
        </label>
        <label className="text-xs">
          Standard library path
          <Input
            className="mt-2"
            value={stdPath}
            onChange={(e) => {
              setStdPath(e.target.value);
              setFiles(null);
            }}
          />
        </label>
      </div>
      <label className="mt-4 block text-xs">
        Question to formalize
        <textarea
          className="mt-2 min-h-24 w-full rounded-md border bg-background p-3 text-sm"
          value={question}
          maxLength={2000}
          onChange={(e) => {
            setQuestion(e.target.value);
            setFiles(null);
          }}
          placeholder="Which facts, conditions, and exceptions determine this answer?"
        />
      </label>
      <Button className="mt-4" onClick={create}>
        Preview draft
      </Button>
      {error && (
        <p role="alert" className="mt-3 text-sm text-destructive">
          {error}
        </p>
      )}
      {files && (
        <div className="mt-5 border-t pt-5">
          <p className="text-sm">Unimplemented · Legal review: Unreviewed</p>
          <p className="mt-2 text-xs leading-6 text-muted-foreground">
            No supporting rule is generated. The initial test expects
            Undetermined. The source hash identifies original XML bytes, not
            this display text.
          </p>
          <details className="mt-4">
            <summary className="cursor-pointer text-sm">
              Preview src/main.oath
            </summary>
            <pre className="mt-3 max-h-96 overflow-auto rounded-md bg-muted p-4 text-xs">
              {files["src/main.oath"]}
            </pre>
          </details>
          <Button className="mt-4" variant="outline" onClick={download}>
            Download draft bundle
          </Button>
          <p className="mt-4 text-xs leading-6">
            From your lawlib checkout, write the bundle to a new directory:
          </p>
          <pre className="mt-2 overflow-auto rounded-md bg-muted p-3 text-xs">
            node tools/inventory/workflow.mjs materialize oath-draft.json
            ../my-draft
          </pre>
          <p className="mt-3 text-xs leading-6 text-muted-foreground">
            Inspect the files, set the standard-library path, then run oath
            lock, check, test, run, and verify. Nothing is uploaded or published
            by this form.
          </p>
        </div>
      )}
    </section>
  );
}
