// SPDX-License-Identifier: AGPL-3.0-only
import { test } from 'node:test'
import assert from 'node:assert/strict'
import { compareOutput } from './browser-wasm-contract.mjs'
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
