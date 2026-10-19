import { access } from 'node:fs/promises'
import { fileURLToPath } from 'node:url'

try {
  for (const file of ['data/module.json', 'data/lawlib-package.json', 'public/assets/kernel/oathkeeper_wasm_bg.wasm']) {
    await access(fileURLToPath(new URL(`../.generated/${file}`, import.meta.url)))
  }
} catch {
  console.error('App build inputs are missing. From the software repository, install the native binding and run: python tools/build_web.py --fetch-sources. See web/README.md for setup.')
  process.exit(1)
}
