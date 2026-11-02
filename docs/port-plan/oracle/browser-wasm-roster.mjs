// SPDX-License-Identifier: AGPL-3.0-only
import assert from 'node:assert/strict'
import { createHash } from 'node:crypto'

export function nativeFileNames(mode) {
  const names = ['input.bin', 'original.bin', 'rust.bin']
  if (mode === 4) names.push('original.progress-or-error.txt', 'rust.progress-or-error.txt')
  return names
}
export function verifyNativeCaseFiles(mode, files, metadata) {
  for (const name of nativeFileNames(mode)) {
    assert.ok(Buffer.isBuffer(files[name]), 'missing native raw file: ' + name)
    assert.ok(metadata.files[name], 'missing native sidecar metadata: ' + name)
    assert.equal(createHash('sha256').update(files[name]).digest('hex'), metadata.files[name].sha256, 'corrupt or changed raw file: ' + name)
    assert.equal(files[name].length, metadata.files[name].bytes, 'changed raw file length: ' + name)
  }
}

export function validateBrowserRoster(defs) {
  assert.equal(defs.length, 5, 'missing browser suite')
  assert.equal(new Set(defs.map(s => s.unit)).size, 5, 'duplicate browser unit')
  assert.deepEqual(defs.map(s => s.mode).sort(), [1, 2, 3, 4, 4], 'changed browser codecs')
  assert.deepEqual(defs.filter(s => s.mode <= 3).map(s => [s.unit, s.cases]), [
    ['fit.plane.tls', 13], ['geometry.mesh.graph', 50], ['fit.plane.selection', 51],
  ], 'previous browser coverage changed')
  assert.equal(defs.reduce((n, s) => n + s.cases, 0), 341, 'trimmed browser coverage')
}

export function selectBrowserCases(def, nativeIds) {
  const exclusions = def.excluded ?? []
  assert.equal(new Set(nativeIds).size, nativeIds.length, 'duplicate native identity')
  assert.equal(new Set(exclusions.map(e => e.id)).size, exclusions.length, 'duplicate exclusion')
  for (const e of exclusions) {
    assert.ok(e.reason?.trim(), 'unexplained runtime exclusion')
    assert.ok(nativeIds.includes(e.id), 'unknown excluded case')
  }
  if (exclusions.length) {
    assert.equal(def.unit, 'io.ply.binary', 'previous browser cases cannot be excluded')
    assert.deepEqual(exclusions.map(e => e.id), ['le-vertex-progress'], 'unreviewed workload exclusion')
  }
  assert.equal(nativeIds.length, def.cases + exclusions.length, 'missing native/browser cases')
  return nativeIds.filter(id => !exclusions.some(e => e.id === id)).slice().sort()
}
