// Pure renderer shared by the CLI and browser. It never invents legal logic.
export function renderDraft(selection, config) {
  const { unit, document, catalog } = selection;
  const { packageName, moduleName, stdPath, question } = config;
  const name = /^[A-Za-z_][A-Za-z0-9_]*(\.[A-Za-z_][A-Za-z0-9_]*)*$/;
  if (!name.test(packageName) || !name.test(moduleName)) throw new Error('Use dotted identifiers for package and module names');
  if (typeof stdPath !== 'string' || !stdPath || /[\r\n\0]/.test(stdPath)) throw new Error('Supply a standard-library path');
  if (typeof question !== 'string' || !question.trim() || question.length > 2000) throw new Error('Supply the question to formalize');
  const q = JSON.stringify, sourceId = `us.olrc.usc.${catalog.source.release}.${document.file}`;
  const query = `${moduleName}.assessment`;
  const request = { version: '0.1.0', query, arguments: {}, metadata: { authoring_status: 'Unimplemented', question } };
  const binding = {
    format: 'lawlib-encoding/1', id: packageName, package: '.', query,
    sources: [{ collection: 'usc', release: catalog.source.release, file: document.file, locator: unit.locator, key: unit.key, content_sha256: unit.content_sha256, role: 'support' }],
    depends_on: [], tests: ['tests/unimplemented.json'], questions: [question],
    unresolved: ['Applicability, definitions, exceptions, other authorities and interpretation have not been investigated'],
    status: { implementation: 'Unimplemented', source_integrity: 'Pinned', semantic_tests: 'UnreviewedExpectations', legal_review: 'Unreviewed' },
  };
  return {
    'oath.toml': `[package]\nname = ${q(packageName)}\nversion = "0.1.0"\nlanguage = "=0.1.0"\nentry = "src/main.oath"\n\n[dependencies.std]\npath = ${q(stdPath.replaceAll('\\', '/'))}\nversion = "=0.1.0"\n`,
    'src/main.oath': `module ${moduleName};\nuse std.provenance.{SourceRef, SourceAnchor, EncodingRecord, CoverageRecord, ReviewStatus};\n\nconst source = SourceRef {\n    stable_id: ${q(sourceId)}, citation: ${q(`${unit.number} ${unit.heading}`.trim())},\n    edition_or_version: ${q(`OLRC release ${catalog.source.release}`)},\n    locations: [${q(`cache:${document.sha256}`)}],\n    expected_sha256: ${q(document.sha256)},\n    publication_metadata: "Office of the Law Revision Counsel; original XML bytes",\n    retrieval_metadata: ${q(`XML member ${document.file} in ${catalog.source.url}; archive SHA-256 ${catalog.source.sha256}. Extract and audit original bytes separately.`)}\n};\nconst note = EncodingRecord {\n    id: ${q(packageName + '.draft')},\n    citations: [SourceAnchor { source_id: ${q(sourceId)}, locator: ${q(unit.locator ?? `${document.file}#element-${unit.element}`)} }],\n    rationale: ${q(`Unimplemented question: ${question}`)}, assumptions: [], review: ReviewStatus.Unreviewed\n};\nconst coverage = CoverageRecord {\n    id: ${q(packageName + '.coverage')}, questions: [${q(question)}],\n    investigated_sources: [${q(sourceId)}], exclusions: [],\n    not_investigated: ["Applicability, definitions, exceptions, other authorities, temporal reach and remedies"],\n    unresolved_interpretations: ["No legal interpretation has been encoded"], review: ReviewStatus.Unreviewed\n};\n\n// TODO: design typed facts and answer, scope/applicability, and cited rules.\n// No supporting rule is generated: this draft cannot produce a legal answer.\nquery assessment() -> Bool {\n    scope = true;\n    applies = true;\n    meta = note;\n}\n`,
    'encoding.json': JSON.stringify(binding, null, 2) + '\n',
    'scenarios/example.json': JSON.stringify(request, null, 2) + '\n',
    'tests/unimplemented.json': JSON.stringify({ name: 'unimplemented_draft_is_unresolved', request, expected: { kind: 'outcome', status: 'undetermined', stage: 'selection', issue_ids: [query] } }, null, 2) + '\n',
    'README.md': `# ${packageName}\n\nUnimplemented draft; legal review is **Unreviewed**.\n\nQuestion: ${question}\n\nSource: ${unit.number} ${unit.heading}; locator ${unit.locator ?? unit.key}; release ${catalog.source.release}.\nSelected extracted content SHA-256: ${unit.content_sha256}.\nOriginal XML SHA-256: ${document.sha256}.\n\nSet the std dependency path for your checkout, then run:\n\n\`\`\`sh\noath lock .\noath check .\noath test .\noath run . --request scenarios/example.json --certificate draft.cert.json\noath verify . --request scenarios/example.json --certificate draft.cert.json\n\`\`\`\n\nThe initial fixture verifies that the draft remains unresolved. It does not test a substantive legal rule. Write independent expected outcomes before adding legal logic. Record assumptions, exclusions and unresolved issues. Add explicit dependency links in encoding.json; registering a source is not proof of legal completeness.\n\nThe source reference hashes original XML, not extracted text or a ZIP. The archive URL and digest identify how to recover that member. Source auditing is separate from computation.\n`,
  };
}
