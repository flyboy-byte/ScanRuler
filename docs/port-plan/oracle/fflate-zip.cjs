// SPDX-License-Identifier: AGPL-3.0-only
// Frozen original's zip reader: `fflate` at the version pinned by the original's package-lock.json,
// called exactly as src/core/parsers/threemf.ts calls it (unzipSync with a name filter).
//   node fflate-zip.cjs gen <seed> <count>   stdout: u32 count, then per archive u32 length + bytes
//   node fflate-zip.cjs unzip                stdin: that framing; stdout per archive:
//       u8 status (1 ok, 0 error, 2 allocation failed, 3 output over 64 MiB)
//       ok:    u32 entries, per entry u32 name length + utf8 name + u32 data length + data
//       other: u32 error code (255 none) + u32 message length + utf8 message
const { readFileSync } = require('node:fs')
const { execFileSync } = require('node:child_process')
const path = require('node:path')
const root = path.resolve(__dirname, '../../..')
const lock = JSON.parse(execFileSync('git', ['show', '2cd89c6769a10c27ee7754b477ada521c9ef5cb7:package-lock.json'], { cwd: root, encoding: 'utf8', maxBuffer: 1 << 28 }))
const pinned = lock.packages['node_modules/fflate'].version
const fflate = require(path.join(root, 'node_modules/fflate'))
const installed = require(path.join(root, 'node_modules/fflate/package.json')).version
if (pinned !== installed) throw new Error(`fflate ${installed} installed, original pins ${pinned}`)
const frame = (chunks) => Buffer.concat(chunks)
const u32 = (n) => { const b = Buffer.alloc(4); b.writeUInt32LE(n >>> 0); return b }
const LIMIT = 64 * 1024 * 1024

if (process.argv[2] === 'gen') {
  let s = (Number(process.argv[3]) >>> 0) || 1
  const rnd = () => { s = (s + 0x6D2B79F5) >>> 0; let t = s; t = Math.imul(t ^ (t >>> 15), t | 1); t ^= t + Math.imul(t ^ (t >>> 7), t | 61); return ((t ^ (t >>> 14)) >>> 0) / 4294967296 }
  const pick = (a) => a[Math.floor(rnd() * a.length)]
  const names = ['3D/3dmodel.model', '3D/Objects/a.model', '_rels/.rels', 'Metadata/x.config', 'Textures/t.png', 'readme.txt', 'A.MODEL', 'dir/', 'ü.model']
  const body = () => {
    const n = Math.floor(rnd() * 400)
    const out = new Uint8Array(n)
    const mode = rnd()
    for (let i = 0; i < n; i++) out[i] = mode < 0.4 ? 65 + (i % 7) : mode < 0.7 ? (i * 31) & 255 : Math.floor(rnd() * 256)
    return out
  }
  const count = Number(process.argv[4])
  const out = [u32(count)]
  for (let k = 0; k < count; k++) {
    const files = {}
    const n = 1 + Math.floor(rnd() * 4)
    for (let j = 0; j < n; j++) files[pick(names)] = [body(), { level: pick([0, 1, 6, 9]), mtime: new Date(2020, 0, 1) }]
    const z = Buffer.from(fflate.zipSync(files))
    out.push(u32(z.length), z)
  }
  process.stdout.write(frame(out))
} else {
  const data = readFileSync(0)
  let at = 0
  const count = data.readUInt32LE(at); at += 4
  const out = []
  for (let i = 0; i < count; i++) {
    const len = data.readUInt32LE(at); at += 4
    const bytes = new Uint8Array(data.subarray(at, at + len)); at += len
    try {
      const files = fflate.unzipSync(bytes, { filter: (f) => /\.(model|rels|config)$/i.test(f.name) })
      const entries = Object.entries(files)
      if (entries.some(([, v]) => v.length > LIMIT)) { out.push(Buffer.from([3]), u32(255), u32(0)); continue }
      const parts = [Buffer.from([1]), u32(entries.length)]
      for (const [name, v] of entries) {
        const nb = Buffer.from(name, 'utf8')
        parts.push(u32(nb.length), nb, u32(v.length), Buffer.from(v.buffer, v.byteOffset, v.length))
      }
      out.push(...parts)
    } catch (e) {
      const m = Buffer.from(String(e && e.message !== undefined ? e.message : e), 'utf8')
      const alloc = /allocation failed|Invalid typed array length|Array buffer allocation/i.test(m.toString())
      out.push(Buffer.from([alloc ? 2 : 0]), u32(typeof e.code === 'number' ? e.code : 255), u32(m.length), m)
    }
  }
  process.stdout.write(frame(out))
}
