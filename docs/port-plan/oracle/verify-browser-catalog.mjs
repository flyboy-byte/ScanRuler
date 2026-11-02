// SPDX-License-Identifier: AGPL-3.0-only
// Compare complete original browser metadata; changing run context is ignored.
import { readFileSync } from 'node:fs'
import { gunzipSync } from 'node:zlib'
import { resolve } from 'node:path'
import { verifyBrowserSnapshot } from './browser-catalog-contract.mjs'
for (const mode of ['core', 'all']) {
  const current = JSON.parse(readFileSync(resolve(process.argv[2], mode + '.json'), 'utf8'))
  const base = JSON.parse(gunzipSync(readFileSync(new URL(`../evidence/browser-2026-11-01/${mode}.json.gz`, import.meta.url))))
  if (current.referenceSha !== base.referenceSha || current.appVersion !== base.appVersion || current.status !== 'captured' || current.runtime !== base.runtime) throw Error('Incompatible original browser identity')
  console.log(`${mode}: frozen browser metadata retained: ${verifyBrowserSnapshot(current, base)}`)
}
