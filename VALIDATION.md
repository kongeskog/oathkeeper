# Web validation

## Constitutional foundation — 2026-10-19

- lawlib `a346c31`, tagged `v0.1.0-dev.4`: 137 constitutional provision units,
  37 authority/history relationships, 10 institutional actors or classes, and
  three new executable authority queries. The inventory reproduces exactly from
  the three pinned National Archives byte snapshots. The six source references
  in the constitutional import closure matched the external cache.
- All 191 library fixtures passed on Windows with independent certificate checks.
  All 69 constitutional fixtures also passed on Linux. The constitutional package
  adds 48 authority fixtures and four direct Amendment XX power fixtures.
- Five new inventory tests and 14 existing source-tooling tests passed. Checks
  reject changed text digests, altered source pins and dangling relationships,
  preserve historical text, and require every executable inventory link to have
  a real query and authored fixture.
- Six website content tests, TypeScript/build checks, formatting and ESLint checks
  passed. The website pins the committed lawlib revision and includes its new
  source archive. Original source documents remain external to the repositories.
- All 15 Playwright tests passed on the public production hostname, with no
  authentication or bypass headers. The new tests cover historical links,
  authority filters, URL restoration, query/scenario navigation, unknown IDs,
  mobile overflow and accessibility. All 13 documentation pages render.
- Desktop and mobile constitutional views were visually inspected. A selected-row
  contrast issue and a logo accessible-name mismatch were fixed without changing
  the shadcn theme. Synchronous native history updates prevent rapid filter
  interactions from restoring stale URL parameters.
- Constitutional records remain legally Unreviewed. Source representation is
  separate from executable coverage. Authority relationships do not create
  priorities or prove validity, and accepted procedural/interpretive findings
  remain explicit inputs. No complete constitutional doctrine is claimed.

Deployed application commit: `66fa7ef`. Production identity is in DEPLOYMENT.md.

## Provision workflow — 2026-10-19

- lawlib `136a7b6`, tagged `v0.1.0-dev.3`: eight packages, 26 modules,
  139 fixtures, and 17 matched source-audit bindings. All 139 fixtures passed on
  Windows with independent certificate verification. All 33 new court fixtures
  also passed on Linux; the existing Rust core is unchanged.
- The W.D. Washington Thanksgiving example returns November 30, 2026. Windows
  and Linux produced byte-identical certificates (SHA-256
  `4ac0ec1baeb634bbaa3efba1ee6ea6687e5103f5c292d30a94339340879b60bf`).
- Both pinned source releases reproduced all compressed index and text hashes:
  58 documents and 67,680 extracted source units each. A fresh preparation run
  from cached original archives exercised the standalone build-asset workflow.
- 14 source-tooling tests pass, including malformed XML, resource bounds, changed
  notes and parent headings, ambiguous identities, explicit dependency impact,
  altered shard hashes, and safe draft materialization.
- A draft downloaded through the browser was materialized, locked, compiled,
  tested, run, and independently verified. It correctly remained Undetermined
  with NoSupportingRule. No legal logic was generated.
- TypeScript, ESLint, Prettier, five content tests, and production build pass.
  Browser checks cover all 12 documentation pages, eight package pages, source
  text/notes, encoding links, draft download, historical passages, cross-reference
  filtering, altered source rejection, mobile layout and accessibility.
- Visually inspected the desktop provision/draft view. Mobile overflow and axe
  checks passed for the new explorer. Source text is a flattened rendition; the
  official archive remains the lossless source.
- The real comparison flags one unimplemented historical draft for reassessment.
  No source change is treated as an enactment, repeal, effective date, or legal
  approval. Comprehensive precedent and whole-system legal coverage remain open.

Production checks and deployment identity are recorded in DEPLOYMENT.md. Hosted
GitHub Actions has not been run; its asset preparation uses explicit pinned-source
downloads and cache keys derived from the committed catalogs.

## Inventory and deadline update — 2026-10-19

- Source snapshot: lawlib `9f6687c`, six packages, 23 modules, 28 queries,
  105 fixtures and 14 source-audit bindings. The Rust core is unchanged.
- Pinned U.S. Code inventory: 58 documents; per-document source integrity,
  executable coverage, semantic test and legal-review fields remain separate.
- Five content tests, nine Playwright tests, TypeScript, ESLint and production
  build pass. Ten documentation pages, six package pages and the new inventory
  page prerender successfully.
- New browser checks cover inventory search/empty states, document pins,
  downloaded manifest, deadline-package navigation and pending-review docs.
  Light/dark axe checks and mobile page-overflow checks pass.
- No legal-review status was upgraded. New packages and source indexing remain
  explicitly distinguished from whole-system formalization.

## Initial website

Validated locally on Windows with Node 24.19.0 and pnpm 11.25.0, 2026-10-19.

- Production Next.js build: passed; landing, eight documentation pages, explorer index, and five package pages prerendered.
- TypeScript: passed, including the production build's type analysis.
- ESLint: passed with no warnings in authored application/test code.
- Prettier: passed.
- Content tests: 4 passed. Checks include 20 downloadable module byte hashes and declaration locations, all 65 fixture query references, the 34-entry constitutional source map, and all 13 audit bindings.
- Playwright Chromium: 8 passed. Navigation, searches, filters, empty states, URL restoration, source declaration links, scenario selection, clipboard behavior, persisted dark mode, downloads, and missing-route status codes were exercised.
- axe scans: no detected WCAG 2 A/AA or 2.1 AA violations on the tested landing, guide/reference, explorer, package, citations, source, and scenario views across light/dark coverage. Automated scans do not establish complete accessibility conformance.
- Mobile: tested at 390 × 844, including navigation drawer, docs menu, and horizontal page overflow. Desktop screenshots use 1440 × 1000.
- Visual review: inspected desktop light/dark landing, explorer, IRA scenario view, and mobile source viewer screenshots. Reports/screenshots are generated under ignored test output directories.
- Production dependency audit: no known vulnerabilities reported. See README for the shadcn authoring-tool advisory, which is outside the shipped runtime.
- Both original Oathkeeper and lawlib repositories were left unchanged.

No hosted GitHub Actions run is claimed. Production deployment validation is recorded in [DEPLOYMENT.md](DEPLOYMENT.md).
