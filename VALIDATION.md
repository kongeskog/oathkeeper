# Web validation

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
