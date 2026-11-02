// SPDX-License-Identifier: AGPL-3.0-only
// Reference-only recipes; no Rust results are inputs. Generated temporary paths
// are neither published nor retained. Each recipe must repeat byte for byte.
import { createHash } from 'node:crypto'
import { readFileSync, rmSync } from 'node:fs'
import { dirname } from 'node:path'
import { fileURLToPath } from 'node:url'
import { ballPairScan, filletRingScan, turnedBlockScan, noisyBoxScan, notchedBlockScan } from '../../../scripts/e2e-fixtures.mjs'

const source = fileURLToPath(new URL('../../../scripts/e2e-fixtures.mjs', import.meta.url))
const hash = (b) => createHash('sha256').update(b).digest('hex')
function fromPath(make) {
  const path = make()
  try { return readFileSync(path) }
  finally { rmSync(dirname(path), { recursive: true }) }
}
const recipes = [
  ['ball-pair', 'ballPairScan({distance:148.64,r:7.96,rings:48,segments:96})', null, () => fromPath(() => ballPairScan())],
  ['fillet-ring', 'filletRingScan({R:25,r:3,radial:240,tube:24})', null, () => fromPath(() => filletRingScan())],
  ['turned-block', 'turnedBlockScan()', null, () => turnedBlockScan().bytes],
  ['noisy-closed-box', 'noisyBoxScan({seed:7,noise:0.02,holes:false})', 7, () => noisyBoxScan().bytes],
  ['noisy-open-box', 'noisyBoxScan({seed:7,noise:0.02,holes:true})', 7, () => noisyBoxScan({ holes: true }).bytes],
  ['notched-block', 'notchedBlockScan()', null, () => fromPath(() => notchedBlockScan())],
]
const assets = recipes.map(([id, recipe, seed, make]) => {
  const a = make(), b = make()
  if (!a.equals(b)) throw new Error(`non-reproducible original fixture: ${id}`)
  if (a.length !== 84 + 50 * a.readUInt32LE(80)) throw new Error(`bad STL fixture: ${id}`)
  return { id, recipe, seed, units: 'mm', license: 'AGPL-3.0-only',
    source: 'scripts/e2e-fixtures.mjs', bytes: a.length, triangles: a.readUInt32LE(80), sha256: hash(a),
    repeated_bytes_equal: true }
})
console.log(JSON.stringify({ schema: 1, generator_sha256: hash(readFileSync(source)),
  license_source: 'LICENSE', assets }, null, 2))
