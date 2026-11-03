// SPDX-License-Identifier: AGPL-3.0-only
// The frozen original's 3MF reader (src/core/parsers/threemf.ts and objTexture.ts from the pinned git
// object, TypeScript erased by the original's own `typescript`) run with the original's pinned fflate.
//   node threemf.cjs run    stdin: u32 count, then per case
//                             u32 zip length + bytes, u32 image count, per image
//                             (u32 name length + utf8, u32 width, u32 height, u32 rgba length + bytes)
//                           stdout per case:
//                             u8 status (1 ok, 0 error, 2 timed out after 1.5 s)
//                             u32 progress line count, per line u32 length + utf8
//                             ok:    u32 float count + float32 bytes, u32 index count + u32 bytes,
//                                    u8 has colours [+ u32 length + bytes],
//                                    u8 texture (0 none, 1 applied, 2 missing) [+ u32 count, strings]
//                             error: u8 isTypeError, u32 message length + utf8
//   node threemf.cjs pack   stdin: u32 count, per archive u8 level, u32 entries, per entry
//                             (u32 name length + utf8, u32 length + bytes); stdout: the same framing
//                             as the input of `run` without images (u32 count, u32 length + zip each)
const { execFileSync } = require('node:child_process')
const { readFileSync } = require('node:fs')
const path = require('node:path')
const vm = require('node:vm')
const root = path.resolve(__dirname, '../../..')
const pin = '2cd89c6769a10c27ee7754b477ada521c9ef5cb7'
const git = (file) => execFileSync('git', ['show', `${pin}:${file}`], { cwd: root, encoding: 'utf8', maxBuffer: 1 << 28 })

const lock = JSON.parse(git('package-lock.json'))
const pinned = lock.packages['node_modules/fflate'].version
const fflate = require(path.join(root, 'node_modules/fflate'))
const installed = require(path.join(root, 'node_modules/fflate/package.json')).version
if (pinned !== installed) throw new Error(`fflate ${installed} installed, original pins ${pinned}`)
const ts = require(path.join(root, 'node_modules/typescript'))

const u32 = (n) => { const b = Buffer.alloc(4); b.writeUInt32LE(n >>> 0); return b }
const str = (s) => { const b = Buffer.from(s, 'utf8'); return Buffer.concat([u32(b.length), b]) }
const data = readFileSync(0)
let at = 0
const word = () => { const v = data.readUInt32LE(at); at += 4; return v }
const text = () => { const n = word(); const s = data.subarray(at, at + n).toString('utf8'); at += n; return s }
const bytes = () => { const n = word(); const v = Uint8Array.from(data.subarray(at, at + n)); at += n; return v }

if (process.argv[2] === 'pack') {
  const count = word()
  const out = [u32(count)]
  for (let i = 0; i < count; i++) {
    const level = data[at++]
    const files = {}
    const entries = word()
    for (let j = 0; j < entries; j++) {
      const name = text()
      files[name] = [bytes(), { level, mtime: new Date(2020, 0, 1) }]
    }
    const z = Buffer.from(fflate.zipSync(files))
    out.push(u32(z.length), z)
  }
  // Exit only once the pipe has taken everything; an early exit truncates large output.
  process.stdout.write(Buffer.concat(out), () => process.exit(0))
  return
}

const load = (file, requires) => {
  const js = ts.transpileModule(git(file), { compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.CommonJS } }).outputText
  const exported = {}
  const mod = { exports: exported }
  vm.runInThisContext(`(function (exports, module, require) {${js}\n})`, { filename: file })(exported, mod, (name) => {
    if (name in requires) return requires[name]
    throw new Error('unexpected import ' + name)
  })
  return mod.exports
}
const objTexture = load('src/core/parsers/objTexture.ts', {})
const threemf = load('src/core/parsers/threemf.ts', { fflate, './objTexture': objTexture })
globalThis.__threemf = threemf

const count = word()
const out = []
for (let i = 0; i < count; i++) {
  const zip = bytes()
  const images = []
  for (let k = 0, n = word(); k < n; k++) images.push({ name: text(), width: word(), height: word(), rgba: bytes() })
  const progress = []
  globalThis.__job = { buffer: zip.buffer.slice(zip.byteOffset, zip.byteOffset + zip.length), images, progress }
  let status, result, error
  try {
    result = vm.runInThisContext(
      '__threemf.parse3MF(__job.buffer, (t) => __job.progress.push(t), { materials: [], images: __job.images })',
      { timeout: 1500 },
    )
    status = 1
  } catch (e) {
    error = e
    status = e && e.code === 'ERR_SCRIPT_EXECUTION_TIMEOUT' ? 2 : 0
  }
  out.push(Buffer.from([status]), u32(progress.length))
  for (const line of progress) out.push(str(line))
  if (status === 1) {
    out.push(u32(result.positions.length), Buffer.from(result.positions.buffer, result.positions.byteOffset, result.positions.byteLength))
    out.push(u32(result.indices.length), Buffer.from(result.indices.buffer, result.indices.byteOffset, result.indices.byteLength))
    if (result.colors) out.push(Buffer.from([1]), u32(result.colors.length), Buffer.from(result.colors))
    else out.push(Buffer.from([0]))
    if (!result.texture) out.push(Buffer.from([0]))
    else {
      const list = result.texture.kind === 'applied' ? result.texture.images : result.texture.files
      out.push(Buffer.from([result.texture.kind === 'applied' ? 1 : 2]), u32(list.length))
      for (const name of list) out.push(str(name))
    }
  } else if (status === 0) {
    out.push(Buffer.from([error instanceof TypeError ? 1 : 0]), str(String(error && error.message !== undefined ? error.message : error)))
  }
}
process.stdout.write(Buffer.concat(out))
