// SPDX-License-Identifier: AGPL-3.0-only
// Differential fuzz: pinned (0.9.2) parsers vs upstream HEAD (0.9.4) parsers.
// usage: tsx diff-parsers.mts <pinSrcDir> <newSrcDir> [casesPerFormat]
import { pathToFileURL } from 'node:url'

const [pinDir, newDir, nArg] = process.argv.slice(2)
const N = Number(nArg ?? 20000)
const load = async (dir: string) => ({
  stl: (await import(pathToFileURL(`${dir}/core/parsers/stl.ts`).href)).parseSTL,
  ply: (await import(pathToFileURL(`${dir}/core/parsers/ply.ts`).href)).parsePLY,
  obj: (await import(pathToFileURL(`${dir}/core/parsers/obj.ts`).href)).parseOBJ,
})
const A = await load(pinDir)
const B = await load(newDir)

let seed = 0x9e3779b9
const rnd = () => {
  seed |= 0; seed = (seed + 0x6d2b79f5) | 0
  let t = Math.imul(seed ^ (seed >>> 15), 1 | seed)
  t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t
  return ((t ^ (t >>> 14)) >>> 0) / 4294967296
}
const pick = <T,>(xs: T[]): T => xs[Math.floor(rnd() * xs.length)]
const int = (a: number, b: number) => a + Math.floor(rnd() * (b - a + 1))
const enc = (s: string) => new TextEncoder().encode(s).buffer as ArrayBuffer

const numTok = () => pick(['0', '1', '-1', '2.5', '1e3', '-3.25e-2', '0.000001', '123456.789', 'nan', 'NaN', 'inf', 'abc', '1,5', '', '+7', '.5', '5.', '0x10', '1e999', '-0'])
const ws = () => pick([' ', '  ', '\t', ' \t ', ' ', '﻿', ' ', '\u0085', '​', '　'])
const eol = () => (process.env.NOBADCOUNT === '1' ? pick(['\n', '\n', '\r\n', '\n\n']) : pick(['\n', '\n', '\n', '\r\n', '\r', '\n\n']))
const PURE = process.env.PURE === '1'
const clean = () => (PURE ? String(Math.round((rnd() * 200 - 100) * 1000) / 1000) : rnd() < 0.7 ? String(Math.round((rnd() * 200 - 100) * 1000) / 1000) : numTok())
const maybeWs = () => (rnd() < 0.85 ? ' ' : ws())

function genSTL(): string {
  let s = rnd() < 0.8 ? 'solid t' + eol() : pick(['', 'solid', 'x', 'vertex ' + clean()]) + eol()
  const facets = int(0, 6)
  for (let f = 0; f < facets; f++) {
    s += ' facet normal 0 0 1' + eol() + '  outer loop' + eol()
    const verts = rnd() < 0.9 ? 3 : int(0, 5)
    for (let v = 0; v < verts; v++) {
      const k = rnd() < 0.93 ? 3 : int(0, 5)
      s += '   vertex'
      for (let i = 0; i < k; i++) s += maybeWs() + clean()
      s += rnd() < 0.1 ? ' extra' : ''
      s += eol()
    }
    s += '  endloop' + eol() + ' endfacet' + eol()
  }
  if (rnd() < 0.85) s += 'endsolid t'
  if (rnd() < 0.5) s += eol()
  if (rnd() < 0.1) s = s.slice(0, int(0, s.length))
  return s
}

function genPLY(): string {
  const nv = int(0, 6)
  const nf = int(0, 5)
  const listType = pick(['uchar int', 'uchar uint', 'int int', 'uchar int'])
  const idxName = pick(['vertex_indices', 'vertex_indices', 'vertex_index', 'other_list'])
  let s = 'ply' + eol() + 'format ascii 1.0' + eol() + `element vertex ${nv}` + eol() + 'property float x' + eol() + 'property float y' + eol() + 'property float z' + eol()
  const hasRed = rnd() < 0.3
  if (hasRed) s += 'property uchar red' + eol()
  s += `element face ${nf}` + eol() + `property list ${listType} ${idxName}` + eol()
  if (rnd() < 0.2) s += 'property float quality' + eol()
  if (rnd() < 0.15) s += 'element extra 2' + eol() + 'property list uchar float vals' + eol()
  s += 'end_header' + eol()
  for (let i = 0; i < nv; i++) {
    s += [0, 1, 2].map(() => clean()).join(maybeWs()) + (hasRed ? ' 255' : '') + eol()
  }
  for (let f = 0; f < nf; f++) {
    const roll = rnd()
    const n = process.env.NOBADCOUNT === '1' ? (roll < 0.85 ? int(3, 5) : int(0, 2)) : roll < 0.75 ? int(3, 5) : roll < 0.85 ? int(0, 2) : roll < 0.92 ? pick([-1, -3]) : pick(['x', 'NaN', '99', '1.5', ''])
    const count = typeof n === 'number' && n >= 0 && n < 50 ? n : int(0, 4)
    s += String(n)
    for (let k = 0; k < count; k++) s += maybeWs() + (rnd() < 0.9 ? int(0, Math.max(0, nv)) : pick(['-1', '9999', 'z', '2.7']))
    if (rnd() < 0.2) s += ' 0.5'
    s += eol()
  }
  if (rnd() < 0.15) s += '2 1.5 2.5' + eol()
  if (rnd() < 0.1) s = s.slice(0, int(0, s.length))
  return s
}

function genOBJ(): string {
  const nv = PURE ? int(3, 8) : int(0, 6)
  let s = rnd() < 0.2 ? '# comment' + eol() : ''
  for (let i = 0; i < nv; i++) {
    const k = rnd() < (PURE ? 0.995 : 0.9) ? 3 : int(0, 5)
    s += 'v'
    for (let j = 0; j < k; j++) s += maybeWs() + clean()
    if (rnd() < 0.1) s += ' # trailing'
    s += eol()
    if (rnd() < 0.1) s += pick(['vn 0 0 1', 'vt 0.5 0.5', 'o part', 'g grp', 's off', 'usemtl m']) + eol()
  }
  const nf = int(0, 5)
  for (let f = 0; f < nf; f++) {
    const k = rnd() < 0.9 ? int(3, 5) : int(0, 2)
    s += pick(['f', 'f', 'f', 'F', 'f' + '\t'])
    for (let j = 0; j < k; j++) {
      const idx = rnd() < (PURE ? 0.97 : 0.8) ? int(1, Math.max(1, nv)) : pick([0, -1, -2, 9999, 'x', '1.5', ''])
      s += maybeWs() + idx + pick(['', '', '/1', '//1', '/1/1'])
    }
    s += eol()
  }
  if (rnd() < 0.1) s = s.slice(0, int(0, s.length))
  return s
}

const run = (fn: (b: ArrayBuffer) => any, buf: ArrayBuffer) => {
  try {
    const m = fn(buf)
    const r: any = { kind: m.kind, p: Array.from(m.positions as Float32Array, (x) => (Object.is(x, -0) ? '-0' : Number.isNaN(x) ? 'nan' : x)) }
    if (m.indices) r.i = Array.from(m.indices as Uint32Array)
    return JSON.stringify(r)
  } catch (e) {
    return 'ERR:' + (e instanceof Error ? e.message : String(e))
  }
}

const gens: Record<string, () => string> = { stl: genSTL, ply: genPLY, obj: genOBJ }
let total = 0
for (const fmt of ['stl', 'ply', 'obj'] as const) {
  let diff = 0, errBoth = 0, okBoth = 0
  const examples: string[] = []
  const kinds = new Map<string, number>()
  let shortest: string | null = null
  for (let n = 0; n < N; n++) {
    const text = gens[fmt]()
    const buf = enc(text)
    const a = run(A[fmt], buf)
    const b = run(B[fmt], buf)
    total++
    if (a === b) { a.startsWith('ERR') ? errBoth++ : okBoth++; continue }
    diff++
    const cat = (a.startsWith('ERR') ? 'pin-error' : 'pin-ok') + ' / ' + (b.startsWith('ERR') ? 'new-error' : 'new-ok')
    kinds.set(cat, (kinds.get(cat) ?? 0) + 1)
    if (shortest === null || text.length < shortest.length) shortest = text
    if (examples.length < 3) examples.push(JSON.stringify(text.slice(0, 160)) + '\n      pin: ' + a.slice(0, 140) + '\n      new: ' + b.slice(0, 140))
  }
  console.log(`${fmt.toUpperCase()}: ${N} inputs | identical ok ${okBoth}, identical error ${errBoth}, DIFFER ${diff}`)
  for (const [k, v] of kinds) console.log(`   ${k}: ${v}`)
  if (shortest !== null) { console.log('   SHORTEST differing input: ' + JSON.stringify(shortest)); console.log('      pin: ' + run(A[fmt], enc(shortest))); console.log('      new: ' + run(B[fmt], enc(shortest))) }
}
console.log('total inputs', total)
