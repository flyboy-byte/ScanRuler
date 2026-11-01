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
