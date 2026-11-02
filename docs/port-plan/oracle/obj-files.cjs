// SPDX-License-Identifier: AGPL-3.0-only
// Execute complete pinned modules; only erase TS and bind original dependencies.
const { execFileSync } = require('node:child_process')
const { readFileSync } = require('node:fs')
const vm = require('node:vm')
const ts = require(process.cwd() + '/node_modules/typescript')
const reference = '2cd89c6769a10c27ee7754b477ada521c9ef5cb7'
function load(path, imports) {
  const source = execFileSync('git', ['show', reference + ':' + path], { encoding: 'utf8' })
  const exports = {}
  vm.runInNewContext(ts.transpileModule(source, { compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.CommonJS } }).outputText,
    { exports, require: id => { if (!(id in imports)) throw Error('Unexpected source import ' + id); return imports[id] }, TextDecoder, Uint8Array })
  return exports
}
const f = load('src/core/formats.ts', {})
const s = load('src/app/scanFiles.ts', { '../core/formats': f, fflate: require(process.cwd() + '/node_modules/fflate') })
const b = readFileSync(0); let at = 0
const take = n => { if (n > b.length - at) throw Error('Truncated fixture'); const v = b.subarray(at, at + n); at += n; return v }
const byte = () => take(1)[0], word = () => take(4).readUInt32LE(), blob = () => take(word()), text = () => blob().toString('utf8')
const list = read => Array.from({ length: word() }, read)
const out = [Buffer.from('FLS1')]
const w = n => { const x = Buffer.alloc(4); x.writeUInt32LE(n); out.push(x) }
const raw = b => { w(b.length); out.push(Buffer.from(b)) }, str = s => raw(Buffer.from(s, 'utf8'))
const ids = v => { w(v.length); for (const i of v) w(i) }
;(async () => {
  const mode = byte(); out.push(Buffer.from([mode]))
  if (mode === 0) {
    const name = text(); str(f.extensionOf(name))
    for (const fn of ['isModelFile', 'isTextureFile', 'is3mfFile', 'isStepFile', 'isReferenceFile', 'isImageFile']) out.push(Buffer.from([Number(f[fn](name))]))
    for (const key of ['SCAN_ACCEPT', 'REFERENCE_ACCEPT', 'IMAGE_ACCEPT', 'SCAN_FORMATS', 'REFERENCE_FORMATS', 'IMAGE_FORMATS']) str(f[key])
  } else if (mode === 1) {
    const files = list(() => { const name = text(), data = Buffer.from(blob()); return { name, arrayBuffer: async () => Uint8Array.from(data).buffer } })
    const operations = list(() => ({ op: byte(), ids: list(word) }))
    w(operations.length)
    for (const operation of operations) {
      const input = operation.ids.map(i => { if (!files[i]) throw Error('Bad fixture ID'); return files[i] })
      out.push(Buffer.from([operation.op]))
      if (operation.op === 1) { const chosen = s.bundleFiles(input); w(chosen === null ? 0xffffffff : files.indexOf(chosen)) }
      else if (operation.op === 2) { if (input.length !== 1) throw Error('Bad query'); ids(s.companionsOf(input[0]).map(f => files.indexOf(f))) }
      else if (operation.op === 3) { const bytes = await s.companionBytes(input); w(bytes.length); for (const f of bytes) { str(f.name); raw(f.bytes) } }
      else throw Error('Bad operation')
      w(files.length)
      for (const f of files) ids(s.companionsOf(f).map(f => files.indexOf(f)))
    }
  } else throw Error('Bad mode')
  if (at !== b.length) throw Error('Trailing fixture')
  process.stdout.write(Buffer.concat(out))
})().catch(e => { console.error(e); process.exitCode = 1 })
