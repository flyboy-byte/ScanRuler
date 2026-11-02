// SPDX-License-Identifier: AGPL-3.0-only
// Guard every original command/schema/flag value, ignoring changing run context.
import { readFileSync } from 'node:fs'
import { gunzipSync } from 'node:zlib'
import { resolve } from 'node:path'
import { verifyCatalogSnapshot } from './catalog-contract.mjs'
for (const mode of ['core', 'all']) {
  const current = JSON.parse(readFileSync(resolve(process.argv[2], mode + '.json'), 'utf8'))
  const base = JSON.parse(gunzipSync(readFileSync(new URL(`../evidence/headless-2026-11-01/${mode}.json.gz`, import.meta.url))))
  if (current.referenceSha !== base.referenceSha || current.appVersion !== base.appVersion || current.status !== 'captured') throw Error('Incompatible original catalog identity')
  console.log(`${mode}: frozen metadata hash retained: ${verifyCatalogSnapshot(current, base)}`)
}
