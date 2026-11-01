// SPDX-License-Identifier: AGPL-3.0-only
// Browser original-registry data, not headless availability or Rust parity.
import { validateCatalog } from './catalog-contract.mjs'
export function validateBrowserCatalog(c) {
  if (c.startupViewport !== true || c.scanEmpty !== true || c.consoleStatus !== 'source-helper-pass') throw Error('Original browser startup/console invariant failed')
  if (!Array.isArray(c.browserOfferedNames) || JSON.stringify(c.browserOfferedNames) !== JSON.stringify(c.commands?.map(x => x.name))) throw Error('Browser offered names differ from registered commands')
  return validateCatalog({ ...c, offeredNames: c.headlessFlagNames, pageOnlyNames: c.pageOnlyFlagNames })
}
export function verifyBrowserSnapshot(current, base) {
  const actual = validateBrowserCatalog(current), expected = validateBrowserCatalog(base)
  if (current.selection !== base.selection || actual !== expected) throw Error('Frozen browser command metadata changed')
  return actual
}
