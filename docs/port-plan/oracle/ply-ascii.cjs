// SPDX-License-Identifier: AGPL-3.0-only
// TypeScript is the frozen original's dev dependency, used only to erase types.
const { execFileSync } = require('node:child_process')
const { readFileSync } = require('node:fs')
const vm = require('node:vm')
const ts = require(process.cwd() + '/node_modules/typescript')
const pin = '2cd89c6769a10c27ee7754b477ada521c9ef5cb7'
const source = execFileSync('git', ['show', `${pin}:src/core/parsers/ply.ts`], { encoding: 'utf8' })
const js = ts.transpileModule(source, { compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.CommonJS } }).outputText
const exportsObject = {}
vm.runInNewContext(js, { exports: exportsObject, TextDecoder, Uint8Array, Float32Array, Uint32Array, DataView })
const bytes = readFileSync(0)
try {
  const mesh = exportsObject.parsePLY(bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength), s => process.stderr.write(s + '\n'))
  if (mesh.kind !== 'indexed' || !mesh.indices) throw new Error('Unexpected pinned result shape')
  const out = Buffer.alloc(16 + 4 * (mesh.positions.length + mesh.indices.length))
  out.write('PLY1', 0, 'ascii')
  out.writeUInt32LE(mesh.keepWinding === true ? 1 : 0, 4)
  out.writeUInt32LE(mesh.positions.length, 8)
  out.writeUInt32LE(mesh.indices.length, 12)
  for (let i = 0; i < mesh.positions.length; i++) out.writeFloatLE(mesh.positions[i], 16 + i * 4)
  for (let i = 0; i < mesh.indices.length; i++) out.writeUInt32LE(mesh.indices[i], 16 + mesh.positions.length * 4 + i * 4)
  process.stdout.write(out)
} catch (e) {
  process.stderr.write(e.message + '\n')
  process.exitCode = 2
}
