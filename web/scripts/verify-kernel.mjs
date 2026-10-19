// Execute the shipped WebAssembly module and compare every field to native Rust.
import fs from 'node:fs/promises';
import assert from 'node:assert/strict';
import { initSync, analyze_json, verify_json, kernel_version, assess_backdoor_roth_json } from '../.generated/public/assets/kernel/oathkeeper_wasm.js';
initSync({ module: await fs.readFile(new URL('../.generated/public/assets/kernel/oathkeeper_wasm_bg.wasm', import.meta.url)) });
assert.equal(kernel_version(), '0.5.0');
const cases = JSON.parse(await fs.readFile(new URL('../.generated/data/workbench-cases.json', import.meta.url), 'utf8'));
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

const tax = JSON.parse(await fs.readFile(new URL('../.generated/data/tax.json', import.meta.url), 'utf8'));
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

// Shared policy API: actual WASM scope certificates and host-pinned rewards.
const { assess_policy_json } = await import('../.generated/public/assets/kernel/oathkeeper_wasm.js');
const policy = request => JSON.parse(assess_policy_json(JSON.stringify(request)));
const historical = cases.find(c => c.slug === 'long-ability-to-pay');
const analysis = JSON.parse(analyze_json(JSON.stringify(historical.bundle), false));
const argument = analysis.supporting[0];
const context = {...historical.bundle, proof:argument.proof, query:{...historical.bundle.query, root:argument.proof.at(-1).id}};
const coverage = policy({operation:'coverage',bundle:context});
assert.equal(coverage.status, 'conditions_supported');
const scope_proofs = coverage.conditions.map(c=>({id:c.id,root:c.supporting[0].proof.at(-1).id,proof:c.supporting[0].proof}));
const task = policy({operation:'freeze',bundle:context,policy:{allow_oracles:true,require_coverage:true}});
assert.equal(task.context.proof.length, 0);
assert.equal(policy({operation:'evaluate',task,expected_sha256:task.task_sha256,candidate:{proof:argument.proof,scope_proofs}}).reward, 1);
assert.equal(policy({operation:'evaluate',task,expected_sha256:task.task_sha256,candidate:context}).reward, 0);
assert.equal(policy({operation:'evaluate',task,expected_sha256:task.task_sha256,candidate:argument.proof}).reward, 0);
assert.throws(()=>policy({operation:'evaluate',task:{...task,policy:{...task.policy,allow_oracles:false}},expected_sha256:task.task_sha256,candidate:[]}));

// Node signs independently; the Rust WASM verifier checks Ed25519, exact artifact and chain.
const crypto = await import('node:crypto');
const privateKey = crypto.createPrivateKey({key:Buffer.concat([Buffer.from('302e020100300506032b657004220420','hex'),Buffer.alloc(32,7)]),format:'der',type:'pkcs8'});
const publicKey = crypto.createPublicKey(privateKey).export({format:'der',type:'spki'}).subarray(-32).toString('hex');
const trust = {keys:[{id:'synthetic.key',principal:'synthetic.reviewer',public_key:publicKey,valid_from:'2020-01-01T00:00:00Z',valid_until:null,revoked_at:null}]};
const receipt = {id:'synthetic.review',sequence:0,previous_sha256:null,signer:'synthetic.key',recorded_at:'2026-10-18T00:00:00Z',payload:{kind:'review',artifact_sha256:'a'.repeat(64),aspect:'formalization',decision:'approved',independence_declared:true,rationale:'Synthetic interoperability test only.'},signature:''};
const bytes = policy({operation:'signing_payload',receipt}).utf8;
receipt.signature = crypto.sign(null,Buffer.from(bytes),privateKey).toString('hex');
const journal = {format:'oathkeeper.journal/1',events:[receipt]};
assert.equal(policy({operation:'reviews',artifact_sha256:'a'.repeat(64),journal,trust,authors:[]}).current.length, 1);
assert.equal(policy({operation:'reviews',artifact_sha256:'b'.repeat(64),journal,trust,authors:[]}).current.length, 0);
assert.throws(()=>policy({operation:'journal',journal:{...journal,events:[{...receipt,recorded_at:'2026-10-19T00:00:00Z'}]},trust}));
assert.throws(()=>policy({operation:'journal',journal,trust:{keys:[]}}));
console.log('WASM scope certificates, frozen rewards and independently signed Ed25519 journals passed.');
