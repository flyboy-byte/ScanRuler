// SPDX-License-Identifier: AGPL-3.0-only
// Source-only pixel capture; no candidate/Rust output participates.
import assert from 'node:assert/strict'
import { execFileSync } from 'node:child_process'
import { createHash } from 'node:crypto'
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs'
import { createServer } from 'node:http'
import { fileURLToPath } from 'node:url'
import { resolve, dirname } from 'node:path'
import { build } from 'esbuild'
import puppeteer from 'puppeteer-core'
const outputArgument = process.argv.indexOf('--out')
const root = fileURLToPath(new URL('../../../', import.meta.url))
const ref = '2cd89c6769a10c27ee7754b477ada521c9ef5cb7'
assert.equal(Number(process.versions.node.split('.')[0]), 22)
const source = path => execFileSync('git', ['show', ref + ':' + path], { cwd: root, encoding: 'utf8' })
const bundle = await build({ stdin: { contents: source('src/app/scanFiles.ts'), resolveDir: resolve(root, 'src/app'), loader: 'ts' },
  bundle: true, write: false, platform: 'browser', format: 'iife', globalName: 'ScanRulerSource',
  plugins: [{ name: 'pinned-formats', setup(b) {
    b.onResolve({ filter: /^\.\.\/core\/formats$/ }, () => ({ path: 'src/core/formats.ts', namespace: 'pinned' }))
    b.onLoad({ filter: /.*/, namespace: 'pinned' }, args => ({ contents: source(args.path), loader: 'ts' }))
  } }] })
const code = bundle.outputFiles[0].contents
const hash = b => createHash('sha256').update(b).digest('hex')
const manifest = JSON.parse(readFileSync(resolve(root, 'docs/port-plan/fixtures/image-audit/manifest.json')))
const report = { contract: 'image.audit.source.v1', reference_sha: ref,
  candidate_sha: execFileSync('git', ['rev-parse', 'HEAD'], { cwd: root, encoding: 'utf8' }).trim(),
  node: process.version, bundle_sha256: hash(code), scan_files_sha256: hash(source('src/app/scanFiles.ts')),
  formats_sha256: hash(source('src/core/formats.ts')), seeds: manifest.seeds, cases: [] }
const server = createServer((req,res) => { res.writeHead(200, { 'content-type': 'text/html' }); res.end('<!doctype html><title>Pinned image pixel audit</title>') })
let browser
try {
  await new Promise(r => server.listen(0, '127.0.0.1', r))
  browser = await puppeteer.launch({ executablePath: process.env.CHROME ?? '/opt/google/chrome/chrome', headless: true, args: ['--no-sandbox','--disable-dev-shm-usage'] })
  const page = await browser.newPage(), errors = []
  page.on('pageerror', e => errors.push(String(e)))
  await page.goto(`http://127.0.0.1:${server.address().port}/`)
  await page.addScriptTag({ content: Buffer.from(code).toString('utf8') })
  report.browser = await browser.version()
  report.platform = await page.evaluate(() => ({ userAgent: navigator.userAgent, offscreenCanvas: typeof OffscreenCanvas !== 'undefined' }))
  for (const c of manifest.cases) {
    const bytes = readFileSync(resolve(root, 'docs/port-plan/fixtures/image-audit', c.id + '.bin'))
    assert.equal(hash(bytes), c.sha256)
    const result = await page.evaluate(async ({ id, input }) => {
      const result = await Promise.race([
        ScanRulerSource.readCompanions([{ name: id + '.img', bytes: Uint8Array.from(input) }]),
        new Promise((_, reject) => setTimeout(() => reject(Error('source decode timeout')), 10000)),
      ])
      if (!result || result.materials.length !== 0 || result.images.length > 1) throw Error('unexpected source state')
      return result.images.length === 0 ? null : { width: result.images[0].width, height: result.images[0].height, rgba: Array.from(result.images[0].rgba) }
    }, { id: c.id, input: Array.from(bytes) })
    report.cases.push({ id: c.id, input_sha256: c.sha256, source: result === null ? null : { width: result.width, height: result.height, rgba_hex: Buffer.from(result.rgba).toString('hex'), rgba_sha256: hash(Buffer.from(result.rgba)) } })
    console.log(c.id + ': complete source pixel/omission captured')
  }
  assert.deepEqual(errors, [])
  report.page_errors = errors; report.status = 'captured-source-only'
  const outputPath = outputArgument >= 0 ? resolve(process.argv[outputArgument + 1]) : resolve(root, 'rust/target/image-audit-source/report.json')
  mkdirSync(dirname(outputPath), { recursive: true })
  writeFileSync(outputPath, JSON.stringify(report, null, 2) + '\n')
} finally {
  if (browser) await browser.close()
  server.closeAllConnections(); await new Promise(r => server.close(r))
}
