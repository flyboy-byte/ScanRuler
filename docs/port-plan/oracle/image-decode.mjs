// SPDX-License-Identifier: AGPL-3.0-only
// The frozen original's decodeImage (src/app/scanFiles.ts at the pin, bundled with esbuild) run in
// actual Chrome, called through readCompanions exactly as the app does: createImageBitmap with
// colorSpaceConversion 'none' and premultiplyAlpha 'none', then canvas drawImage and getImageData.
//   node image-decode.mjs < batch > results
//   stdin:  u32 count, then per case u32 length + bytes
//   stdout: u32 count, then per case u8 status (1 decoded, 0 omitted, 2 timed out after 10 s),
//           and when decoded u32 width, u32 height, u32 rgba length + bytes
//   stderr: the Chrome version, once.
// Chrome comes from $CHROME or /opt/google/chrome/chrome. No Rust output participates.
import { execFileSync } from 'node:child_process'
import { readFileSync } from 'node:fs'
import { createServer } from 'node:http'
import { fileURLToPath } from 'node:url'
import { resolve } from 'node:path'
import { build } from 'esbuild'
import puppeteer from 'puppeteer-core'

const root = fileURLToPath(new URL('../../../', import.meta.url))
const pin = '2cd89c6769a10c27ee7754b477ada521c9ef5cb7'
const source = (path) => execFileSync('git', ['show', `${pin}:${path}`], { cwd: root, encoding: 'utf8' })
const bundle = await build({
  stdin: { contents: source('src/app/scanFiles.ts'), resolveDir: resolve(root, 'src/app'), loader: 'ts' },
  bundle: true, write: false, platform: 'browser', format: 'iife', globalName: 'ScanRulerSource',
  plugins: [{ name: 'pinned', setup(b) {
    b.onResolve({ filter: /^\.\.\/core\/formats$/ }, () => ({ path: 'src/core/formats.ts', namespace: 'pinned' }))
    b.onLoad({ filter: /.*/, namespace: 'pinned' }, (args) => ({ contents: source(args.path), loader: 'ts' }))
  } }],
})

const data = readFileSync(0)
let at = 0
const word = () => { const v = data.readUInt32LE(at); at += 4; return v }
const cases = []
for (let i = 0, n = word(); i < n; i++) { const len = word(); cases.push(data.subarray(at, at + len).toString('base64')); at += len }

const server = createServer((_, res) => { res.writeHead(200, { 'content-type': 'text/html' }); res.end('<!doctype html><title>decode</title>') })
await new Promise((r) => server.listen(0, '127.0.0.1', r))
const browser = await puppeteer.launch({ executablePath: process.env.CHROME ?? '/opt/google/chrome/chrome', headless: true, args: ['--no-sandbox', '--disable-dev-shm-usage'] })
const out = []
const u32 = (n) => { const b = Buffer.alloc(4); b.writeUInt32LE(n >>> 0); return b }
try {
  const page = await browser.newPage()
  const errors = []
  page.on('pageerror', (e) => errors.push(String(e)))
  await page.goto(`http://127.0.0.1:${server.address().port}/`)
  await page.addScriptTag({ content: Buffer.from(bundle.outputFiles[0].contents).toString('utf8') })
  process.stderr.write((await browser.version()) + '\n')
  out.push(u32(cases.length))
  for (let i = 0; i < cases.length; i += 64) {
    const results = await page.evaluate(async (batch) => {
      const fromB64 = (s) => Uint8Array.from(atob(s), (c) => c.charCodeAt(0))
      const toB64 = (u) => { let s = ''; for (let j = 0; j < u.length; j += 0x8000) s += String.fromCharCode(...u.subarray(j, j + 0x8000)); return btoa(s) }
      const res = []
      for (const b64 of batch) {
        let r
        try {
          r = await Promise.race([
            ScanRulerSource.readCompanions([{ name: 'case.png', bytes: fromB64(b64) }]),
            new Promise((_, rej) => setTimeout(() => rej(Error('timeout')), 10000)),
          ])
        } catch (e) { res.push({ s: 2 }); continue }
        if (r.images.length === 0) res.push({ s: 0 })
        else { const im = r.images[0]; res.push({ s: 1, w: im.width, h: im.height, p: toB64(im.rgba) }) }
      }
      return res
    }, cases.slice(i, i + 64))
    for (const r of results) {
      out.push(Buffer.from([r.s]))
      if (r.s === 1) { const p = Buffer.from(r.p, 'base64'); out.push(u32(r.w), u32(r.h), u32(p.length), p) }
    }
  }
  if (errors.length) throw new Error('page errors: ' + errors.join('; '))
} finally {
  await browser.close()
  server.closeAllConnections()
  await new Promise((r) => server.close(r))
}
process.stdout.write(Buffer.concat(out), () => process.exit(0))
