// SPDX-License-Identifier: AGPL-3.0-only
// Complete frozen bodies; only function declarations have TS types removed.
const {execFileSync}=require('node:child_process'),{readFileSync}=require('node:fs'),vm=require('node:vm')
const frozen=p=>execFileSync('git',['show','2cd89c6769a10c27ee7754b477ada521c9ef5cb7:'+p],{encoding:'utf8'})
let source=frozen('src/core/geometry/remap.ts')+'\n'+frozen('src/core/geometry/region.ts')
for(const [a,b]of[
 ['export function remapVertices(ids: Uint32Array, vertexMap: Int32Array): Uint32Array {','function remapVertices(ids,vertexMap) {'],
 [`export function trianglesWithin(
  indices: Uint32Array,
  vertices: Uint32Array,
  vertexCount: number,
): Uint32Array {`,'function trianglesWithin(indices,vertices,vertexCount) {']
]){if(!source.includes(a))throw Error('Frozen declaration changed');source=source.replace(a,b)}
const api=vm.runInNewContext(source+'\n({remapVertices,trianglesWithin})',{Uint32Array,Uint8Array})
const bytes=readFileSync(0),mode=bytes.readUInt32LE(0),n=bytes.readUInt32LE(4),len=bytes.readUInt32LE(8)
const copy=(a,b)=>bytes.buffer.slice(bytes.byteOffset+a,bytes.byteOffset+b)
const a=new Uint32Array(copy(12,12+len*4)),b=mode===0?new Int32Array(copy(12+len*4,bytes.length)):new Uint32Array(copy(12+len*4,bytes.length))
const result=mode===0?api.remapVertices(a,b):api.trianglesWithin(a,b,n)
const out=Buffer.alloc(4+result.length*4);out.writeUInt32LE(result.length,0);result.forEach((v,i)=>out.writeUInt32LE(v,4+i*4));process.stdout.write(out)
