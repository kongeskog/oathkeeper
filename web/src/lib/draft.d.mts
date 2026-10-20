import type { Catalog, SourceDocument, Provision } from "./provisions";
export function renderDraft(
  selection: { catalog: Catalog; document: SourceDocument; unit: Provision },
  config: {
    packageName: string;
    moduleName: string;
    stdPath: string;
    question: string;
  },
): Record<string, string>;
