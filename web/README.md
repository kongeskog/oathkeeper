# Oathkeeper web application

React, Vite, TypeScript, and neutral shadcn/ui components. The application includes the law library, source inspection, historical case proofs, live argument workbench, and bounded 2025 federal IRA calculator. Calculations run locally through the Rust WebAssembly kernel.

## Run locally

From this directory, with Node.js 22 or newer:

```sh
npm ci
npm run dev
```

The checked-in application data and WASM assets support local use without installing Rust or Python. `npm run build` creates `dist/`; `npm test` runs the UI tests. `node scripts/verify-kernel.mjs` checks shipped WASM against the recorded native results and replays certificates.

## Refresh from the Rust core

Run these commands from the repository root after changing the engine or lawlib:

```sh
python -m venv .venv
. .venv/bin/activate
pip install 'maturin>=1.7,<2'
maturin develop --locked
python tools/audit_tax.py
python web/scripts/sync-lawlib.py
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.105 --locked
cargo build -p oathkeeper-wasm --target wasm32-unknown-unknown --release --locked
wasm-bindgen target/wasm32-unknown-unknown/release/oathkeeper_wasm.wasm --target web --out-dir web/public/assets/kernel --out-name oathkeeper_wasm
node web/scripts/verify-kernel.mjs
```

The sync script defaults to the parent repository; an explicit core path may also be passed. It compiles canonical `.oath` sources, verifies historical certificates, refreshes documentation and source snapshots, and creates the downloadable **core** source archive. The `web/` directory is excluded from that archive to avoid recursive packaging. The complete core and application are available in this GitHub repository.

## Hosting provenance

This source was imported from the managed Sites repository at commit `87c15b7a7023f4ecda5ce464832e97cbc9458023`. The existing deployment is <https://oathkeeper-field-guide.rainy-puppy-3202.chatgpt.site/>. Hosting credentials and project bindings stay in the managed Sites checkout; `dist/`, dependency caches, and Git metadata are excluded from this source copy.
