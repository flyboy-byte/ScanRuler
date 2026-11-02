// SPDX-License-Identifier: AGPL-3.0-only
const {execFileSync}=require('node:child_process'),{readFileSync}=require('node:fs'),vm=require('node:vm')
const source=p=>execFileSync('git',['show','2cd89c6769a10c27ee7754b477ada521c9ef5cb7:'+p],{encoding:'utf8'})
const bounds=source('src/core/geometry/bounds.ts'),centroid=source('src/core/geometry/centroid.ts'),normals=source('src/core/geometry/normals.ts')
const extentSig='export function extentOf(positions: Float32Array, count = positions.length / 3): Extent {'
let extent=bounds.slice(bounds.indexOf(extentSig),bounds.indexOf('\n/** The extent of the points'))
extent=extent.replace(extentSig,'function extentOf(positions,count=positions.length/3) {').replaceAll(': Vec3','')
const tetraSig='export function tetraVolume6(p: Float32Array, a: number, b: number, c: number, o: Vec3 = ORIGIN): number {'
let tetra=centroid.slice(centroid.indexOf(tetraSig)).replace(tetraSig,'function tetraVolume6(p,a,b,c,o=[0,0,0]) {')
const orientSig='export function orientNormalsOutward(\n  positions: Float32Array,\n  indices: Uint32Array,\n  normals: Float32Array,\n): boolean {'
let orient=normals.slice(normals.indexOf(orientSig)).replace(orientSig,'function orientNormalsOutward(positions,indices,normals) {').replace(': [number, number, number]','')
if(!extent.startsWith('function extentOf')||!tetra.startsWith('function tetraVolume6')||!orient.startsWith('function orientNormalsOutward'))throw Error('Frozen source changed')
const run=vm.runInNewContext(extent+tetra+orient+'\norientNormalsOutward',{})
const input=readFileSync(0),n=input.readUInt32LE(0),m=input.readUInt32LE(4)
const copy=(a,b)=>input.buffer.slice(input.byteOffset+a,input.byteOffset+b)
const p=new Float32Array(copy(8,8+n*4)),idx=new Uint32Array(copy(8+n*4,8+n*4+m*4)),ns=new Float32Array(copy(8+n*4+m*4,input.length))
const changed=run(p,idx,ns),out=Buffer.alloc(1+m*4+n*4);out[0]=Number(changed)
let off=1;for(const i of idx){out.writeUInt32LE(i,off);off+=4}for(const v of ns){out.writeFloatLE(v,off);off+=4}process.stdout.write(out)
