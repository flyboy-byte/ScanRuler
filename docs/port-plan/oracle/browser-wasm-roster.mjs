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

export function validateBrowserRoster(defs, contract = 'browser.wasm.corpus.v4') {
  assert.ok(['browser.wasm.corpus.v2', 'browser.wasm.corpus.v3', 'browser.wasm.corpus.v4'].includes(contract), 'unknown browser roster contract')
  const files = contract === 'browser.wasm.corpus.v4'
  const current = contract !== 'browser.wasm.corpus.v2'
  assert.equal(defs.length, files ? 8 : current ? 7 : 5, 'missing browser suite')
  assert.equal(new Set(defs.map(s => s.unit)).size, defs.length, 'duplicate browser unit')
  assert.deepEqual(defs.map(s => s.mode).sort(), files ? [1, 2, 3, 4, 4, 5, 6, 7] : current ? [1, 2, 3, 4, 4, 5, 6] : [1, 2, 3, 4, 4], 'changed browser codecs')
  assert.deepEqual(defs.slice(0, 5).map(s => [s.unit, s.directory, s.mode, s.cases]), [
    ['fit.plane.tls', 'plane-diff', 1, 13], ['geometry.mesh.graph', 'mesh-graph-diff', 2, 50],
    ['fit.plane.selection', 'marked-plane-diff', 3, 51], ['io.ply.ascii', 'ply-ascii-diff', 4, 60],
    ['io.ply.binary', 'ply-binary-diff', 4, 167],
  ], 'previous browser coverage changed')
  if (current) assert.deepEqual(defs.slice(5, 7).map(s => [s.unit, s.directory, s.mode, s.cases]), [
    ['io.obj.texture', 'obj-texture-diff', 6, 144], ['io.obj.mesh', 'obj-mesh-diff', 5, 79],
  ], 'changed OBJ browser coverage')
  if (files) assert.deepEqual(defs.slice(7).map(s => [s.unit, s.directory, s.mode, s.cases]), [
    ['io.obj.companions.files', 'obj-files-diff', 7, 87],
  ], 'changed file-state browser coverage')
  assert.equal(defs.reduce((n, s) => n + s.cases, 0), files ? 651 : current ? 564 : 341, 'trimmed browser coverage')
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
    const allowed = { 'io.ply.binary': ['le-vertex-progress'], 'io.obj.mesh': ['line-progress', 'short-unicode-line-skips-progress'] }
    assert.ok(allowed[def.unit], 'previous browser cases cannot be excluded')
    assert.deepEqual(exclusions.map(e => e.id), allowed[def.unit], 'unreviewed workload exclusion')
  }
  assert.equal(nativeIds.length, def.cases + exclusions.length, 'missing native/browser cases')
  return nativeIds.filter(id => !exclusions.some(e => e.id === id)).slice().sort()
}
