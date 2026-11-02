// SPDX-License-Identifier: AGPL-3.0-only
import { test } from 'node:test'
import assert from 'node:assert/strict'
import { compareOutput } from './browser-wasm-contract.mjs'
const tls = () => { const b = Buffer.alloc(33); b[0] = 1; [0, 0, 1, 3].forEach((v, i) => b.writeDoubleLE(v, 1 + i * 8)); return b }
test('valid numerical and exact graph results', () => {
  assert.equal(compareOutput(1, tls(), tls()), 0)
  assert.equal(compareOutput(1, Buffer.from([0]), Buffer.from([0])), 0)
  assert.equal(compareOutput(2, Buffer.from([0, 1, 2]), Buffer.from([0, 1, 2])), 0)
})
for (const [name, change] of [
  ['changed value', b => b.writeDoubleLE(42, 1)],
  ['missing value', b => b.subarray(0, -8)],
  ['nonfinite', b => b.writeDoubleLE(NaN, 1)],
  ['changed presence', b => { b[0] = 0 }],
]) test('rejects ' + name, () => {
  const base = tls(), actual = Buffer.from(base), returned = change(actual)
  assert.throws(() => compareOutput(1, base, Buffer.isBuffer(returned) ? returned : actual))
})
test('rejects empty, malformed and unknown codecs', () => {
  assert.throws(() => compareOutput(1, Buffer.alloc(0), Buffer.alloc(0)))
  assert.throws(() => compareOutput(1, Buffer.from([0, 0]), Buffer.from([0, 0])))
  assert.throws(() => compareOutput(99, tls(), tls()))
})
test('changed discrete graph byte cannot pass', () => assert.throws(() => compareOutput(2, Buffer.from([0, 1]), Buffer.from([0, 2]))))
test('changed marked-plane zero sign/region cannot pass', () => {
  const b = Buffer.alloc(10 + 144 + 12 + 4 + 720); b.writeUInt32LE(5, 1); b.write('plane', 5); b.writeUInt32LE(1, 10 + 144 + 8)
  const signed = Buffer.from(b); signed.writeDoubleLE(-0, 10)
  assert.throws(() => compareOutput(3, b, signed))
  const region = Buffer.from(b); region.writeUInt32LE(4, 10 + 144 + 12)
  assert.throws(() => compareOutput(3, b, region))
  assert.equal(compareOutput(3, b, b), 0)
})

test('committed Chrome evidence retains full corpus and module identities', async () => {
  const { readFileSync } = await import('node:fs')
  const { createHash } = await import('node:crypto')
  const base = new URL('../evidence/browser-wasm-2026-11-01/', import.meta.url)
  const bytes = readFileSync(new URL('report.json', base)), report = JSON.parse(bytes)
  const manifest = JSON.parse(readFileSync(new URL('manifest.json', base)))
  assert.equal(createHash('sha256').update(bytes).digest('hex'), manifest.report_sha256)
  assert.equal(report.status, 'passed'); assert.equal(report.runtime, 'chrome-wasm')
  assert.equal(report.candidate_sha, manifest.candidate_sha); assert.equal(report.reference_sha, manifest.reference_sha)
  assert.deepEqual(report.browser.imports, []); assert.deepEqual(report.page_errors, [])
  assert.equal(report.module_sha256, manifest.module_sha256)
  assert.equal(report.browser.moduleSha256, manifest.module_sha256)
  assert.match(report.browser.version, /^Chrome\//); assert.match(report.node, /^v22\./)
  assert.equal(report.cases.length, 114); assert.equal(report.successful_value_cases, 90); assert.equal(report.source_refusal_cases, 24)
  const definitions = JSON.parse(readFileSync(new URL('../ci/wasm-suites.json', import.meta.url))).suites
  const expected = []
  for (const def of definitions) {
    const unit = JSON.parse(readFileSync(new URL(`../plan/units/${def.unit}.json`, import.meta.url)))
    for (const id of unit.evidence.find(e => e.kind === 'differential').case_ids) expected.push(def.unit + '/' + id)
  }
  assert.deepEqual(report.cases.map(c => c.unit + '/' + c.id).sort(), expected.sort())
  assert.equal(new Set(expected).size, 114)
  for (const c of report.cases) for (const key of ['input_sha256', 'original_sha256', 'native_sha256', 'wasm_sha256']) assert.match(c[key], /^[a-f0-9]{64}$/)
})

test('a mismatch retains complete raw inputs and all outputs before failing', async () => {
  const { mkdtempSync, readFileSync, rmSync } = await import('node:fs')
  const { tmpdir } = await import('node:os')
  const { join } = await import('node:path')
  const { retainAndCompare } = await import('./browser-wasm-contract.mjs')
  const dir = mkdtempSync(join(tmpdir(), 'scanruler-wasm-mismatch-'))
  try {
    const files = { 'input.bin': Buffer.from([9]), 'original.bin': tls(), 'rust.bin': tls() }
    const wrong = tls(); wrong.writeDoubleLE(42, 1)
    await assert.rejects(retainAndCompare(dir, 1, files, wrong))
    for (const [name, bytes] of Object.entries({ ...files, 'wasm.bin': wrong })) assert.deepEqual(readFileSync(join(dir, name)), bytes)
  } finally { rmSync(dir, { recursive: true, force: true }) }
})
