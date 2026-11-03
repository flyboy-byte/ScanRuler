// SPDX-License-Identifier: AGPL-3.0-only
// Test comparison only. No geometry, source-result injection or product runtime.
import assert from 'node:assert/strict'
import { validateFileOutput } from './browser-wasm-files.mjs'
// Envelope only, assembled after independent execution; never a Rust input.
export function plyPacket(payload, events) {
  assert.ok(Buffer.isBuffer(payload) && Buffer.isBuffer(events), 'missing PLY output/progress sidecar')
  const out = Buffer.alloc(9 + payload.length + events.length)
  out[0] = payload.length ? 0 : 2
  out.writeUInt32LE(payload.length, 1); payload.copy(out, 5)
  out.writeUInt32LE(events.length, 5 + payload.length); events.copy(out, 9 + payload.length)
  return out
}
// Validate complete variable-length state before exact comparison. Counts cannot
// drive iteration beyond the available packet; UTF-8 is part of the contract.
function validateObjOutput(mode, bytes) {
  let at = 0
  const take = n => { assert.ok(Number.isSafeInteger(n) && n >= 0 && at + n <= bytes.length, 'truncated OBJ state'); const b = bytes.subarray(at, at + n); at += n; return b }
  const byte = () => take(1)[0]
  const word = () => take(4).readUInt32LE()
  const text = () => new TextDecoder('utf-8', { fatal: true }).decode(take(word()))
  const array = (width, read) => { const n = word(); assert.ok(n <= Math.floor((bytes.length - at) / width), 'invalid OBJ count'); for (let i = 0; i < n; i++) read(); return n }
  const texts = () => array(4, text)
  const blob = () => take(word())
  if (mode === 5) {
    assert.equal(take(4).toString(), 'OBJ1', 'wrong OBJ result kind')
    const status = byte(); assert.ok(status === 0 || status === 2, 'unknown OBJ status')
    if (status === 2) assert.ok(text().length > 0, 'empty OBJ refusal')
    else {
      const positions = array(4, () => take(4)); assert.equal(positions % 3, 0, 'incomplete OBJ position')
      const indices = array(4, () => take(4)); assert.equal(indices % 3, 0, 'incomplete OBJ triangle')
      const colors = byte(); assert.ok(colors <= 1, 'invalid OBJ color presence')
      if (colors) assert.equal(blob().length, positions, 'missing OBJ colors')
      const note = byte(); assert.ok(note <= 2, 'unknown OBJ texture note')
      if (note) texts()
    }
    texts() // All progress messages, including a successful empty sequence.
  } else {
    assert.equal(take(4).toString(), 'OTX1', 'wrong OBJ helper kind')
    const op = byte()
    if (op === 1) text()
    else if (op === 2) array(8, () => { text(); text() })
    else if (op === 3) texts()
    else if (op === 4) blob()
    else if (op === 5) {
      const note = byte(); assert.ok(note <= 2, 'unknown helper texture note')
      if (note === 2) assert.equal(blob().length % 3, 0, 'incomplete RGB output')
      if (note) texts()
    } else throw Error('unknown OBJ helper operation')
  }
  assert.equal(at, bytes.length, 'trailing OBJ state')
}

// Complete sphere output structure. Its separately pre-registered numerical
// budget applies only to mode8; all earlier mode budgets remain unchanged.
function validateSphere(bytes) {
  assert.ok(bytes.length > 0 && bytes[0] <= 1, 'invalid sphere result tag')
  if (!bytes[0]) { assert.equal(bytes.length, 1, 'malformed sphere refusal'); return null }
  assert.ok(bytes.length >= 70, 'truncated sphere result')
  const count = bytes.readUInt32LE(65)
  assert.ok(count <= Math.floor((bytes.length - 70) / 4), 'invalid sphere used count')
  const end = 69 + count * 4, free = bytes[end]
  assert.ok(free <= 1, 'invalid sphere companion presence')
  assert.equal(bytes.length, end + 1 + free * 32, 'trailing/truncated sphere state')
  return { end, free }
}
function compareSphere(expected, actual) {
  const a = validateSphere(expected), b = validateSphere(actual)
  assert.equal(Boolean(a), Boolean(b), 'sphere optional changed')
  if (!a) return 0
  assert.ok(expected.subarray(65, a.end + 1).equals(actual.subarray(65, b.end + 1)), 'sphere used IDs/free presence changed')
  let worst = 0
  for (const [start, count] of [[1, 8], [a.end + 1, a.free ? 4 : 0]]) for (let i = 0; i < count; i++) {
    const x = expected.readDoubleLE(start + i * 8), y = actual.readDoubleLE(start + i * 8)
    if (Number.isNaN(x)) assert.ok(Number.isNaN(y), 'sphere NaN changed')
    else if (!Number.isFinite(x)) assert.equal(x, y, 'sphere nonfinite changed')
    else if (x === 0 && y === 0) assert.equal(Object.is(x, -0), Object.is(y, -0), 'sphere zero sign changed')
    else { const error = Math.abs(x-y); assert.ok(Number.isFinite(y) && error <= 1e-10 + 1e-10 * Math.abs(x), 'sphere numeric field changed'); worst = Math.max(worst, error) }
  }
  return worst
}

export function successfulOutput(mode, bytes) {
  if (mode === 9) { return validateMarkedSphere(bytes) !== null }
  if (mode === 8) { validateSphere(bytes); return bytes[0] === 1 }
  if (mode === 7) { validateFileOutput(bytes); return true }
  if (mode === 5 || mode === 6) { validateObjOutput(mode, bytes); return mode === 6 || bytes[4] === 0 }
  return mode === 1 ? bytes[0] === 1 : bytes[0] === 0
}
export function compareOutput(mode, expected, actual) {
  assert.equal(actual.length, expected.length, 'output byte length changed')
  assert.ok(expected.length > 0, 'empty output cannot pass')
  assert.equal(actual[0], expected[0], 'success/refusal changed')
  const exact = (a, b) => assert.ok(a.equals(b), 'discrete/error/region bytes changed')
  let worst = 0
  const numeric = (a, b, signed) => {
    assert.equal(a.length, b.length)
    assert.equal(a.length % 8, 0)
    for (let i = 0; i < a.length; i += 8) {
      const x = a.readDoubleLE(i), y = b.readDoubleLE(i)
      assert.ok(Number.isFinite(x) && Number.isFinite(y), 'nonfinite compared value')
      if (signed && x === 0 && y === 0) assert.equal(Object.is(x, -0), Object.is(y, -0), 'zero sign changed')
      const error = Math.abs(x - y)
      assert.ok(error <= 1e-12 * (1 + Math.abs(x)), `numeric field ${i / 8}: ${x} vs ${y}`)
      worst = Math.max(worst, error)
    }
  }
  if (mode === 9) { return compareMarkedSphere(expected, actual) }
  else if (mode === 8) { return compareSphere(expected, actual) }
  else if (mode === 7) {
    validateFileOutput(expected); validateFileOutput(actual); exact(expected, actual)
  } else if (mode === 5 || mode === 6) {
    validateObjOutput(mode, expected); validateObjOutput(mode, actual); exact(expected, actual)
  } else if (mode === 4) {
    assert.ok(expected.length >= 9, 'malformed PLY envelope')
    assert.ok(expected[0] === 0 || expected[0] === 2, 'unknown PLY result status')
    const n = expected.readUInt32LE(1)
    assert.ok(9 + n <= expected.length, 'truncated PLY output')
    assert.equal(expected.length, 9 + n + expected.readUInt32LE(5 + n), 'missing PLY progress/error')
    if (expected[0] === 0) {
      assert.ok(n >= 16, 'empty successful PLY result')
      assert.equal(expected.subarray(5, 9).toString(), 'PLY1', 'wrong PLY result kind')
      assert.ok(expected.readUInt32LE(9) <= 1, 'wrong optional winding flag')
      assert.equal(n, 16 + 4 * (expected.readUInt32LE(13) + expected.readUInt32LE(17)), 'missing PLY values')
    } else {
      assert.equal(n, 0, 'source refusal contains values')
      assert.ok(expected.readUInt32LE(5) > 0, 'empty source refusal')
    }
    exact(expected, actual) // IEEE float32 bits, including NaNs, are source observables here.
  } else if (mode === 2) exact(expected, actual)
  else if (mode === 1) {
    assert.ok(expected[0] === 0 || expected[0] === 1)
    if (!expected[0]) assert.equal(expected.length, 1, 'malformed TLS refusal')
    else { assert.ok(expected.length >= 33); numeric(expected.subarray(1), actual.subarray(1), false) }
  } else if (mode === 3) {
    assert.ok(expected[0] === 0 || expected[0] === 1)
    assert.ok(expected.length >= 5 + 720)
    const prefix = 5 + expected.readUInt32LE(1)
    exact(expected.subarray(0, prefix), actual.subarray(0, prefix))
    if (expected[0] === 0) {
      assert.equal(expected.subarray(5, prefix).toString(), 'plane')
      const end = prefix + 18 * 8
      assert.ok(expected.length >= end + 12 + 720)
      assert.equal(expected.length, end + 12 + expected.readUInt32LE(end + 8) * 4 + 720)
      numeric(expected.subarray(prefix, end), actual.subarray(prefix, end), true)
      exact(expected.subarray(end, -720), actual.subarray(end, -720))
    } else assert.equal(expected.length, prefix + 720, 'malformed marked refusal')
    numeric(expected.subarray(-720), actual.subarray(-720), true)
  } else throw Error('Unknown codec mode')
  return worst
}

// This function is executed inside Chrome as well as the explicit Node smoke mode.
// The page receives only mode/input. Expected/native/original data stays outside it.
export function runPacket({ mode, input }) {
  const api = globalThis.__scanrulerWasmAPI
  if (api.reset(mode) !== 0) throw Error('Wasm mode refused')
  for (let i = 0; i < input.length; i += 4) {
    const count = Math.min(4, input.length - i)
    let word = 0
    for (let k = 0; k < count; k++) word |= input[i + k] << (8 * k)
    if (api.push(word >>> 0, count) !== 0) throw Error('Wasm input refused')
  }
  if (api.run() !== 0) throw Error('Wasm codec/host failure')
  const length = api.length()
  if (length === 0 || length > 1048576) throw Error('Missing/oversized Wasm result')
  const padded = new Uint8Array(Math.ceil(length / 4) * 4), view = new DataView(padded.buffer)
  for (let i = 0; i < length; i += 4) view.setUint32(i, api.word(i), true)
  if (api.memory.buffer.byteLength > 67108864) throw Error('Wasm memory budget exceeded')
  return Array.from(padded.subarray(0, length))
}
export function negativePackets() {
  const a = globalThis.__scanrulerWasmAPI, checks = []
  const check = (ok, name) => { if (!ok) throw Error(name); checks.push(name) }
  check(a.reset(1) === 0 && a.push(0, 4) === 0 && a.run() === 0 && a.length() === 1, 'valid empty TLS reference case')
  check(a.word(0xffffffff) === 0, 'out-of-range word read bounded')
  const memoryBefore = a.memory.buffer.byteLength
  let memoryRefused = false
  try { a.memory.grow(1025) } catch { memoryRefused = true }
  check(memoryRefused && a.memory.buffer.byteLength === memoryBefore, 'linear memory hard limit enforced')
  check(a.push(0, 0) !== 0 && a.run() !== 0 && a.length() === 0, 'bad write cannot reuse previous output')
  check(a.reset(99) !== 0 && a.run() !== 0 && a.length() === 0, 'unknown mode fails')
  check(a.reset(1) === 0 && a.run() !== 0 && a.length() === 0, 'truncated packet fails')
  check(a.reset(1) === 0 && a.push(0xffffffff, 4) === 0 && a.run() !== 0, 'oversized scalar declaration fails')
  check(a.reset(1) === 0, 'reset after failure')
  for (let i = 0; i < 1048576 / 4; i++) if (a.push(0, 4) !== 0) throw Error('input cap ended early')
  check(a.push(0, 1) !== 0 && a.length() === 0 && a.run() !== 0, 'input budget enforced and poisoned')
  check(a.reset(1) === 0 && a.push(0, 4) === 0 && a.run() === 0 && a.length() === 1, 'clean reset recovers')
  check(a.reset(4) === 0 && a.run() === 0 && a.length() > 9 && (a.word(0) & 255) === 2, 'PLY source refusal is a completed result')
  const tooMany = 'ply\nformat ascii 1.0\nelement vertex 10001\nproperty float x\nproperty float y\nproperty float z\nelement face 1\nend_header\n'
  check(a.reset(4) === 0, 'reset before PLY host guard')
  for (let i = 0; i < tooMany.length; i++) if (a.push(tooMany.charCodeAt(i), 1) !== 0) throw Error('small PLY packet refused')
  check(a.run() !== 0 && a.length() === 0, 'PLY host budget fails without source-refusal output')
  check(a.reset(5) === 0 && a.run() !== 0 && a.length() === 0, 'truncated OBJ packet fails without output')
  check(a.reset(6) === 0 && a.push(255, 1) === 0 && a.run() !== 0 && a.length() === 0, 'unknown OBJ helper fails without output')
  check(a.reset(6) === 0 && a.push(5, 1) === 0 && a.push(0xffffffff, 4) === 0 && a.run() !== 0 && a.length() === 0, 'oversized OBJ refs fail before allocation')
  check(a.reset(5) === 0 && a.push(0, 4) === 0 && a.push(0, 1) === 0 && a.run() === 0 && a.word(0) === 0x314a424f && (a.word(4) & 255) === 2, 'OBJ source refusal is a completed result')
  check(a.reset(6) === 0 && a.push(1, 1) === 0 && a.push(0, 4) === 0 && a.run() === 0 && a.word(0) === 0x3158544f, 'OBJ helper recovers after failed packet')
  check(a.reset(7) === 0 && a.run() !== 0 && a.length() === 0, 'truncated file-state packet fails without output')
  check(a.reset(7) === 0 && a.push(255, 1) === 0 && a.run() !== 0 && a.length() === 0, 'unknown file-state mode fails without output')
  check(a.reset(7) === 0 && a.push(1, 1) === 0 && a.push(0xffffffff, 4) === 0 && a.run() !== 0 && a.length() === 0, 'oversized file declaration fails before allocation')
  check(a.reset(7) === 0, 'reset before invalid file identity')
  const badFileId = [1,1,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,1,1,0,0,0,1,0,0,0]
  for (const byte of badFileId) if (a.push(byte, 1) !== 0) throw Error('small file-state packet refused')
  check(a.run() !== 0 && a.length() === 0, 'invalid file identity fails without output')
  check(a.reset(7) === 0 && a.push(0, 1) === 0 && a.push(0, 4) === 0 && a.run() === 0 && a.word(0) === 0x31534c46, 'file-state codec recovers with complete classifier result')
  const sphere = bytes => { if (a.reset(8) !== 0) throw Error('sphere reset refused'); for (const byte of bytes) if (a.push(byte, 1) !== 0) throw Error('sphere write refused'); return a.run() }
  check(sphere([]) !== 0 && a.length() === 0, 'truncated sphere packet fails without output')
  const emptySphere = new Array(17).fill(0)
  check(sphere(emptySphere) === 0 && a.length() === 1 && a.word(0) === 0, 'sphere numerical refusal is a completed result')
  const badHeld = emptySphere.slice(); badHeld[16] = 2
  check(sphere(badHeld) !== 0 && a.length() === 0, 'unknown sphere held presence poisons output')
  const hugeSphere = emptySphere.slice(); for (let i = 0; i < 4; i++) hugeSphere[i] = 255
  check(sphere(hugeSphere) !== 0 && a.length() === 0, 'oversized sphere scalars fail before allocation')
  check(sphere([...emptySphere, 0]) !== 0 && a.length() === 0, 'trailing sphere data fails without output')
  const badSphereIndex = new Array(33).fill(0); badSphereIndex[0] = 3; badSphereIndex[4] = 1; badSphereIndex[29] = 1
  check(sphere(badSphereIndex) !== 0 && a.length() === 0, 'invalid sphere selection fails without output')
  const marked = bytes => { if (a.reset(9) !== 0) throw Error('marked-sphere reset refused'); for (const byte of bytes) if (a.push(byte, 1) !== 0) throw Error('marked-sphere write refused'); return a.run() }
  check(marked([]) !== 0 && a.length() === 0, 'truncated marked-sphere packet fails without output')
  check(marked(emptySphere) === 0 && a.length() > 10 && (a.word(0) & 255) === 1, 'marked-sphere minimum marking FitError is completed')
  check(marked(badHeld) !== 0 && a.length() === 0, 'unknown marked-sphere held presence poisons output')
  const manyMarked = emptySphere.slice(); for(let i=4;i<8;i++)manyMarked[i]=255
  check(marked(manyMarked) !== 0 && a.length() === 0, 'oversized marking count fails before allocation')
  check(marked([...emptySphere,0]) !== 0 && a.length() === 0, 'trailing marked-sphere bytes fail without output')
  const badMarked = new Array(109).fill(0); badMarked[0]=3;badMarked[4]=20;for(let i=29;i<109;i+=4)badMarked[i]=1
  check(marked(badMarked) !== 0 && a.length() === 0, 'invalid marked-sphere vertex fails without output')
  return checks
}

// Node-side only: preserve the mismatching bytes before any assertion can throw.
export async function retainAndCompare(directory, mode, files, wasm) {
  const { mkdirSync, writeFileSync } = await import('node:fs')
  const { resolve } = await import('node:path')
  mkdirSync(directory, { recursive: true })
  for (const [name, bytes] of Object.entries(files)) writeFileSync(resolve(directory, name), bytes)
  writeFileSync(resolve(directory, 'wasm.bin'), wasm)
  const original = mode === 4 ? plyPacket(files['original.bin'], files['original.progress-or-error.txt']) : files['original.bin']
  const native = mode === 4 ? plyPacket(files['rust.bin'], files['rust.progress-or-error.txt']) : files['rust.bin']
  if (mode === 4) {
    writeFileSync(resolve(directory, 'original.packet.bin'), original)
    writeFileSync(resolve(directory, 'rust.packet.bin'), native)
  }
  return {
    errorNative: compareOutput(mode, original, native),
    errorWasm: compareOutput(mode, original, wasm),
    errorPair: compareOutput(mode, native, wasm),
  }
}

function validateMarkedSphere(bytes) {
  let at=1
  const take=n=>{assert.ok(Number.isSafeInteger(n)&&n>=0&&at+n<=bytes.length,'truncated marked-sphere output');const v=bytes.subarray(at,at+n);at+=n;return v}
  const byte=()=>take(1)[0],word=()=>take(4).readUInt32LE(),text=()=>new TextDecoder('utf-8',{fatal:true}).decode(take(word()))
  assert.ok(bytes.length>0&&bytes[0]<=1,'invalid marked-sphere tag')
  if(bytes[0]===1){assert.equal(text(),'Error');assert.equal(byte(),1,'missing FitError brand');assert.equal(text(),'FitError');assert.ok(text().length>0,'missing FitError message');assert.equal(at,bytes.length);return null}
  assert.equal(text(),'sphere');const numbers=at;take(64);const counts=at,used=word(),size=word(),held=byte();assert.ok(held<=1,'invalid held presence');assert.ok(used<=size,'used count exceeds original region');const heldNumbers=at;if(held)take(16);const region=at,n=word();assert.equal(n,size,'lost original recipe region');assert.ok(n<=Math.floor((bytes.length-at-1)/4),'invalid region count');take(4*n);assert.equal(byte(),1,'original region identity changed');assert.equal(at,bytes.length,'trailing marked-sphere state');return {numbers,counts,heldNumbers,held,region}
}
function compareMarkedSphere(expected,actual){
  const a=validateMarkedSphere(expected),b=validateMarkedSphere(actual);assert.equal(Boolean(a),Boolean(b),'marked-sphere refusal changed');if(!a){assert.ok(expected.equals(actual),'FitError fields/message changed');return 0}
  assert.ok(expected.subarray(0,a.numbers).equals(actual.subarray(0,b.numbers)),'kind changed');assert.ok(expected.subarray(a.counts,a.heldNumbers).equals(actual.subarray(b.counts,b.heldNumbers)),'used/region/held presence changed');assert.ok(expected.subarray(a.region).equals(actual.subarray(b.region)),'region/order/identity changed')
  let worst=0
  for(const [start,count]of[[a.numbers,8],[a.heldNumbers,a.held?2:0]])for(let i=0;i<count;i++){
    const x=expected.readDoubleLE(start+8*i),y=actual.readDoubleLE(start+8*i)
    if(Number.isNaN(x))assert.ok(Number.isNaN(y))
    else if(!Number.isFinite(x))assert.equal(x,y)
    else if(x===0&&y===0)assert.equal(Object.is(x,-0),Object.is(y,-0),'zero sign changed')
    else{const error=Math.abs(x-y);assert.ok(Number.isFinite(y)&&error<=1e-10+1e-10*Math.abs(x),'marked-sphere numeric field changed');worst=Math.max(worst,error)}
  }return worst
}
