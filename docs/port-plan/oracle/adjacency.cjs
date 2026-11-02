// SPDX-License-Identifier: AGPL-3.0-only
// Frozen source body; only the TypeScript function declaration is replaced.
const {execFileSync}=require('node:child_process'),{readFileSync}=require('node:fs'),vm=require('node:vm')
const source=execFileSync('git',['show','2cd89c6769a10c27ee7754b477ada521c9ef5cb7:src/core/geometry/adjacency.ts'],{encoding:'utf8'})
const signature=`export function buildAdjacency(
  indices: Uint32Array,
  vertexCount: number,
): { offsets: Uint32Array; list: Uint32Array } {`
if(!source.includes(signature))throw Error('Frozen adjacency declaration changed')
const body=source.replace(signature,'function buildAdjacency(indices,vertexCount) {')
const api=vm.runInNewContext(body+'\n({buildAdjacency})',{Uint32Array})
const input=readFileSync(0),vertexCount=input.readUInt32LE(0)
const indices=new Uint32Array(input.buffer.slice(input.byteOffset+4,input.byteOffset+input.length))
const {offsets,list}=api.buildAdjacency(indices,vertexCount)
const out=Buffer.alloc(8+(offsets.length+list.length)*4);out.writeUInt32LE(offsets.length,0);out.writeUInt32LE(list.length,4)
let off=8;for(const values of [offsets,list])for(const v of values){out.writeUInt32LE(v,off);off+=4}
process.stdout.write(out)
