# Application

The existing Oathkeeper application uses React and neutral shadcn/ui components. The lawlib, sources, documentation, proof walkthroughs, review workflow, and live argument workbench are separate pages.

The application source lives in `web/`. See [the application README](../web/README.md) for local development and asset regeneration. `web/scripts/sync-lawlib.py` compiles canonical source with the native Python binding, verifies historical proofs, copies review/source artifacts, and packages a core source ZIP. WebAssembly bindings are generated from `oathkeeper-wasm`, which calls the same Rust core. A browser worker runs analysis locally with a timeout; stale results are discarded when inputs change.

The workbench reports conditional support, opposition, conflict, or unresolved evidence. It exposes search bounds, assumptions, oracle state, and sensitivity. Historical fixture walkthroughs remain labeled as dependency visualizations. The retired scalar score is absent from current reports and application pages.
