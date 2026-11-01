// SPDX-License-Identifier: AGPL-3.0-only
// Oracle-only adapter: frozen function bodies, annotations removed.
const {execFileSync}=require('node:child_process')
const {readFileSync}=require('node:fs')
const vm=require('node:vm')
let s=execFileSync('git',['show','2cd89c6769a10c27ee7754b477ada521c9ef5cb7:src/core/geometry/weld.ts'],{encoding:'utf8'})
for(const [before,after] of [
 ['export function weldTriangleSoup(\n  soup: Float32Array,\n  onProgress?: (text: string) => void,\n): { positions: Float32Array; indices: Uint32Array } {','function weldTriangleSoup(soup,onProgress) {'],
 ['function hash3(a: number, b: number, c: number): number {','function hash3(a,b,c) {'],
 ['export function filterDegenerateTriangles(indices: Uint32Array): Uint32Array {','function filterDegenerateTriangles(indices) {']
]) {if(!s.includes(before)) throw Error('Source annotation changed');s=s.replace(before,after)}
const api=vm.runInNewContext(s+'\n({weldTriangleSoup,filterDegenerateTriangles})',{Float32Array,Uint32Array,Int32Array})
const bytes=readFileSync(0)
const soup=new Float32Array(bytes.buffer.slice(bytes.byteOffset,bytes.byteOffset+bytes.byteLength))
const mesh=api.weldTriangleSoup(soup,t=>process.stderr.write(t+'\n'))
const filtered=api.filterDegenerateTriangles(mesh.indices)
const out=Buffer.alloc(4+mesh.positions.length*4+mesh.indices.length*4+filtered.length*4)
let offset=0
out.writeUInt32LE(mesh.positions.length,offset);offset+=4
for(const value of mesh.positions){out.writeFloatLE(value,offset);offset+=4}
for(const values of [mesh.indices,filtered])for(const value of values){out.writeUInt32LE(value,offset);offset+=4}
process.stdout.write(out)
