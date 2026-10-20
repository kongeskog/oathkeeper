# Production deployment

## Constitutional foundation — 2026-10-19

- Production: https://oathkeeper-sable.vercel.app
- Explorer: https://oathkeeper-sable.vercel.app/explorer/constitution
- Guide: https://oathkeeper-sable.vercel.app/docs/constitutional-authority
- Team/project: rlindskog-projects/oathkeeper
- Project ID: prj_UNSVsmxkhVWCUlKsBFnZWXpB9z6Y
- Deployment ID: dpl_9gY9QxndzF5BmTcQKZfxURje1hdy
- Immutable deployment: https://oathkeeper-dy7ctxj9c-rlindskog-projects.vercel.app
- Deployed application source: 66fa7ef; lawlib: a346c31 (v0.1.0-dev.4).
- Status: READY; target: production.

Vercel built the standalone upload and aliased the existing production hostname.
All 15 Playwright tests passed against that public hostname without authentication
or bypass headers. The deployed snapshot contains eight packages, 30 modules,
191 fixtures, 137 constitutional provision units and 37 authority/history links.
All legal review remains Unreviewed. Documentation-only commits following the
application commit record validation and deployment; they do not change the app.

## Previous deployment: provision workflow

Updated 2026-10-19 in the existing Oathkeeper project.

- Production: https://oathkeeper-sable.vercel.app
- Team/project: rlindskog-projects/oathkeeper
- Project ID: prj_UNSVsmxkhVWCUlKsBFnZWXpB9z6Y
- Deployment ID: dpl_E6TeSTHRSYW9JtPRmRftpe2mXiKr
- Immutable deployment: https://oathkeeper-g3amz2mwe-rlindskog-projects.vercel.app
- Deployed application source: 5b72488; lawlib: 136a7b6 (v0.1.0-dev.3).
- Status: READY; target: production.

Vercel rebuilt the standalone website upload, including the hash-checked derived
source assets for both pinned releases. Original archives and generated shards
remain outside Git. The prepared upload was approximately 246 MB; subsequent
source-only deployments reused the assets.

All 12 Playwright tests passed against the public production hostname, without
authentication or bypass headers. Tests covered 12 documentation pages, package
views, the source inventory, provision text and notes, a draft download, release
comparison, reference navigation, source tampering rejection, mobile layout,
light/dark modes, accessibility, source downloads and missing routes. Production
desktop and mobile provision views were also inspected.

The browser-generated draft was independently compiled and verified locally as
Undetermined, as designed. Lawlib has 139 passing semantic fixtures; all successful
fixture runs are independently certificate-checked. The new court package passed
on Windows and Linux, with an identical certificate for the showcased scenario.
All legal encodings remain Unreviewed. No comprehensive legal coverage is claimed.

Future CLI deployments must prepare public/provisions from the pinned catalogs
as described in README.md before running Vercel deploy. The explicit .vercelignore
includes those generated assets while excluding credentials, .env files, Git,
node_modules, local build output and test reports. No runtime secrets are required.

The prior developer-release-2 deployment was dpl_8oAwZnVNFmJyrmt6U1RUUiDYEYud.
Previous source releases and downloadable archives remain available.
