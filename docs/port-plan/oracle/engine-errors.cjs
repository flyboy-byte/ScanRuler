// SPDX-License-Identifier: AGPL-3.0-only
// Entire pinned class after TypeScript annotation erasure, not a reimplementation.
const { execFileSync } = require('node:child_process')
const { readFileSync } = require('node:fs')
const vm = require('node:vm')
const ts = require(process.cwd() + '/node_modules/typescript')
const pin = '2cd89c6769a10c27ee7754b477ada521c9ef5cb7'
const source = execFileSync('git', ['show', `${pin}:src/commands/types.ts`], { encoding: 'utf8' })
const union = source.match(/export type CommandErrorCode =([\s\S]*?)export class CommandError/)
if (!union) throw Error('Missing original error union')
const codes = [...union[1].matchAll(/'([^']+)'/g)].map(m => m[1])
if (codes.length !== 12) throw Error('Changed original code schedule')
const js = ts.transpileModule(source, { compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.CommonJS } }).outputText
const exported = {}
vm.runInNewContext(js, { exports: exported })
const bytes = readFileSync(0), count = bytes.readUInt32LE(1)
if (bytes[0] >= codes.length || bytes.length !== 5 + 2 * count) throw Error('Invalid test input')
let message = ''
for (let i = 0; i < count; i++) message += String.fromCharCode(bytes.readUInt16LE(5 + 2 * i))
const e = new exported.CommandError(codes[bytes[0]], message)
const fields = [Buffer.from(e.code, 'utf8'), Buffer.from(e.name, 'utf16le'), Buffer.from(e.message, 'utf16le'), Buffer.from(String(e), 'utf16le'), Buffer.from(JSON.stringify(e.toJSON()), 'utf8')]
process.stdout.write(Buffer.concat([Buffer.from('ERR1'), ...fields.flatMap(b => { const n = Buffer.alloc(4); n.writeUInt32LE(b.length); return [n,b] })]))
