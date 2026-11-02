// SPDX-License-Identifier: AGPL-3.0-only
const {execFileSync}=require('node:child_process'),{readFileSync}=require('node:fs'),vm=require('node:vm')
const s=execFileSync('git',['show','2cd89c6769a10c27ee7754b477ada521c9ef5cb7:src/core/geometry/normals.ts'],{encoding:'utf8'})
const signature='export function computeVertexNormals(positions: Float32Array, indices: Uint32Array): Float32Array {'
const start=s.indexOf(signature),end=s.indexOf('\n/**',start)
if(start<0||end<0)throw Error('Pinned helper changed')
const body=s.slice(start,end).replace(signature,'function computeVertexNormals(positions,indices) {')
const compute=vm.runInNewContext(body+'\ncomputeVertexNormals',{Float32Array})
const bytes=readFileSync(0),count=bytes.readUInt32LE(0)
const positions=new Float32Array(bytes.buffer.slice(bytes.byteOffset+4,bytes.byteOffset+4+count*4))
const indices=new Uint32Array(bytes.buffer.slice(bytes.byteOffset+4+count*4,bytes.byteOffset+bytes.byteLength))
const result=compute(positions,indices),out=Buffer.alloc(result.length*4)
for(let i=0;i<result.length;i++)out.writeFloatLE(result[i],i*4)
process.stdout.write(out)
