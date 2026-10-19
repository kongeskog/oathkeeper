// Execute the shipped WebAssembly module and compare every field to native Rust.
import fs from 'node:fs/promises';
import assert from 'node:assert/strict';
import { initSync, analyze_json, verify_json, kernel_version, assess_backdoor_roth_json } from '../public/assets/kernel/oathkeeper_wasm.js';
initSync({ module: await fs.readFile(new URL('../public/assets/kernel/oathkeeper_wasm_bg.wasm', import.meta.url)) });
assert.equal(kernel_version(), '0.4.1');
const cases = JSON.parse(await fs.readFile(new URL('../src/data/workbench-cases.json', import.meta.url), 'utf8'));
for (const record of cases) {
  const result = JSON.parse(analyze_json(JSON.stringify(record.bundle), true));
  assert.deepEqual(result, record.baseline, `WASM/native analysis mismatch: ${record.slug}`);
  for (const argument of [...result.supporting, ...result.opposing]) {
    const bundle = {...record.bundle, proof:argument.proof, query:{...record.bundle.query, goal:argument.report.goal}};
    const report = JSON.parse(verify_json(JSON.stringify(bundle)));
    assert.deepEqual(report, argument.report, `Certificate replay mismatch: ${record.slug}`);
  }
}
assert.throws(()=>analyze_json('{"invalid":true}', true));
console.log(`WebAssembly/native parity and certificate replay passed for ${cases.length} real-record contexts.`);

const tax = JSON.parse(await fs.readFile(new URL('../src/data/tax.json', import.meta.url), 'utf8'));
for (const fixture of tax.fixtures) {
  const result = JSON.parse(assess_backdoor_roth_json(JSON.stringify(fixture.input)));
  assert.deepEqual(result, fixture.baseline, `Tax WASM/native mismatch: ${fixture.name}`);
  assert.deepEqual(JSON.parse(verify_json(JSON.stringify(result.certificate))), result.verification);
}
for (const record of tax.cases) {
  assert.deepEqual(JSON.parse(verify_json(JSON.stringify(record.bundle))), record.report, `Shank replay mismatch: ${record.name}`);
}
for (const field of ['prior_basis_cents', 'dec31_ira_value_cents', 'scope_confirmed']) {
  const result = JSON.parse(assess_backdoor_roth_json(JSON.stringify({...tax.fixtures[0].input, [field]:null})));
  assert.equal(result.status, 'needs_information'); assert.equal(result.certificate, null); assert.equal(result.allocation, null);
}
assert.equal(JSON.parse(assess_backdoor_roth_json(JSON.stringify({...tax.fixtures[0].input, conversion_year:2026}))).status, 'outside_scope');
assert.throws(()=>assess_backdoor_roth_json(JSON.stringify({...tax.fixtures[0].input, prior_basis_cents:0.5})));
console.log('Tax WASM/native parity, both official-case certificates, unknown inputs and year boundary passed.');
