// SPDX-License-Identifier: AGPL-3.0-only
// Run the actual original app and original Chrome helpers in a fresh process/profile.
import { writeFileSync, readFileSync, mkdirSync, copyFileSync } from 'node:fs'
import { resolve, relative, sep } from 'node:path'
import { execFileSync } from 'node:child_process'
import os from 'node:os'
import { reference } from './catalog-reference.mjs'
import { canonical } from './catalog-contract.mjs'
import { validateBrowserCatalog } from './browser-catalog-contract.mjs'
const ref = reference(process.argv[2]), selection = process.argv[3], outputDir = resolve(process.argv[4])
if (!['core', 'all'].includes(selection)) throw Error('Expected core or all')
if (outputDir === ref.root || !relative(ref.root, outputDir).startsWith('..' + sep)) throw Error('Browser evidence must be outside frozen checkout')
mkdirSync(outputDir, { recursive: true })
copyFileSync(new URL('./browser-catalog-launcher.mjs', import.meta.url), resolve(ref.root, '.port-browser-catalog.mjs'))
execFileSync(process.execPath, ['.port-browser-catalog.mjs', selection, outputDir], { cwd: ref.root, stdio: 'inherit', timeout: 240000 })
reference(ref.root)
const data = JSON.parse(readFileSync(resolve(outputDir, selection + '-raw.json'), 'utf8'))
if (data.appVersion !== ref.pin.app_version) throw Error('Wrong original browser version')
const commandsSha256 = validateBrowserCatalog({ selection, ...data })
const record = { contract: 'original.browser.catalog.v1', status: 'captured', runtime: 'original-browser-vite-dev',
  referenceSha: ref.pin.commit, candidateSha: ref.candidate, capturedAt: new Date().toISOString(), selection,
  environment: { node: process.version, os: os.type(), release: os.release(), arch: os.arch() }, locks: ref.locks,
  commandsSha256, ...data }
writeFileSync(resolve(outputDir, selection + '.json'), JSON.stringify(canonical(record), null, 2) + '\n')
console.log(`${selection}: actual browser ${data.commands.length} registered, viewport=true, schema hash ${commandsSha256}`)
