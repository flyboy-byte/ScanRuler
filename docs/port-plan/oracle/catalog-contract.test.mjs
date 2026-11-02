// SPDX-License-Identifier: AGPL-3.0-only
import { test } from 'node:test'
import assert from 'node:assert/strict'
import { canonical, validateCatalog } from './catalog-contract.mjs'
function valid() {
  const offeredNames = ['history.redo', 'history.undo', 'session.state']
  const pageOnlyNames = ['scan.open', 'view.render']
  return { selection: 'core', plugins: [], offeredNames, pageOnlyNames,
    commands: [...offeredNames, ...pageOnlyNames].map(name => ({ name, title: name, description: name, input: { type: 'object', properties: {} }, headless: offeredNames.includes(name), readOnly: false, returnsFile: false, returnsImage: false })) }
}
test('valid synthetic validation contract and canonical key order', () => {
  assert.match(validateCatalog(valid()), /^[0-9a-f]{64}$/)
  assert.deepEqual(canonical({ z: 1, a: [{ y: 2, b: 3 }] }), { a: [{ b: 3, y: 2 }], z: 1 })
})
for (const [name, mutate] of [
  ['empty suite', c => { c.commands = [] }],
  ['duplicate identity', c => c.commands.push(c.commands[0])],
  ['missing schema', c => { delete c.commands[0].input }],
  ['missing flag', c => { delete c.commands[0].returnsFile }],
  ['offered mismatch', c => { c.offeredNames = [] }],
  ['page overlap', c => c.pageOnlyNames.push(c.offeredNames[0])],
  ['missing core invariant', c => { c.commands = c.commands.filter(x => x.name !== 'session.state'); c.offeredNames = c.offeredNames.filter(x => x !== 'session.state') }],
  ['wrong plugins', c => c.plugins.push('reverse')],
  ['unknown mode', c => { c.selection = 'browser' }],
]) test('rejects ' + name, () => { const c = valid(); mutate(c); assert.throws(() => validateCatalog(c)) })

test('snapshot guard detects optional-field/schema loss with otherwise valid catalogs', async () => {
  const { verifyCatalogSnapshot } = await import('./catalog-contract.mjs')
  const baseline = valid()
  baseline.commands[0].names = { space: 'history', at: ['item'] }
  const current = structuredClone(baseline)
  delete current.commands[0].names
  validateCatalog(current) // remains structurally valid, but cannot lose baseline values
  assert.throws(() => verifyCatalogSnapshot(current, baseline))
  const changed = structuredClone(baseline)
  changed.commands[0].input.properties.newField = { type: 'string' }
  assert.throws(() => verifyCatalogSnapshot(changed, baseline))
  assert.equal(verifyCatalogSnapshot(baseline, structuredClone(baseline)), validateCatalog(baseline))
})

test('captured source snapshots retain full data/hash/partition identities', async () => {
  const { readFileSync } = await import('node:fs')
  const { gunzipSync } = await import('node:zlib')
  const { digest, verifyCatalogSnapshot } = await import('./catalog-contract.mjs')
  const base = new URL('../evidence/headless-2026-11-01/', import.meta.url)
  const manifest = JSON.parse(readFileSync(new URL('manifest.json', base), 'utf8'))
  const modes = {}
  for (const [mode, entry] of Object.entries(manifest.files)) {
    const compressed = readFileSync(new URL(entry.path, base))
    assert.equal(digest(compressed), entry.gzipSha256)
    const raw = gunzipSync(compressed)
    assert.equal(digest(raw), entry.rawSha256)
    const d = JSON.parse(raw)
    assert.equal(d.status, 'captured')
    assert.equal(d.referenceSha, manifest.referenceSha)
    assert.equal(d.candidateSha, manifest.candidateSha)
    assert.match(d.environment.node, /^v22\./)
    assert.equal(verifyCatalogSnapshot(d, d), entry.commandsSha256)
    assert.equal(d.commands.length, entry.registered)
    assert.equal(d.offeredNames.length, entry.headless)
    assert.equal(d.pageOnlyNames.length, entry.pageOnly)
    modes[mode] = d
  }
  assert.ok(modes.core.commands.every(c => modes.all.commands.some(a => a.name === c.name)))
})

test('actual built-engine proof retains source/module/partition identities', async () => {
  const { readFileSync } = await import('node:fs')
  const { digest } = await import('./catalog-contract.mjs')
  const base = new URL('../evidence/headless-2026-11-01/', import.meta.url)
  const manifest = JSON.parse(readFileSync(new URL('manifest.json', base), 'utf8'))
  for (const [mode, file] of Object.entries(manifest.builtEngineVerification.files)) {
    const bytes = readFileSync(new URL(file.path, base))
    assert.equal(digest(bytes), file.sha256)
    const d = JSON.parse(bytes)
    assert.equal(d.status, 'verified')
    assert.equal(d.referenceSha, manifest.referenceSha)
    assert.equal(d.candidateSha, manifest.builtEngineVerification.candidateSha)
    assert.equal(d.engineSha256, manifest.builtEngineVerification.engineSha256)
    assert.equal(d.offered, manifest.files[mode].headless)
    assert.equal(d.pageOnly, manifest.files[mode].pageOnly)
    assert.equal(d.startupViewport, false)
    assert.equal(d.scanEmpty, true)
  }
})
