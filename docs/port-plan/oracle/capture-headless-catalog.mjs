// SPDX-License-Identifier: AGPL-3.0-only
// One fresh process per selection: the source engine permits only one session.
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs'
import { resolve, dirname, relative, sep } from 'node:path'
import { pathToFileURL } from 'node:url'
import os from 'node:os'
import { canonical, digest, validateCatalog } from './catalog-contract.mjs'
import { reference } from './catalog-reference.mjs'
const ref = reference(process.argv[2])
const selection = process.argv[3]
const output = resolve(process.argv[4])
if (output === ref.root || !relative(ref.root, output).startsWith('..' + sep)) throw Error('Catalog output must be outside the frozen checkout')
if (!['core', 'all'].includes(selection)) throw Error('Expected core or all mode')
if (selection === 'core') globalThis.location = { search: '?plugins=none' }
const timer = setTimeout(() => { console.error('Original engine startup/capture timed out'); process.exit(1) }, 120000)
let engine
try {
  const api = await import(pathToFileURL(resolve(ref.root, 'dist-catalog/catalog.mjs')).href)
  engine = await api.startEngine()
  if (engine.app !== 'ScanRuler' || engine.version !== ref.pin.app_version) throw Error('Wrong original engine identity')
  const state = await engine.run('session.state', {})
  if (state?.app?.viewport !== false || state.scan !== null) throw Error('Original headless startup state differs from source contract')
  const commands = api.listCommands().map(c => ({ ...c, headless: api.runsHeadless(c.name) }))
  const data = { selection, plugins: engine.plugins, commands,
    offeredNames: engine.listCommands().map(c => c.name), pageOnlyNames: engine.pageCommands() }
  const commandsSha256 = validateCatalog(data)
  reference(ref.root)
  const record = { contract: 'original.headless.catalog.v1', status: 'captured',
    referenceSha: ref.pin.commit, candidateSha: ref.candidate, appVersion: engine.version,
    capturedAt: new Date().toISOString(), runtime: 'original-headless-adapter-ssr',
    environment: { node: process.version, versions: process.versions, os: os.type(), release: os.release(), arch: os.arch(), cpuCount: os.cpus().length, locale: Intl.DateTimeFormat().resolvedOptions().locale },
    locks: ref.locks, commandsSha256,
    originalEngineSha256: digest(readFileSync(resolve(ref.root, 'dist-engine/engine.mjs'))),
    adapterBundleSha256: digest(readFileSync(resolve(ref.root, 'dist-catalog/catalog.mjs'))),
    startupStateCheck: { viewport: state.app.viewport, scanEmpty: state.scan === null },
    ...data }
  mkdirSync(dirname(output), { recursive: true })
  writeFileSync(output, JSON.stringify(canonical(record), null, 2) + '\n')
  console.log(`${selection}: ${commands.length} registered; ${data.offeredNames.length} headless; ${data.pageOnlyNames.length} page-only; schema hash ${commandsSha256}`)
} finally {
  await engine?.stop()
  clearTimeout(timer)
}
