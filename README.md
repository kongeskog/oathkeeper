# Oathkeeper web

Live: [oathkeeper-sable.vercel.app](https://oathkeeper-sable.vercel.app)

The Oathkeeper landing page, getting-started guide, language documentation, and lawlib explorer. Built with Next.js App Router, TypeScript, Tailwind, and the unmodified neutral theme from shadcn's default `base-nova` preset. Light and dark modes use the supplied theme tokens.

## Develop

Use Node 24 and pnpm 11.25.0.

```sh
pnpm install --frozen-lockfile
pnpm dev
```

Open http://localhost:3000. For a production preview, run `pnpm build` followed by `pnpm start`.

## Content and scope

- `/`: an introduction to questions, rules, uncertainty, and verification.
- `/docs/getting-started`: downloadable source snapshots and a complete run/verify walkthrough.
- `/docs/language`: the implemented language reference, with seven companion documentation pages.
- `/explorer`: package search and category filters.
- `/explorer/[package path]`: coverage records, executable query inventory, `.oath` source with declaration navigation, cited documents and byte hashes, and authored scenario fixtures.

Explorer tabs, selected files, declaration lines, scenarios, and package filters have shareable URLs. Scenario panels show repository-authored expected results; they do not run a second JavaScript implementation of the legal engine.

The content snapshot contains five packages, 20 modules, 27 queries, 65 fixtures, 13 source references, and a source map for seven constitutional Articles and 27 Amendments. Source mapping does not imply executable coverage. All initial legal models remain **Unreviewed**. The recorded source-byte audit is dated 2026-10-18; the website does not fetch or audit original documents at runtime.

## Refresh from the source repositories

The website is independently buildable. Its generated snapshot, raw `.oath` downloads, and ZIP source archives are committed so Vercel does not need the sibling Rust or lawlib checkouts.

For a content refresh, place committed checkouts alongside this repository:

```text
workspace/
  oathkeeper/
  lawlib/
  oathkeeper-web/
```

Then run:

```sh
pnpm content:sync
pnpm test:content
pnpm build
```

`scripts/sync-content.mjs` reads the five package manifests and their actual modules and fixtures, copies the reference documents, and records both Git revisions. It uses `git archive` for source downloads. It refuses tracked uncommitted changes so archive bytes and indexed content refer to the same revisions. Its declaration/source record index is for browsing, not a replacement compiler. If the source grammar or legal metadata schemas change, update the importer and its assertions together.

The getting-started narrative is in `src/content/getting-started.md`; its code and request are inserted from the runnable tutorial snapshot. Primary reference text comes from the Rust and lawlib repositories. No public repository URL or hosted CI result is invented.

## Checks

```sh
pnpm lint
pnpm typecheck
pnpm test:content
pnpm build
pnpm exec playwright install chromium
pnpm test:e2e
```

The content tests check file hashes, declarations, fixture query references, source-audit bindings, and the coverage inventory. Browser tests exercise navigation, search, filters, empty states, deep links, source downloads, clipboard behavior, theme persistence, the IRA fixtures, mobile layouts, and axe accessibility scans in both themes. Screenshots and reports are generated under ignored `test-results/` and `playwright-report/` directories. Use `TEST_BASE_URL` to run browser tests against an existing deployment.

GitHub Actions is configured for these checks; hosted runs require connecting a remote repository. shadcn is an authoring/build dependency, not a runtime CLI. Its tooling currently inherits a `braces` denial-of-service advisory with no patched version reported; the static app does not process user-supplied glob patterns. The production dependency audit reports no known advisories at initial validation.

## Vercel

`vercel.json` selects Next.js and frozen-lockfile installs. No application secrets, database, or external storage are required. This app takes over the existing `rlindskog-projects/oathkeeper` project:

```sh
pnpm --config.node-linker=hoisted dlx vercel@64.5.0 link --yes --project oathkeeper --scope rlindskog-projects
pnpm --config.node-linker=hoisted dlx vercel@64.5.0 deploy --prod
```

The local `.vercel/` link is ignored. CLI hoisting accommodates the tool's transitive dependency resolution with this pnpm version; it does not change the app's dependency layout.

## License

MIT. Source snapshot downloads include the original repositories' MIT licenses.
