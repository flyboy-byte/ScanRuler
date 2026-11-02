// SPDX-License-Identifier: AGPL-3.0-only
// Frozen original only. Runs many inputs through one original parser in a single process.
// usage: node fuzz-batch.cjs <ply|stl>   stdin: u32le count, then per case u32le length + bytes
// stdout per case: u8 status (1 ok, 0 refused, 2 original timed out), u32 progress length + text (lines joined by \n),
//   u32 message length + text, u32 output length + bytes (same encodings as ply-ascii.cjs / stl-dispatch.cjs)
const { execFileSync } = require('node:child_process')
const { readFileSync } = require('node:fs')
const vm = require('node:vm')
const pin = '2cd89c6769a10c27ee7754b477ada521c9ef5cb7'
const unit = process.argv[2]
// A case the original cannot finish in this time is reported as status 2 (its own class).
const LIMIT_MS = Number(process.env.SCANRULER_FUZZ_CASE_MS || 1500)
const show = path => execFileSync('git', ['show', `${pin}:${path}`], { encoding: 'utf8', maxBuffer: 1 << 26 })
let parse
if (unit === 'ply') {
  const ts = require(process.cwd() + '/node_modules/typescript')
  const js = ts.transpileModule(show('src/core/parsers/ply.ts'), { compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.CommonJS } }).outputText
  const exportsObject = {}
  const ctx = vm.createContext({ exports: exportsObject, TextDecoder, Uint8Array, Float32Array, Uint32Array, DataView })
  vm.runInContext(js, ctx)
  parse = (input, progress) => {
    ctx.__input = input; ctx.__progress = progress
    const mesh = vm.runInContext('exports.parsePLY(__input, __progress)', ctx, { timeout: LIMIT_MS })
    if (mesh.kind !== 'indexed' || !mesh.indices) throw new Error('Unexpected pinned result shape')
    const out = Buffer.alloc(16 + 4 * (mesh.positions.length + mesh.indices.length))
    out.write('PLY1', 0, 'ascii')
    out.writeUInt32LE(mesh.keepWinding === true ? 1 : 0, 4)
    out.writeUInt32LE(mesh.positions.length, 8)
    out.writeUInt32LE(mesh.indices.length, 12)
    for (let i = 0; i < mesh.positions.length; i++) out.writeFloatLE(mesh.positions[i], 16 + i * 4)
    for (let i = 0; i < mesh.indices.length; i++) out.writeUInt32LE(mesh.indices[i], 16 + mesh.positions.length * 4 + i * 4)
    return out
  }
} else if (unit === 'stl') {
  let source = show('src/core/parsers/stl.ts')
  const replacements = [
    ["import type { ParsedMesh } from '../types'", ''],
    ['export function parseSTL(buffer: ArrayBuffer, onProgress?: (text: string) => void): ParsedMesh {', 'function parseSTL(buffer, onProgress) {'],
    ['function parseBinary(dv: DataView, triCount: number, onProgress?: (text: string) => void): ParsedMesh {', 'function parseBinary(dv, triCount, onProgress) {'],
    ['function parseAscii(buffer: ArrayBuffer, onProgress?: (text: string) => void): ParsedMesh {', 'function parseAscii(buffer, onProgress) {'],
    ['const coords: number[] = []', 'const coords = []'],
    ['let m: RegExpExecArray | null', 'let m'],
  ]
  for (const [before, after] of replacements) {
    if (!source.includes(before)) throw new Error('Pinned source annotation changed: ' + before)
    source = source.replace(before, after)
  }
  const ctx = vm.createContext({ Float32Array, TextDecoder, Uint8Array, DataView })
  vm.runInContext(source, ctx)
  parse = (input, progress) => {
    ctx.__input = input; ctx.__progress = progress
    const result = vm.runInContext('parseSTL(__input, __progress)', ctx, { timeout: LIMIT_MS })
    if (result.kind !== 'soup') throw new Error('Unexpected source result kind')
    const out = Buffer.alloc(result.positions.length * 4)
    for (let i = 0; i < result.positions.length; i++) out.writeFloatLE(result.positions[i], i * 4)
    return out
  }
} else {
  throw new Error('unit must be ply or stl')
}
const data = readFileSync(0)
let at = 0
const count = data.readUInt32LE(at); at += 4
const chunks = []
const put = (buf) => { const h = Buffer.alloc(4); h.writeUInt32LE(buf.length); chunks.push(h, buf) }
for (let i = 0; i < count; i++) {
  const len = data.readUInt32LE(at); at += 4
  const bytes = data.subarray(at, at + len); at += len
  const input = bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength)
  const lines = []
  let ok = 1, message = '', out = Buffer.alloc(0)
  try { out = parse(input, s => lines.push(String(s))) } catch (e) {
    if (e && e.code === 'ERR_SCRIPT_EXECUTION_TIMEOUT') ok = 2
    else { ok = 0; message = e && e.message !== undefined ? String(e.message) : String(e) }
  }
  chunks.push(Buffer.from([ok]))
  put(Buffer.from(lines.join('\n'), 'utf8'))
  put(Buffer.from(message, 'utf8'))
  put(out)
}
process.stdout.write(Buffer.concat(chunks))
