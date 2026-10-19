# Production deployment

Deployed on 2026-10-19 to the existing Oathkeeper Vercel project, replacing its previous production app as requested.

- Production: https://oathkeeper-sable.vercel.app
- Team/project: `rlindskog-projects/oathkeeper`
- Project ID: `prj_UNSVsmxkhVWCUlKsBFnZWXpB9z6Y`
- Deployment ID: `dpl_8oAwZnVNFmJyrmt6U1RUUiDYEYud`
- Immutable deployment: https://oathkeeper-rm9oiusvz-rlindskog-projects.vercel.app
- Deployed application source: `8440bc7`; pinned lawlib source: `9f6687c`.
- Vercel status: `READY`, target `production`.
- Saved project settings: Next.js, Node 24.x, `pnpm build`, and `pnpm install --frozen-lockfile`.

Vercel rebuilt the application from the standalone website upload without sibling source repositories. The existing production hostname was reassigned successfully.

All nine Playwright tests passed against the public production hostname, without authentication or bypass headers. These checks covered the landing page, ten documentation pages, explorer navigation/search, source inventory and manifest, source and citation views, scenario expectations, downloads, 404 handling, mobile layouts, clipboard behavior, persisted themes, and automated accessibility scans. Production screenshots and reports were generated in the ignored test output directories.

The updated site includes the 58-document OLRC source inventory and the bounded
Rule 6(a) package. Imported text structure, executable coverage, tests and legal
review remain separate. The review packet is pending; no legal approval is claimed.

Future deployment from this checkout:

```sh
pnpm --config.node-linker=hoisted dlx vercel@64.5.0 deploy --prod --scope rlindskog-projects
```

The ignored `.vercel/` directory binds this checkout to the project. The CLI-created `.env.local` and deployment credentials are excluded from Git and deployment uploads. No runtime application secrets are required.
