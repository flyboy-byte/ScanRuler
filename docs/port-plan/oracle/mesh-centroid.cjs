// SPDX-License-Identifier: AGPL-3.0-only
const {execFileSync}=require('node:child_process'),{readFileSync}=require('node:fs'),vm=require('node:vm')
const source=p=>execFileSync('git',['show','2cd89c6769a10c27ee7754b477ada521c9ef5cb7:'+p],{encoding:'utf8'})
const b=source('src/core/geometry/bounds.ts'),c=source('src/core/geometry/centroid.ts')
let extent=b.slice(b.indexOf('export function extentOfIndexed'),b.indexOf('export function boundsOf')).replace('export function extentOfIndexed(positions: Float32Array, idx: ArrayLike<number>): Extent {','function extentOfIndexed(positions,idx) {').replaceAll(': Vec3','')
let body=c.slice(c.indexOf('const CLOSED_TOLERANCE'))
for(const[a,b]of[
 ['export function triangleAreaSum(positions: Float32Array, indices: Uint32Array): number {','function triangleAreaSum(positions,indices) {'],
 ['export function meshCentroid(positions: Float32Array, indices: Uint32Array): MeshCentroid {','function meshCentroid(positions,indices) {'],
 ['function tetraVolume(p: Float32Array, a: number, b: number, c: number, o: Vec3): number {','function tetraVolume(p,a,b,c,o) {'],
 ['export function tetraVolume6(p: Float32Array, a: number, b: number, c: number, o: Vec3 = ORIGIN): number {','function tetraVolume6(p,a,b,c,o=ORIGIN) {']
]){if(!body.includes(a))throw Error('Frozen signature changed');body=body.replace(a,b)}body=body.replaceAll(': Vec3 | null','').replaceAll(': Vec3','')
const api=vm.runInNewContext(extent+body+'\n({triangleAreaSum,meshCentroid})',{})
const bytes=readFileSync(0),n=bytes.readUInt32LE(0),copy=(a,b)=>bytes.buffer.slice(bytes.byteOffset+a,bytes.byteOffset+b),p=new Float32Array(copy(4,4+n*4)),idx=new Uint32Array(copy(4+n*4,bytes.length)),r=api.meshCentroid(p,idx)
const values=[api.triangleAreaSum(p,idx),...(r.volume||[]),...r.area,r.volumeMm3],out=Buffer.alloc(2+values.length*8);out[0]=Number(r.closed);out[1]=Number(r.volume!==null);values.forEach((v,i)=>out.writeDoubleLE(v,2+i*8));process.stdout.write(out)
