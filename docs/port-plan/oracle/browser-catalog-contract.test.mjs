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
