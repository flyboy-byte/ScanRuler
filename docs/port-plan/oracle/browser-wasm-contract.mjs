// SPDX-License-Identifier: AGPL-3.0-only
// Test comparison only. No geometry, source-result injection or product runtime.
import assert from 'node:assert/strict'
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
export function successfulOutput(mode, bytes) {
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
  if (mode === 5 || mode === 6) {
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
