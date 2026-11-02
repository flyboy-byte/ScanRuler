// SPDX-License-Identifier: AGPL-3.0-only
// Complete frozen source bodies; TypeScript declarations only are removed.
const {execFileSync}=require('node:child_process'),{readFileSync}=require('node:fs'),vm=require('node:vm')
let source=execFileSync('git',['show','2cd89c6769a10c27ee7754b477ada521c9ef5cb7:src/core/geometry/unionFind.ts'],{encoding:'utf8'})
const a=source.indexOf('export interface UnionFind {'),b=source.indexOf('\n}',a)
if(a<0||b<0)throw Error('Frozen interface boundary changed')
source=source.slice(0,a)+source.slice(b+2)
for(const [a,b]of[
 ['export function findRoot(parent: Int32Array | Uint32Array | number[], i: number): number {','function findRoot(parent,i) {'],
 ['export function makeUnionFind(n: number): UnionFind {','function makeUnionFind(n) {'],
 ['const find = (i: number): number =>','const find = (i) =>'],
 ['union(a: number, b: number) {','union(a,b) {'],
 ['export function vertexComponents(indices: Uint32Array, vertexCount: number): UnionFind {','function vertexComponents(indices,vertexCount) {']
]){if(!source.includes(a))throw Error('Frozen annotation changed: '+a);source=source.replace(a,b)}
const api=vm.runInNewContext(source+'\n({findRoot,makeUnionFind,vertexComponents})',{Int32Array})
const bytes=readFileSync(0),mode=bytes.readUInt32LE(0),n=bytes.readUInt32LE(4),len=bytes.readUInt32LE(8)
const copy=(a,b)=>bytes.buffer.slice(bytes.byteOffset+a,bytes.byteOffset+b)
const data=new Uint32Array(copy(12,12+len*4)),ops=new Uint32Array(copy(12+len*4,bytes.length))
const sets=mode===0?api.makeUnionFind(n):mode===1?api.vertexComponents(data,n):{parent:data,find(i){return api.findRoot(data,i)}}
const output=[sets.parent.length,...sets.parent]
for(let k=0;k<ops.length;k+=3){
 const result=ops[k]===0?sets.find(ops[k+1]):(sets.union(ops[k+1],ops[k+2]),0xffffffff)
 output.push(result,...sets.parent)
}
const out=Buffer.alloc(output.length*4);output.forEach((v,i)=>out.writeUInt32LE(v,i*4));process.stdout.write(out)
