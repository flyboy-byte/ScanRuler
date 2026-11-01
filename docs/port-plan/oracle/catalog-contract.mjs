// SPDX-License-Identifier: AGPL-3.0-only
// Oracle evidence validation only; no Rust product or geometry implementation.
import { createHash } from 'node:crypto'

export function canonical(value) {
  if (Array.isArray(value)) return value.map(canonical)
  if (value && typeof value === 'object') return Object.fromEntries(Object.keys(value).sort().map(k => [k, canonical(value[k])]))
  return value
}
export const digest = bytes => createHash('sha256').update(bytes).digest('hex')
const equal = (a, b) => JSON.stringify(a) === JSON.stringify(b)
export function validateCatalog(catalog) {
  const { selection, plugins, commands, offeredNames, pageOnlyNames } = catalog
  if (!['core', 'all'].includes(selection)) throw Error('Unknown selection')
  const expectedPlugins = selection === 'core' ? [] : ['mesh-tools', 'reverse']
  if (!Array.isArray(plugins) || !equal([...plugins].sort(), expectedPlugins)) throw Error('Unexpected installed plugins')
  if (!Array.isArray(commands) || commands.length === 0) throw Error('Empty command catalog')
  const names = commands.map(c => c.name)
  if (names.some(n => typeof n !== 'string' || !n) || new Set(names).size !== names.length) throw Error('Invalid/duplicate command names')
  for (const c of commands) {
    if (typeof c.title !== 'string' || typeof c.description !== 'string' || !c.input || typeof c.input !== 'object' || Array.isArray(c.input)) throw Error('Missing command schema/description')
    for (const flag of ['headless', 'readOnly', 'returnsFile', 'returnsImage']) if (typeof c[flag] !== 'boolean') throw Error('Missing command flag: ' + flag)
  }
  if (!equal(offeredNames, commands.filter(c => c.headless).map(c => c.name))) throw Error('Headless offered list mismatch')
  if (!equal(pageOnlyNames, commands.filter(c => !c.headless).map(c => c.name))) throw Error('Page-only list mismatch')
  for (const n of ['session.state', 'history.undo', 'history.redo']) if (!offeredNames.includes(n)) throw Error('Missing source headless invariant: ' + n)
  for (const n of ['view.render', 'scan.open']) if (!pageOnlyNames.includes(n)) throw Error('Missing source page invariant: ' + n)
  return digest(JSON.stringify(canonical(commands)))
}
