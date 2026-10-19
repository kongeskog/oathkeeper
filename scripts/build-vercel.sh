#!/usr/bin/env bash
set -euo pipefail

# A Vercel Git deployment starts from a clean checkout, so create the ignored
# lawlib data and Rust/WASM browser kernel before Vite builds the static site.
if ! command -v rustup >/dev/null 2>&1; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
  export PATH="$HOME/.cargo/bin:$PATH"
fi

rustup toolchain install stable --profile minimal
rustup target add wasm32-unknown-unknown --toolchain stable

export OATHKEEPER_CACHE_DIR="$PWD/.vercel/cache/oathkeeper"
export CARGO_TARGET_DIR="$PWD/.vercel/cache/cargo-target"
WASM_BINDGEN_ROOT="$PWD/.vercel/cache/wasm-bindgen-0.2.105"
mkdir -p "$OATHKEEPER_CACHE_DIR" "$CARGO_TARGET_DIR" "$WASM_BINDGEN_ROOT"
export PATH="$WASM_BINDGEN_ROOT/bin:$PATH"

if ! command -v wasm-bindgen >/dev/null 2>&1 || [[ "$(wasm-bindgen --version)" != "wasm-bindgen 0.2.105" ]]; then
  cargo install wasm-bindgen-cli --version 0.2.105 --locked --root "$WASM_BINDGEN_ROOT"
fi

python3 tools/lawlib_package.py sync
python3 -m pip install .
cargo run --locked -- verify examples/library-access.case.oath
python3 tools/build_web.py --fetch-sources
npm test --prefix web
npm run build --prefix web
node web/scripts/verify-kernel.mjs
