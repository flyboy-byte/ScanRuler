// SPDX-License-Identifier: AGPL-3.0-only
// Test-only framing validation; no selection/classification algorithm lives here.
import assert from 'node:assert/strict'
export function validateFileOutput(bytes) {
  assert.ok(bytes.length <= 1048576, 'oversized file-state output')
  let at = 0
  const take = n => { assert.ok(Number.isSafeInteger(n) && n >= 0 && at + n <= bytes.length, 'truncated file state'); const b = bytes.subarray(at, at + n); at += n; return b }
  const byte = () => take(1)[0], word = () => take(4).readUInt32LE()
  const count = (max, width) => { const n = word(); assert.ok(n <= max && n <= Math.floor((bytes.length - at) / width), 'invalid file-state count'); return n }
  const text = (max = 4096) => { const n = count(max, 1); return new TextDecoder('utf-8', { fatal: true }).decode(take(n)) }
  const ids = () => { const n = count(64, 4); return Array.from({ length: n }, word) }
  assert.equal(take(4).toString(), 'FLS1', 'wrong file-state result kind')
  const mode = byte()
  if (mode === 0) {
    text(12288)
    for (let i = 0; i < 6; i++) assert.ok(byte() <= 1, 'invalid classifier flag')
    for (let i = 0; i < 6; i++) assert.ok(text().length > 0, 'missing file registry constant')
  } else if (mode === 1) {
    const operations = count(64, 9)
    let fileCount
    for (let i = 0; i < operations; i++) {
      const op = byte(), refs = []
      if (op === 1) { const selected = word(); if (selected !== 0xffffffff) refs.push(selected) }
      else if (op === 2) refs.push(...ids())
      else if (op === 3) {
        const n = count(64, 8)
        for (let j = 0; j < n; j++) { text(); take(count(1048576, 1)) }
      } else throw Error('unknown file-state operation')
      const n = count(64, 4)
      if (fileCount !== undefined) assert.equal(n, fileCount, 'file identities changed mid-trace')
      fileCount = n
      for (let j = 0; j < n; j++) refs.push(...ids())
      for (const id of refs) assert.ok(id < n, 'file association/selection outside owner')
    }
  } else throw Error('unknown file-state mode')
  assert.equal(at, bytes.length, 'trailing file state')
}
