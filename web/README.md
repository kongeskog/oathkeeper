# Oathkeeper web application

React, Vite, TypeScript and neutral shadcn/ui components. The app includes lawlib exploration, source inspection, historical case proofs, live argument analysis and the bounded 2025 federal IRA calculator. Calculations run through the Rust WebAssembly kernel.

## Source and generated files

`src/` contains authored application code and presentation labels. `public/` contains authored workers. Generated lawlib JSON, copied documentation/code, source-document bytes, source archives and WASM all live under ignored `.generated/`. Vite serves `.generated/public/` and writes the deployment to ignored `dist/`. No PDFs or generated legal models are committed in the app source.

## Build from a clean software checkout

Use Rust, Python 3.10+, and Node.js 22+. From the software repository root:

```sh
python3 tools/lawlib_package.py sync
python -m venv .venv
. .venv/bin/activate
pip install 'maturin>=1.7,<2'
maturin develop --locked
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.105 --locked
python tools/build_web.py --fetch-sources
npm ci --prefix web
npm run dev --prefix web
```

The first command installs the external lawlib revision from `lawlib.lock.json` into ignored `.deps/lawlib/`. The software repository does not contain a second corpus copy. The app preparation command validates that immutable dependency, builds legal artifacts, fetches and checks the selected pilot evidence, refreshes documentation and compiles WASM. Subsequent preparation can omit `--fetch-sources` to require a fully offline evidence cache. Rust dependencies and compiler tools must also already be installed for a fully offline build.

The current app deliberately packages the two selected pilots, not every future lawlib module. Original evidence is stored once per digest in the shared cache; the deployable app receives copies of only its selected sources so browser integrity checks remain usable. Source downloads preserve the same pinned bytes.

After preparation, use `npm test --prefix web`, `npm run build --prefix web`, and `node web/scripts/verify-kernel.mjs`. Native/WASM parity checks replay the certificates. Re-run preparation after engine, lawlib or copied documentation changes. Normal React UI edits only need the Vite build.

## Vercel deployment

Production: <https://oathkeeper-sable.vercel.app/>.
Project: `rlindskog-projects/oathkeeper`.

Prepare the generated inputs using the clean-checkout instructions above, then run:

```sh
npm ci --prefix web
npm test --prefix web
node web/scripts/verify-kernel.mjs
npm run build:vercel --prefix web
cd web
vercel link --project oathkeeper --scope rlindskog-projects
vercel deploy --prebuilt --prod
```

The package contains only the production static files, including the Rust WASM
kernel and the selected pinned evidence. `.vercel/` is ignored by Git. Browser
calculations need no backend service or API keys. The app uses hash routes such
as `/#/workbench`, which work on static hosting without rewrites.

This workflow builds locally and uploads the complete output. A plain Vite build
from Git is insufficient until the Python/Rust preparation step has generated the
ignored inputs; automatic Git deployments are not configured by these commands.

## Managed Sites checkout

The existing app is <https://oathkeeper-field-guide.rainy-puppy-3202.chatgpt.site/>. Its managed source keeps `.openai/hosting.json` and a `.openai/build-inputs.json` recording the exact software commit. To reproduce it, check out that software revision, install the binding and tools above, then run:

```sh
python tools/build_web.py --web /path/to/managed-app --fetch-sources
npm ci --prefix /path/to/managed-app
npm run build --prefix /path/to/managed-app
```

The builder checks the managed checkout's recorded software revision. Generated inputs, `dist/`, dependency caches, Git metadata and hosting credentials stay out of Git source. The deployment archive contains `.openai/hosting.json` and `dist/` only.
