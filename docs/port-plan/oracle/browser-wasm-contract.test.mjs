// SPDX-License-Identifier: AGPL-3.0-only
import { test } from 'node:test'
import assert from 'node:assert/strict'
import { compareOutput, plyPacket, successfulOutput } from './browser-wasm-contract.mjs'
import { validateBrowserRoster, selectBrowserCases, nativeFileNames, verifyNativeCaseFiles } from './browser-wasm-roster.mjs'
const tls = () => { const b = Buffer.alloc(33); b[0] = 1; [0, 0, 1, 3].forEach((v, i) => b.writeDoubleLE(v, 1 + i * 8)); return b }
test('valid numerical and exact graph results', () => {
  assert.equal(compareOutput(1, tls(), tls()), 0)
  assert.equal(compareOutput(1, Buffer.from([0]), Buffer.from([0])), 0)
  assert.equal(compareOutput(2, Buffer.from([0, 1, 2]), Buffer.from([0, 1, 2])), 0)
})
for (const [name, change] of [
  ['changed value', b => b.writeDoubleLE(42, 1)],
  ['missing value', b => b.subarray(0, -8)],
  ['nonfinite', b => b.writeDoubleLE(NaN, 1)],
  ['changed presence', b => { b[0] = 0 }],
]) test('rejects ' + name, () => {
  const base = tls(), actual = Buffer.from(base), returned = change(actual)
  assert.throws(() => compareOutput(1, base, Buffer.isBuffer(returned) ? returned : actual))
})
test('rejects empty, malformed and unknown codecs', () => {
  assert.throws(() => compareOutput(1, Buffer.alloc(0), Buffer.alloc(0)))
  assert.throws(() => compareOutput(1, Buffer.from([0, 0]), Buffer.from([0, 0])))
  assert.throws(() => compareOutput(99, tls(), tls()))
})
test('changed discrete graph byte cannot pass', () => assert.throws(() => compareOutput(2, Buffer.from([0, 1]), Buffer.from([0, 2]))))
test('changed marked-plane zero sign/region cannot pass', () => {
  const b = Buffer.alloc(10 + 144 + 12 + 4 + 720); b.writeUInt32LE(5, 1); b.write('plane', 5); b.writeUInt32LE(1, 10 + 144 + 8)
  const signed = Buffer.from(b); signed.writeDoubleLE(-0, 10)
  assert.throws(() => compareOutput(3, b, signed))
  const region = Buffer.from(b); region.writeUInt32LE(4, 10 + 144 + 12)
  assert.throws(() => compareOutput(3, b, region))
  assert.equal(compareOutput(3, b, b), 0)
})

test('committed Chrome evidence retains full corpus and module identities', async () => {
  const { readFileSync } = await import('node:fs')
  const { createHash } = await import('node:crypto')
  const base = new URL('../evidence/browser-wasm-2026-11-01/', import.meta.url)
  const bytes = readFileSync(new URL('report.json', base)), report = JSON.parse(bytes)
  const manifest = JSON.parse(readFileSync(new URL('manifest.json', base)))
  assert.equal(createHash('sha256').update(bytes).digest('hex'), manifest.report_sha256)
  assert.equal(report.status, 'passed'); assert.equal(report.runtime, 'chrome-wasm')
  assert.equal(report.candidate_sha, manifest.candidate_sha); assert.equal(report.reference_sha, manifest.reference_sha)
  assert.deepEqual(report.browser.imports, []); assert.deepEqual(report.page_errors, [])
  assert.equal(report.module_sha256, manifest.module_sha256)
  assert.equal(report.browser.moduleSha256, manifest.module_sha256)
  assert.match(report.browser.version, /^Chrome\//); assert.match(report.node, /^v22\./)
  assert.equal(report.cases.length, 114); assert.equal(report.successful_value_cases, 90); assert.equal(report.source_refusal_cases, 24)
  const definitions = JSON.parse(readFileSync(new URL('../ci/wasm-suites.json', import.meta.url))).suites
  const expected = []
  for (const def of definitions.filter(s => s.mode <= 3)) {
    const unit = JSON.parse(readFileSync(new URL(`../plan/units/${def.unit}.json`, import.meta.url)))
    for (const id of unit.evidence.find(e => e.kind === 'differential').case_ids) expected.push(def.unit + '/' + id)
  }
  assert.deepEqual(report.cases.map(c => c.unit + '/' + c.id).sort(), expected.sort())
  assert.equal(new Set(expected).size, 114)
  for (const c of report.cases) for (const key of ['input_sha256', 'original_sha256', 'native_sha256', 'wasm_sha256']) assert.match(c[key], /^[a-f0-9]{64}$/)
})

test('a mismatch retains complete raw inputs and all outputs before failing', async () => {
  const { mkdtempSync, readFileSync, rmSync } = await import('node:fs')
  const { tmpdir } = await import('node:os')
  const { join } = await import('node:path')
  const { retainAndCompare } = await import('./browser-wasm-contract.mjs')
  const dir = mkdtempSync(join(tmpdir(), 'scanruler-wasm-mismatch-'))
  try {
    const files = { 'input.bin': Buffer.from([9]), 'original.bin': tls(), 'rust.bin': tls() }
    const wrong = tls(); wrong.writeDoubleLE(42, 1)
    await assert.rejects(retainAndCompare(dir, 1, files, wrong))
    for (const [name, bytes] of Object.entries({ ...files, 'wasm.bin': wrong })) assert.deepEqual(readFileSync(join(dir, name)), bytes)
  } finally { rmSync(dir, { recursive: true, force: true }) }
})

function ply() {
  const b = Buffer.alloc(28)
  b.write('PLY1'); b.writeUInt32LE(3, 8)
  b.writeUInt32LE(0x7fc12345, 16); b.writeFloatLE(-0, 20); b.writeFloatLE(Infinity, 24)
  return plyPacket(b, Buffer.from('Reading PLY…\n'))
}
test('PLY retains exact nonfinite bits, zero signs and progress', () => {
  const b = ply()
  assert.equal(compareOutput(4, b, b), 0)
  for (const offset of [5, 9, 13, 21, 25, b.length - 1]) {
    const changed = Buffer.from(b); changed[offset] ^= 1
    assert.throws(() => compareOutput(4, b, changed))
  }
  assert.throws(() => compareOutput(4, b, b.subarray(0, -1)))
})
test('PLY cannot accept empty or malformed value/refusal envelopes', () => {
  const error = plyPacket(Buffer.alloc(0), Buffer.from('Not a valid PLY file.\n'))
  assert.equal(compareOutput(4, error, error), 0)
  for (const b of [Buffer.alloc(0), plyPacket(Buffer.alloc(0), Buffer.alloc(0)), plyPacket(Buffer.from('bad'), Buffer.alloc(0))]) {
    assert.throws(() => compareOutput(4, b, b))
  }
  const unknown = Buffer.from(error); unknown[0] = 1
  assert.throws(() => compareOutput(4, unknown, unknown))
  const missing = ply(); missing.writeUInt32LE(1000, 13)
  assert.throws(() => compareOutput(4, missing, missing))
  assert.throws(() => plyPacket(Buffer.alloc(0), undefined))
})
test('browser roster preserves prior cases and only the declared new workload exclusion', async () => {
  const { readFileSync } = await import('node:fs')
  const defs = JSON.parse(readFileSync(new URL('../ci/wasm-suites.json', import.meta.url))).suites
  validateBrowserRoster(defs)
  let count = 0
  for (const def of defs) {
    const unit = JSON.parse(readFileSync(new URL(`../plan/units/${def.unit}.json`, import.meta.url)))
    const ids = unit.evidence.find(e => e.kind === 'differential').case_ids
    count += selectBrowserCases(def, ids).length
    assert.throws(() => selectBrowserCases(def, ids.slice(1)))
    if (def.excluded) {
      assert.throws(() => selectBrowserCases({ ...def, excluded: [] }, ids))
      assert.throws(() => selectBrowserCases({ ...def, excluded: [{ id: 'le-vertex-progress', reason: '' }] }, ids))
      assert.throws(() => selectBrowserCases({ ...def, excluded: [{ id: ids[0], reason: 'unreviewed' }] }, ids))
    }
  }
  assert.equal(count, 2211)
  assert.throws(() => validateBrowserRoster(defs.slice(0, 5)))
  validateBrowserRoster(defs.slice(0, 5), 'browser.wasm.corpus.v2')
  assert.throws(() => validateBrowserRoster(defs.slice(0, 7)))
  validateBrowserRoster(defs.slice(0, 7), 'browser.wasm.corpus.v3')
  assert.throws(() => validateBrowserRoster(defs.slice(0, 8)))
  validateBrowserRoster(defs.slice(0, 8), 'browser.wasm.corpus.v4')
  assert.throws(() => validateBrowserRoster(defs.slice(1)))
})
test('PLY mismatches retain raw sidecars and actual Wasm before failing', async () => {
  const { mkdtempSync, readFileSync, rmSync } = await import('node:fs')
  const { tmpdir } = await import('node:os')
  const { join } = await import('node:path')
  const { retainAndCompare } = await import('./browser-wasm-contract.mjs')
  const dir = mkdtempSync(join(tmpdir(), 'scanruler-ply-wasm-mismatch-'))
  try {
    const b = ply(), n = b.readUInt32LE(1)
    const files = { 'input.bin': Buffer.from('input'), 'original.bin': b.subarray(5, 5 + n), 'rust.bin': b.subarray(5, 5 + n),
      'original.progress-or-error.txt': b.subarray(9 + n), 'rust.progress-or-error.txt': b.subarray(9 + n) }
    const wrong = Buffer.from(b); wrong[21] ^= 1
    await assert.rejects(retainAndCompare(dir, 4, files, wrong))
    for (const [name, bytes] of Object.entries({ ...files, 'wasm.bin': wrong })) assert.deepEqual(readFileSync(join(dir, name)), bytes)
  } finally { rmSync(dir, { recursive: true, force: true }) }
})
test('missing, stale and corrupt PLY sidecars fail before browser execution', async () => {
  const { createHash } = await import('node:crypto')
  const files = Object.fromEntries(nativeFileNames(4).map(name => [name, Buffer.from(name)]))
  const metadata = { files: Object.fromEntries(Object.entries(files).map(([name, b]) => [name,
    { bytes: b.length, sha256: createHash('sha256').update(b).digest('hex') }])) }
  verifyNativeCaseFiles(4, files, metadata)
  for (const name of ['original.progress-or-error.txt', 'rust.progress-or-error.txt']) {
    const missing = { ...files }; delete missing[name]
    assert.throws(() => verifyNativeCaseFiles(4, missing, metadata))
    assert.throws(() => verifyNativeCaseFiles(4, { ...files, [name]: Buffer.from('changed') }, metadata))
    const absent = structuredClone(metadata); delete absent.files[name]
    assert.throws(() => verifyNativeCaseFiles(4, files, absent))
    const stale = structuredClone(metadata); stale.files[name].sha256 = '0'.repeat(64)
    assert.throws(() => verifyNativeCaseFiles(4, files, stale))
    const badLength = structuredClone(metadata); badLength.files[name].bytes++
    assert.throws(() => verifyNativeCaseFiles(4, files, badLength))
  }
})
for (const [folder, total, successes, refusals, protocol, excluded] of [
  ['browser-wasm-ply-2026-11-02', 341, 287, 54, 13, ['io.ply.binary/le-vertex-progress']],
  ['browser-wasm-obj-2026-11-02', 564, 482, 82, 18, ['io.ply.binary/le-vertex-progress', 'io.obj.mesh/line-progress', 'io.obj.mesh/short-unicode-line-skips-progress']],
  ['browser-wasm-files-2026-11-02', 651, 569, 82, 24, ['io.ply.binary/le-vertex-progress', 'io.obj.mesh/line-progress', 'io.obj.mesh/short-unicode-line-skips-progress']],
  ['browser-wasm-sphere-2026-11-03', 2211, 1193, 1018, 30, ['io.ply.binary/le-vertex-progress', 'io.obj.mesh/line-progress', 'io.obj.mesh/short-unicode-line-skips-progress']],
]) test(folder + ': complete committed Chrome proof replays all source/native/Wasm results', async () => {
  const { readFileSync } = await import('node:fs')
  const { createHash } = await import('node:crypto')
  const { gunzipSync } = await import('node:zlib')
  const base = new URL(`../evidence/${folder}/`, import.meta.url)
  const read = name => readFileSync(new URL(name, base))
  const hash = b => createHash('sha256').update(b).digest('hex')
  const manifest = JSON.parse(read('manifest.json')), report = JSON.parse(read('report.json'))
  assert.equal(hash(read('report.json')), manifest.report_sha256)
  assert.equal(hash(read('corpus.json')), manifest.corpus_sha256)
  assert.equal(hash(read('raw.json.gz')), manifest.raw_gzip_sha256)
  const bytes = gunzipSync(read('raw.json.gz'))
  assert.equal(hash(bytes), manifest.raw_decoded_sha256)
  const raw = JSON.parse(bytes), definitions = JSON.parse(read('corpus.json')).suites
  validateBrowserRoster(definitions, JSON.parse(read('corpus.json')).contract)
  assert.equal(report.status, 'passed'); assert.equal(report.runtime, 'chrome-wasm')
  assert.equal(report.candidate_sha, manifest.candidate_sha); assert.equal(report.reference_sha, manifest.reference_sha)
  assert.equal(report.browser.moduleSha256, manifest.module_sha256); assert.equal(report.module_sha256, manifest.module_sha256)
  assert.deepEqual(report.browser.imports, []); assert.deepEqual(report.page_errors, [])
  assert.match(report.browser.version, /^Chrome\//); assert.match(report.node, /^v22\./)
  assert.equal(report.cases.length, total); assert.equal(raw.length, total)
  assert.equal(report.successful_value_cases, successes); assert.equal(report.source_refusal_cases, refusals)
  assert.equal(report.negative_protocol_checks.length, protocol)
  assert.deepEqual(report.excluded_native_cases, manifest.excluded_native_cases)
  assert.deepEqual(report.excluded_native_cases.map(c => c.unit + '/' + c.id), excluded)
  const expected = manifest.units.flatMap(u => u.case_ids.map(id => u.unit + '/' + id)).sort()
  assert.equal(new Set(expected).size, total)
  assert.deepEqual(manifest.units.map(u => [u.unit, u.mode, u.case_ids.length]), definitions.map(d => [d.unit, d.mode, d.cases]))
  for (const d of definitions) {
    const unit = JSON.parse(readFileSync(new URL(`../plan/units/${d.unit}.json`, import.meta.url)))
    const ids = unit.evidence.find(e => e.kind === 'differential').case_ids
    assert.deepEqual(manifest.units.find(u => u.unit === d.unit).case_ids, selectBrowserCases(d, ids))
  }
  assert.deepEqual(report.cases.map(c => c.unit + '/' + c.id).sort(), expected)
  assert.deepEqual(raw.map(c => c.unit + '/' + c.id).sort(), expected)
  for (const item of raw) {
    const c = report.cases.find(c => c.unit === item.unit && c.id === item.id)
    const mode = definitions.find(d => d.unit === c.unit).mode
    const files = Object.fromEntries(Object.entries(item.files).map(([name, hex]) => [name, Buffer.from(hex, 'hex')]))
    const original = mode === 4 ? files['original.packet.bin'] : files['original.bin']
    const native = mode === 4 ? files['rust.packet.bin'] : files['rust.bin']
    if (mode === 4) {
      assert.deepEqual(original, plyPacket(files['original.bin'], files['original.progress-or-error.txt']))
      assert.deepEqual(native, plyPacket(files['rust.bin'], files['rust.progress-or-error.txt']))
    }
    for (const [bytes, key] of [[files['input.bin'], 'input_sha256'], [original, 'original_sha256'],
      [native, 'native_sha256'], [files['wasm.bin'], 'wasm_sha256']]) assert.equal(hash(bytes), c[key])
    if (mode === 8) { assert.deepEqual(files['input.bin'], files['original.after.bin']); assert.deepEqual(files['input.bin'], files['rust.after.bin']); assert.equal(files['descriptor.tsv'].toString().split('\t')[0], c.id); assert.equal(files['original.stderr.txt'].length, 0) }
    compareOutput(mode, original, native); compareOutput(mode, original, files['wasm.bin']); compareOutput(mode, native, files['wasm.bin'])
    assert.equal(c.result, successfulOutput(mode, files['wasm.bin']) ? 'successful_values' : 'source_refusal')
  }
})

test('OBJ complete original/native states validate with exact values, signed zeros, notes and progress', async () => {
  const { readFileSync } = await import('node:fs')
  const { successfulOutput } = await import('./browser-wasm-contract.mjs')
  let count = 0
  for (const [mode, name] of [[5, 'obj-mesh'], [6, 'obj-texture']]) {
    const proof = JSON.parse(readFileSync(new URL(`../evidence/2026-11-02-${name}.json`, import.meta.url)))
    for (const c of proof.cases) {
      const original = Buffer.from(c.files['original.bin'].hex, 'hex'), native = Buffer.from(c.files['rust.bin'].hex, 'hex')
      assert.equal(compareOutput(mode, original, native), 0, c.id)
      assert.equal(successfulOutput(mode, original), mode === 6 || c.success)
      const changed = Buffer.from(original); changed[changed.length - 1] ^= 1
      assert.throws(() => compareOutput(mode, original, changed), c.id + ': changed final state')
      assert.throws(() => compareOutput(mode, original, original.subarray(0, -1)), c.id + ': missing state')
      assert.throws(() => compareOutput(mode, Buffer.concat([original, Buffer.from([0])]), Buffer.concat([original, Buffer.from([0])])), c.id + ': trailing data')
      if (mode === 5 && c.id === 'triangle') {
        const signed = Buffer.from(original); signed.writeFloatLE(-0, 9)
        assert.throws(() => compareOutput(mode, original, signed), 'float32 zero sign must remain exact')
        const malformed = Buffer.from(original); malformed.writeUInt32LE(0xffffffff, 5)
        assert.throws(() => compareOutput(mode, malformed, malformed), 'counts bounded by packet')
      }
      count++
    }
  }
  assert.equal(count, 225)
})
test('OBJ helper kinds, tags, UTF-8 and incomplete results cannot pass even when paired', () => {
  for (const hex of [
    '', '4f424a31', '4f424a3101', '4f54583100', '4f5458310503',
    '4f5458310101000000ff', '4f54583102ffffffff', '4f545831050201000000ff00000000',
    '4f424a31020000000000000000',
  ]) {
    const b = Buffer.from(hex, 'hex')
    assert.throws(() => compareOutput(b.subarray(0, 4).toString() === 'OBJ1' ? 5 : 6, b, b))
  }
})
test('OBJ mismatch retains exact input and three outputs before assertion', async () => {
  const { mkdtempSync, readFileSync, rmSync } = await import('node:fs')
  const { tmpdir } = await import('node:os')
  const { join } = await import('node:path')
  const { retainAndCompare } = await import('./browser-wasm-contract.mjs')
  const dir = mkdtempSync(join(tmpdir(), 'scanruler-obj-wasm-'))
  try {
    const output = Buffer.from('4f5458310403000000010203', 'hex')
    const files = { 'input.bin': Buffer.from('independent input'), 'original.bin': output, 'rust.bin': output }
    const changed = Buffer.from(output); changed[changed.length - 1] ^= 1
    await assert.rejects(retainAndCompare(dir, 6, files, changed))
    for (const [name, bytes] of Object.entries({ ...files, 'wasm.bin': changed })) assert.deepEqual(readFileSync(join(dir, name)), bytes)
  } finally { rmSync(dir, { recursive: true, force: true }) }
})

test('complete file classifications, identity traces and byte copies are exact', async () => {
  const { readFileSync } = await import('node:fs')
  const proof = JSON.parse(readFileSync(new URL('../evidence/2026-11-02-obj-files.json', import.meta.url)))
  assert.equal(proof.cases.length, 87)
  for (const c of proof.cases) {
    const source = Buffer.from(c.files['original.bin'].hex, 'hex'), native = Buffer.from(c.files['rust.bin'].hex, 'hex')
    assert.equal(compareOutput(7, source, native), 0, c.id)
    assert.equal(successfulOutput(7, source), true)
    const changed = Buffer.from(source); changed[changed.length - 1] ^= 1
    assert.throws(() => compareOutput(7, source, changed), c.id + ': changed last state/byte')
    assert.throws(() => compareOutput(7, source, source.subarray(0, -1)), c.id + ': missing state')
    const trailing = Buffer.concat([source, Buffer.from([0])])
    assert.throws(() => compareOutput(7, trailing, trailing), c.id + ': trailing state')
    if (c.id === 'format-0') {
      const flag = Buffer.from(source); flag[9] = 2
      assert.throws(() => compareOutput(7, flag, flag), 'classifier flags must be boolean')
    }
  }
})
test('file-state kinds, counts, references and invalid UTF8 cannot pass even paired', () => {
  for (const hex of ['', '464c5331', '464c533102', '464c533101ffffffff',
    '464c53310001000000ff', '464c53310101000000ff0000000000000000',
    '464c53310101000000010000000000000000']) {
    const b = Buffer.from(hex, 'hex'); assert.throws(() => compareOutput(7, b, b))
  }
  const zeroOps = Buffer.from('464c53310100000000', 'hex')
  assert.equal(compareOutput(7, zeroOps, zeroOps), 0)
  // One bundle chooses file0, yet its association snapshot owns zero files.
  const invalidId = Buffer.from('464c53310101000000010000000000000000', 'hex')
  assert.throws(() => compareOutput(7, invalidId, invalidId))
})
test('file-state mismatch retains exact independent input and all three outputs', async () => {
  const { mkdtempSync, readFileSync, rmSync } = await import('node:fs')
  const { tmpdir } = await import('node:os')
  const { join } = await import('node:path')
  const { retainAndCompare } = await import('./browser-wasm-contract.mjs')
  const dir = mkdtempSync(join(tmpdir(), 'scanruler-files-wasm-'))
  try {
    const source = Buffer.from('464c53310100000000', 'hex'), wrong = Buffer.from(source); wrong[4] = 255
    const files = { 'input.bin': Buffer.from('authored input'), 'original.bin': source, 'rust.bin': source }
    await assert.rejects(retainAndCompare(dir, 7, files, wrong))
    for (const [name, bytes] of Object.entries({ ...files, 'wasm.bin': wrong })) assert.deepEqual(readFileSync(join(dir, name)), bytes)
  } finally { rmSync(dir, { recursive: true, force: true }) }
})

test('sphere preserves complete native corpus under its separately frozen budget', async () => {
  const {readFileSync}=await import('node:fs'),{gunzipSync}=await import('node:zlib')
  const proof=JSON.parse(gunzipSync(readFileSync(new URL('../evidence/2026-11-03-sphere.json.gz',import.meta.url))))
  let successes=0
  for(const c of proof.cases){const original=Buffer.from(c.files['original.bin'].hex,'hex'),native=Buffer.from(c.files['rust.bin'].hex,'hex');compareOutput(8,original,native);successes+=Number(successfulOutput(8,native))}
  assert.equal(proof.cases.length,1560);assert.equal(successes,624)
  const original=Buffer.from(proof.cases.find(c=>c.files['original.bin'].hex.startsWith('01')).files['original.bin'].hex,'hex')
  const within=Buffer.from(original);within.writeDoubleLE(original.readDoubleLE(1)+1e-11,1);compareOutput(8,original,within)
  const old=tls(),changedOld=tls();changedOld.writeDoubleLE(old.readDoubleLE(1)+1e-11,1);assert.throws(()=>compareOutput(1,old,changedOld),'older tighter budget must not change')
  for(const offset of [1,69]){const changed=Buffer.from(original);if(offset===1)changed.writeDoubleLE(37,offset);else changed.writeUInt32LE(37,offset);assert.throws(()=>compareOutput(8,original,changed))}
  const sign=Buffer.from(original);sign.writeDoubleLE(-0,1);assert.throws(()=>compareOutput(8,original,sign))
  const nonfinite=Buffer.from(original);nonfinite.writeDoubleLE(NaN,1);assert.throws(()=>compareOutput(8,original,nonfinite))
})
test('malformed sphere result counts/presence/trailing bytes fail even against themselves',()=>{
  for(const b of [Buffer.alloc(0),Buffer.from([2]),Buffer.from([0,0]),Buffer.from([1])])assert.throws(()=>compareOutput(8,b,b))
  const valid=Buffer.alloc(70);valid[0]=1
  assert.equal(compareOutput(8,valid,valid),0)
  const count=Buffer.from(valid);count.writeUInt32LE(0xffffffff,65)
  const present=Buffer.from(valid);present[69]=2
  for(const b of [count,present,Buffer.concat([valid,Buffer.from([0])])])assert.throws(()=>compareOutput(8,b,b))
})
test('sphere native descriptors and post-input metadata are mandatory before Chrome',async()=>{
  const {createHash}=await import('node:crypto')
  const files=Object.fromEntries(nativeFileNames(8).map(name=>[name,Buffer.from(name)]))
  const metadata={files:Object.fromEntries(Object.entries(files).map(([name,b])=>[name,{bytes:b.length,sha256:createHash('sha256').update(b).digest('hex')}]))}
  verifyNativeCaseFiles(8,files,metadata)
  for(const name of nativeFileNames(8)){
    const missing={...files};delete missing[name];assert.throws(()=>verifyNativeCaseFiles(8,missing,metadata))
    assert.throws(()=>verifyNativeCaseFiles(8,{...files,[name]:Buffer.from('corrupt')},metadata))
  }
})
