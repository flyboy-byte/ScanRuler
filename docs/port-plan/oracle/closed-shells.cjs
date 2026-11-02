// SPDX-License-Identifier: AGPL-3.0-only
// Full frozen closure and union-find bodies, with type declarations removed only.
const {execFileSync}=require('node:child_process'),{readFileSync}=require('node:fs'),vm=require('node:vm')
const frozen=p=>execFileSync('git',['show','2cd89c6769a10c27ee7754b477ada521c9ef5cb7:src/core/geometry/'+p+'.ts'],{encoding:'utf8'})
function removeInterface(s,name){const a=s.indexOf('export interface '+name+' {'),b=s.indexOf('\n}',a);if(a<0||b<0)throw Error('Frozen interface changed');return s.slice(0,a)+s.slice(b+2)}
let source=removeInterface(frozen('unionFind'),'UnionFind')+'\n'+removeInterface(frozen('closedShells'),'ClosedShells').replace(/^import .*\n/gm,'')
for(const[a,b]of[
 ['export function findRoot(parent: Int32Array | Uint32Array | number[], i: number): number {','function findRoot(parent,i) {'],
 ['export function makeUnionFind(n: number): UnionFind {','function makeUnionFind(n) {'],
 ['const find = (i: number): number =>','const find = (i) =>'],
 ['union(a: number, b: number) {','union(a,b) {'],
 ['export function vertexComponents(indices: Uint32Array, vertexCount: number): UnionFind {','function vertexComponents(indices,vertexCount) {'],
 ['export function closedShells(positions: Float32Array, indices: ArrayLike<number> | null): ClosedShells {','function closedShells(positions,indices) {'],
 ['const tally = (r: number, p: number, q: number) =>','const tally = (r,p,q) =>'],
 ['function weldIds(positions: Float32Array): { id: Int32Array; count: number } {','function weldIds(positions) {'],
 ['function hash3(a: number, b: number, c: number): number {','function hash3(a,b,c) {'],
 ['function edgeHash(x: number, y: number, seed: number): number {','function edgeHash(x,y,seed) {']
]){if(!source.includes(a))throw Error('Frozen declaration changed: '+a);source=source.replace(a,b)}
source=source.replaceAll('(i: number)','(i)')
const api=vm.runInNewContext(source+'\n({closedShells,weldIds,edgeHash,hash3})',{Int32Array,Uint32Array,Uint8Array})
const bytes=readFileSync(0),has=bytes.readUInt32LE(0)!==0,n=bytes.readUInt32LE(4)
const copy=(a,b)=>bytes.buffer.slice(bytes.byteOffset+a,bytes.byteOffset+b)
const p=new Float32Array(copy(8,8+n*4)),idx=has?new Uint32Array(copy(8+n*4,bytes.length)):null
const result=api.closedShells(p,idx),weld=api.weldIds(p),chunks=[]
const word=v=>{const b=Buffer.alloc(4);b.writeUInt32LE(v>>>0);chunks.push(b)}
word(result.count);word(result.closed.length);chunks.push(Buffer.from(result.closed));word(weld.count);word(weld.id.length);weld.id.forEach(word)
for(const[x,y]of[[0,1],[1,0],[1,1],[0xffffffff,1],[0x80000000,0x7fffffff]])for(const seed of[0,1,0x9e3779b1,0xffffffff]){word(api.edgeHash(x,y,seed));word(api.hash3(x,y,seed))}
process.stdout.write(Buffer.concat(chunks))
