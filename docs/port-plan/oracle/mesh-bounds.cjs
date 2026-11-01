// SPDX-License-Identifier: AGPL-3.0-only
const {execFileSync}=require('node:child_process'),{readFileSync}=require('node:fs'),vm=require('node:vm')
const source=execFileSync('git',['show','2cd89c6769a10c27ee7754b477ada521c9ef5cb7:src/core/geometry/bounds.ts'],{encoding:'utf8'})
let body=source.slice(source.indexOf('export function extentOf('))
for(const[a,b]of[
 ['export function extentOf(positions: Float32Array, count = positions.length / 3): Extent {','function extentOf(positions,count=positions.length/3) {'],
 ['export function extentOfIndexed(positions: Float32Array, idx: ArrayLike<number>): Extent {','function extentOfIndexed(positions,idx) {'],
 ['export function boundsOf(positions: Float32Array, count = positions.length / 3): Bounds {','function boundsOf(positions,count=positions.length/3) {']
]){if(!body.includes(a))throw Error('Source signature changed');body=body.replace(a,b)}body=body.replaceAll(': Vec3','')
const api=vm.runInNewContext(body+'\n({extentOf,extentOfIndexed,boundsOf})',{})
const bytes=readFileSync(0),n=bytes.readUInt32LE(0),count=bytes.readUInt32LE(4),copy=(a,b)=>bytes.buffer.slice(bytes.byteOffset+a,bytes.byteOffset+b)
const p=new Float32Array(copy(8,8+n*4)),idx=new Uint32Array(copy(8+n*4,bytes.length)),e=api.extentOf(p,count),ei=api.extentOfIndexed(p,idx),b=api.boundsOf(p,count)
const values=[...e.min,...e.max,...ei.min,...ei.max,...b.min,...b.max,...b.center,b.radius],out=Buffer.alloc(values.length*8)
values.forEach((v,i)=>out.writeDoubleLE(v,i*8));process.stdout.write(out)
