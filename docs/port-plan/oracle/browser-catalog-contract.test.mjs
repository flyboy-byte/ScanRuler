// SPDX-License-Identifier: AGPL-3.0-only
import { test } from 'node:test'
import assert from 'node:assert/strict'
import { validateBrowserCatalog, verifyBrowserSnapshot } from './browser-catalog-contract.mjs'
function valid() {
  const names = ['guide.read', 'history.redo', 'history.undo', 'scan.open', 'session.state', 'view.render']
  const headlessFlagNames = names.filter(x => !['scan.open', 'view.render'].includes(x))
  return { selection: 'core', plugins: [], startupViewport: true, scanEmpty: true, consoleStatus: 'source-helper-pass', browserOfferedNames: names, headlessFlagNames, pageOnlyFlagNames: ['scan.open', 'view.render'], commands: names.map(name => ({ name, title: name, description: name, input: { type: 'object' }, readOnly: false, returnsFile: false, returnsImage: false, headless: headlessFlagNames.includes(name) })) }
}
test('valid browser contract retains all registered commands', () => assert.match(validateBrowserCatalog(valid()), /^[0-9a-f]{64}$/))
for (const [name, change] of [
  ['headless viewport', c => { c.startupViewport = false }],
  ['loaded scan', c => { c.scanEmpty = false }],
  ['console errors', c => { c.consoleStatus = 'failed' }],
  ['dropped offered name', c => c.browserOfferedNames.pop()],
  ['incorrect flag partition', c => c.headlessFlagNames.push('view.render')],
  ['empty registry', c => { c.commands = []; c.browserOfferedNames = [] }],
]) test('rejects ' + name, () => { const c = valid(); change(c); assert.throws(() => validateBrowserCatalog(c)) })
test('schema changes cannot pass a frozen browser baseline', () => {
  const base = valid(), current = structuredClone(base)
  current.commands[0].input.properties = { changed: { type: 'number' } }
  validateBrowserCatalog(current)
  assert.throws(() => verifyBrowserSnapshot(current, base))
  assert.equal(verifyBrowserSnapshot(base, structuredClone(base)), validateBrowserCatalog(base))
})

test('actual browser snapshots retain hashes, runtime and independent registry metadata', async () => {
  const { readFileSync } = await import('node:fs')
  const { gunzipSync } = await import('node:zlib')
  const { digest } = await import('./catalog-contract.mjs')
  const base = new URL('../evidence/browser-2026-11-01/', import.meta.url)
  const manifest = JSON.parse(readFileSync(new URL('manifest.json', base), 'utf8'))
  for (const [mode, file] of Object.entries(manifest.files)) {
    const compressed = readFileSync(new URL(file.path, base))
    assert.equal(digest(compressed), file.gzipSha256)
    const raw = gunzipSync(compressed)
    assert.equal(digest(raw), file.rawSha256)
    const d = JSON.parse(raw)
    assert.equal(d.referenceSha, manifest.referenceSha)
    assert.equal(d.candidateSha, manifest.candidateSha)
    assert.equal(d.status, 'captured')
    assert.equal(d.runtime, 'original-browser-vite-dev')
    assert.match(d.environment.node, /^v22\./)
    assert.match(d.browserVersion, /^Chrome\//)
    assert.equal(verifyBrowserSnapshot(d, d), file.commandsSha256)
    assert.equal(d.commands.length, file.registered)
    assert.equal(d.headlessFlagNames.length, file.headlessFlags)
    assert.equal(d.pageOnlyFlagNames.length, file.pageOnlyFlags)
    const headless = JSON.parse(gunzipSync(readFileSync(new URL(`../evidence/headless-2026-11-01/${mode}.json.gz`, import.meta.url))))
    assert.deepEqual(d.commands, headless.commands)
  }
})
