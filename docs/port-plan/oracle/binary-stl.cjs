// SPDX-License-Identifier: AGPL-3.0-only
// Oracle-only adapter. Execute the pinned helper body, not a reimplementation.
const { execFileSync } = require('node:child_process')
const { readFileSync } = require('node:fs')
const vm = require('node:vm')
const pin = '2cd89c6769a10c27ee7754b477ada521c9ef5cb7'
const source = execFileSync('git', ['show', `${pin}:src/core/parsers/stl.ts`], { encoding: 'utf8' })
const signature = 'function parseBinary(dv: DataView, triCount: number, onProgress?: (text: string) => void): ParsedMesh {'
const start = source.indexOf(signature)
const end = source.indexOf('\nfunction parseAscii', start)
if (start < 0 || end < 0) throw new Error('Pinned helper boundaries changed')
const helper = source.slice(start, end).replace(signature, 'function parseBinary(dv, triCount, onProgress) {')
const parse = vm.runInNewContext(`${helper}\nparseBinary`, { Float32Array })
const bytes = readFileSync(0)
const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength)
try {
  const result = parse(view, Number(process.argv[2]), text => process.stderr.write(text + '\n'))
  // Buffer bytes preserve signed zeros and NaN payloads, which JSON would erase.
  const out = Buffer.alloc(result.positions.length * 4)
  const dv = new DataView(out.buffer, out.byteOffset, out.byteLength)
  for (let i = 0; i < result.positions.length; i++) dv.setFloat32(i * 4, result.positions[i], true)
  process.stdout.write(out)
} catch (error) {
  process.stderr.write(error.name + '\n')
  process.exitCode = 2
}
