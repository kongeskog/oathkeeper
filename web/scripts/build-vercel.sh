#!/usr/bin/env bash
set -euo pipefail

# Deployment preparation only; the legal runtime never fetches sources.
cache=".next/cache/oathkeeper"
mkdir -p "$cache"
unzip -o -q public/downloads/lawlib-v0.1.0-dev.4.zip -d "$cache"
# The extractor is a separate locked package inside the web build directory.
pnpm --dir "$cache/lawlib/tools/inventory" install --frozen-lockfile --ignore-workspace
node scripts/fetch-provisions.mjs \
  "$cache/lawlib/tools/inventory/extract-provisions.mjs" \
  "$cache/source-cache" --fetch
pnpm build
