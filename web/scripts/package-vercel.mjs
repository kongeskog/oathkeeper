// Package the tested static app using Vercel's Build Output API.
import { access, cp, mkdir, rm, writeFile } from 'node:fs/promises'
import { fileURLToPath } from 'node:url'

const dist = fileURLToPath(new URL('../dist/', import.meta.url))
const output = fileURLToPath(new URL('../.vercel/output/', import.meta.url))

// Fail before replacing a previous package if the production build is missing.
for (const file of ['index.html', 'assets/kernel/oathkeeper_wasm_bg.wasm']) {
  await access(new URL(`../dist/${file}`, import.meta.url))
}
await rm(output, { recursive: true, force: true })
await mkdir(output, { recursive: true })
await cp(dist, `${output}/static`, { recursive: true })
// The application uses hash routes, so static hosting needs no catch-all rewrite.
await writeFile(`${output}/config.json`, JSON.stringify({ version: 3 }, null, 2) + '\n')
console.log('Vercel output ready in web/.vercel/output; deploy with vercel deploy --prebuilt --prod from web/.')
