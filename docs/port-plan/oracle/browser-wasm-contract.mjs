// SPDX-License-Identifier: AGPL-3.0-only
// Test comparison only. No geometry, source-result injection or product runtime.
import assert from 'node:assert/strict'
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
  if (mode === 2) exact(expected, actual)
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
  check(a.push(0, 0) !== 0 && a.run() !== 0 && a.length() === 0, 'bad write cannot reuse previous output')
  check(a.reset(99) !== 0 && a.run() !== 0 && a.length() === 0, 'unknown mode fails')
  check(a.reset(1) === 0 && a.run() !== 0 && a.length() === 0, 'truncated packet fails')
  check(a.reset(1) === 0 && a.push(0xffffffff, 4) === 0 && a.run() !== 0, 'oversized scalar declaration fails')
  check(a.reset(1) === 0, 'reset after failure')
  for (let i = 0; i < 1048576 / 4; i++) if (a.push(0, 4) !== 0) throw Error('input cap ended early')
  check(a.push(0, 1) !== 0 && a.length() === 0 && a.run() !== 0, 'input budget enforced and poisoned')
  check(a.reset(1) === 0 && a.push(0, 4) === 0 && a.run() === 0 && a.length() === 1, 'clean reset recovers')
  return checks
}
