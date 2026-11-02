// SPDX-License-Identifier: AGPL-3.0-only
// Temporary original/native/browser-Wasm comparison coordinator, not product logic.
import assert from 'node:assert/strict'
import { createHash } from 'node:crypto'
import { execFileSync } from 'node:child_process'
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs'
import { createServer } from 'node:http'
import { resolve, basename } from 'node:path'
import { fileURLToPath } from 'node:url'
import { compareOutput, runPacket, negativePackets } from './browser-wasm-contract.mjs'
const root = fileURLToPath(new URL('../../../', import.meta.url))
const smoke = process.argv.includes('--node-smoke')
if (!smoke && Number(process.versions.node.split('.')[0]) !== 22) throw Error('Hosted browser runner requires Node 22')
const target = resolve(root, 'rust/target'), out = resolve(target, smoke ? 'node-wasm-smoke' : 'browser-wasm')
mkdirSync(out, { recursive: true })
const read = p => readFileSync(p), json = p => JSON.parse(read(p)), hash = b => createHash('sha256').update(b).digest('hex')
const candidate = execFileSync('git', ['rev-parse', 'HEAD'], { cwd: root, encoding: 'utf8' }).trim()
const reference = json(resolve(root, 'docs/port-plan/reference-pin.json')).commit
const modulePath = resolve(target, 'wasm32-unknown-unknown/debug/port_wasm_probe.wasm'), moduleBytes = read(modulePath)
assert.equal(read(modulePath.replace(/\.wasm$/, '.candidate.txt')).toString().trim(), candidate, 'stale Wasm build')
const native = json(resolve(target, 'ci/native-evidence.json'))
assert.equal(native.candidate_sha, candidate, 'stale native evidence')
assert.equal(native.reference_sha, reference)
const defs = json(resolve(root, 'docs/port-plan/ci/wasm-suites.json')).suites
assert.equal(defs.length, 3)
assert.equal(new Set(defs.map(s => s.mode)).size, 3)
const unitPaths = resolve(root, 'docs/port-plan/plan/units')
const cases = []
for (const def of defs) {
  const unit = json(resolve(unitPaths, def.unit + '.json'))
  const ids = unit.evidence.find(e => e.kind === 'differential').case_ids.slice().sort()
  assert.equal(ids.length, def.cases, 'declared browser case count changed')
  const suite = native.suites.find(s => s.unit === def.unit)
  assert.ok(suite, 'missing native suite')
  assert.deepEqual(suite.cases.map(c => c.id).slice().sort(), ids, 'dropped native cases')
  const run = basename(suite.run)
  assert.match(run, /^run-\d+$/)
  for (const id of ids) {
    const directory = resolve(target, def.directory, run, id), meta = suite.cases.find(c => c.id === id)
    const files = {}
    for (const name of ['input.bin', 'original.bin', 'rust.bin']) {
      const bytes = read(resolve(directory, name))
      assert.equal(hash(bytes), meta.files[name].sha256, 'corrupt or changed raw file: ' + id + '/' + name)
      assert.equal(bytes.length, meta.files[name].bytes)
      files[name] = bytes
    }
    cases.push({ def, id, files })
  }
}
assert.equal(cases.length, 114, 'empty/trimmed browser corpus')
const report = { contract: 'browser.wasm.comparison.v1', candidate_sha: candidate, reference_sha: reference,
  runtime: smoke ? 'diagnostic-node-wasm-not-browser' : 'chrome-wasm', node: process.version,
  rustc: read(modulePath.replace(/\.wasm$/, '.rustc.txt')).toString().trim(), module_sha256: hash(moduleBytes),
  native_manifest_sha256: hash(read(resolve(target, 'ci/native-evidence.json'))), memory_limit_bytes: 67108864,
  source_runtime: 'frozen pure-core Node oracle from independently executed native suites', cases: [] }
let browser, server
try {
  let call, negatives
  if (smoke) {
    const module = new WebAssembly.Module(moduleBytes)
    assert.deepEqual(WebAssembly.Module.imports(module), [], 'foreign imports prohibited')
    const e = new WebAssembly.Instance(module).exports, api = { memory: e.memory }
    for (const name of ['reset', 'push', 'run', 'length', 'word']) {
      const keys = Object.keys(e).filter(k => k.endsWith('scanruler_wasm_' + name))
      assert.equal(keys.length, 1, 'missing/ambiguous private export ' + name); api[name] = e[keys[0]]
    }
    globalThis.__scanrulerWasmAPI = api
    call = packet => runPacket(packet); negatives = negativePackets()
  } else {
    // This server exposes only the compiled module and an empty test page.
    // Original/native output files are never exposed to the browser or Rust.
    server = createServer((req, res) => {
      if (req.url === '/probe.wasm') { res.writeHead(200, { 'content-type': 'application/wasm' }); res.end(moduleBytes) }
      else if (req.url === '/') { res.writeHead(200, { 'content-type': 'text/html' }); res.end('<!doctype html><title>Rust Wasm comparison harness</title>') }
      else { res.writeHead(204); res.end() }
    })
    await new Promise(resolve => server.listen(0, '127.0.0.1', resolve))
    const { default: puppeteer } = await import('puppeteer-core')
    browser = await puppeteer.launch({ executablePath: process.env.CHROME ?? '/usr/bin/google-chrome', headless: true,
      args: ['--no-sandbox', '--disable-dev-shm-usage'], protocolTimeout: 120000 })
    const page = await browser.newPage(), errors = []
    page.on('pageerror', e => errors.push(String(e)))
    await page.goto(`http://127.0.0.1:${server.address().port}/`)
    const context = await page.evaluate(async () => {
      const bytes = await (await fetch('/probe.wasm')).arrayBuffer(), module = await WebAssembly.compile(bytes)
      const imports = WebAssembly.Module.imports(module)
      if (imports.length) throw Error('Wasm must not import foreign numeric/runtime functions')
      const e = (await WebAssembly.instantiate(module)).exports, api = { memory: e.memory }, symbols = {}
      for (const name of ['reset', 'push', 'run', 'length', 'word']) {
        const keys = Object.keys(e).filter(k => k.endsWith('scanruler_wasm_' + name))
        if (keys.length !== 1) throw Error('Missing/ambiguous private export: ' + name)
        api[name] = e[keys[0]]; symbols[name] = keys[0]
      }
      globalThis.__scanrulerWasmAPI = api
      const digest = await crypto.subtle.digest('SHA-256', bytes)
      return { imports, symbols, userAgent: navigator.userAgent,
        moduleSha256: Array.from(new Uint8Array(digest), x => x.toString(16).padStart(2, '0')).join('') }
    })
    assert.equal(context.moduleSha256, report.module_sha256, 'browser executed different module')
    report.browser = { version: await browser.version(), ...context }
    call = packet => page.evaluate(runPacket, packet)
    negatives = await page.evaluate(negativePackets)
    report.page_errors = errors
  }
  report.negative_protocol_checks = negatives
  for (const { def, id, files } of cases) {
    // Only mode and byte-for-byte fixture input cross the browser/Wasm boundary.
    const wasm = Buffer.from(await call({ mode: def.mode, input: Array.from(files['input.bin']) }))
    const source = files['original.bin'], nativeOutput = files['rust.bin']
    const errorNative = compareOutput(def.mode, source, nativeOutput)
    const errorWasm = compareOutput(def.mode, source, wasm)
    const errorPair = compareOutput(def.mode, nativeOutput, wasm)
    const directory = resolve(out, def.unit, id); mkdirSync(directory, { recursive: true })
    for (const [name, bytes] of Object.entries({ ...files, 'wasm.bin': wasm })) writeFileSync(resolve(directory, name), bytes)
    report.cases.push({ unit: def.unit, id, result: (def.mode === 1 ? wasm[0] === 1 : wasm[0] === 0) ? 'successful_values' : 'source_refusal', input_sha256: hash(files['input.bin']), original_sha256: hash(source),
      native_sha256: hash(nativeOutput), wasm_sha256: hash(wasm), bytes: wasm.length,
      max_abs_native: errorNative, max_abs_wasm: errorWasm, max_abs_native_wasm: errorPair })
    console.log(`${def.unit}/${id}: original/native/Wasm values + state pass`)
  }
  if (report.page_errors?.length) throw Error('Browser page errors: ' + report.page_errors.join('; '))
  report.successful_value_cases = report.cases.filter(c => c.result === 'successful_values').length
  report.source_refusal_cases = report.cases.length - report.successful_value_cases
  assert.equal(report.successful_value_cases, 90)
  assert.equal(report.source_refusal_cases, 24)
  report.status = 'passed'
} catch (error) {
  report.status = 'failed'; report.error = String(error); throw error
} finally {
  writeFileSync(resolve(out, 'report.json'), JSON.stringify(report, null, 2) + '\n')
  if (browser) await browser.close()
  if (server) { server.closeAllConnections(); await new Promise(resolve => server.close(resolve)) }
}
console.log(`${report.runtime}: ${report.cases.length} value/state comparisons; ${report.negative_protocol_checks.length} protocol checks`)
