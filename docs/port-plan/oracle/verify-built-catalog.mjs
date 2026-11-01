// SPDX-License-Identifier: AGPL-3.0-only
// Verify the actual production headless bundle agrees with the thin API adapter.
import { readFileSync, writeFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { pathToFileURL } from 'node:url'
import { canonical, digest } from './catalog-contract.mjs'
import { reference } from './catalog-reference.mjs'
const ref = reference(process.argv[2]), mode = process.argv[3], dir = resolve(process.argv[4])
if (!['core', 'all'].includes(mode)) throw Error('Expected core or all')
if (mode === 'core') globalThis.location = { search: '?plugins=none' }
const capture = JSON.parse(readFileSync(resolve(dir, mode + '.json'), 'utf8'))
const timer = setTimeout(() => { console.error('Original built engine timed out'); process.exit(1) }, 120000)
let engine
try {
  const modulePath = resolve(ref.root, 'dist-engine/engine.mjs')
  if (digest(readFileSync(modulePath)) !== capture.originalEngineSha256) throw Error('Capture/built-module hash differs')
  engine = await (await import(pathToFileURL(modulePath).href)).startEngine()
  const same = (a, b) => JSON.stringify(canonical(a)) === JSON.stringify(canonical(b))
  const expected = capture.commands.filter(c => c.headless).map(({ headless, ...info }) => info)
  if (!same(engine.listCommands(), expected) || !same(engine.pageCommands(), capture.pageOnlyNames) || !same(engine.plugins, capture.plugins)) throw Error('Actual built engine disagrees with adapter catalog')
  const state = await engine.run('session.state', {})
  if (engine.app !== 'ScanRuler' || engine.version !== ref.pin.app_version || state?.app?.viewport !== false || state.scan !== null) throw Error('Built engine startup invariant failed')
  reference(ref.root)
  writeFileSync(resolve(dir, 'built-' + mode + '.json'), JSON.stringify({ contract: 'original.built-engine.catalog.v1', status: 'verified', referenceSha: ref.pin.commit, candidateSha: ref.candidate, mode, engineSha256: capture.originalEngineSha256, offered: expected.length, pageOnly: capture.pageOnlyNames.length, startupViewport: state.app.viewport, scanEmpty: state.scan === null }, null, 2) + '\n')
  console.log(`${mode}: actual built engine matches all offered schemas and unavailable names`)
} finally { await engine?.stop(); clearTimeout(timer) }
