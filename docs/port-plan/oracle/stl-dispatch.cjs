// SPDX-License-Identifier: AGPL-3.0-only
// Frozen original only: remove TypeScript signatures, execute all STL bodies.
const { execFileSync } = require('node:child_process')
const { readFileSync } = require('node:fs')
const vm = require('node:vm')
const pin = '2cd89c6769a10c27ee7754b477ada521c9ef5cb7'
let source = execFileSync('git', ['show', `${pin}:src/core/parsers/stl.ts`], { encoding: 'utf8' })
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
const parse = vm.runInNewContext(`${source}\nparseSTL`, { Float32Array, TextDecoder, Uint8Array, DataView })
const bytes = readFileSync(0)
const input = bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength)
try {
  const result = parse(input, text => process.stderr.write(text + '\n'))
  if (result.kind !== 'soup') throw new Error('Unexpected source result kind')
  const out = Buffer.alloc(result.positions.length * 4)
  const dv = new DataView(out.buffer, out.byteOffset, out.byteLength)
  for (let i = 0; i < result.positions.length; i++) dv.setFloat32(i * 4, result.positions[i], true)
  process.stdout.write(out)
} catch (error) {
  process.stderr.write(error.message + '\n')
  process.exitCode = 2
}
