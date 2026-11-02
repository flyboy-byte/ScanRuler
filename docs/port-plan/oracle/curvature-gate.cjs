// SPDX-License-Identifier: AGPL-3.0-only
const {execFileSync}=require('node:child_process'),{readFileSync}=require('node:fs'),vm=require('node:vm')
const source=p=>execFileSync('git',['show','2cd89c6769a10c27ee7754b477ada521c9ef5cb7:'+p],{encoding:'utf8'})
const l=source('src/core/fit/linalg.ts'),p=source('src/core/fit/plane.ts')
const slice=(s,a,b)=>{const i=s.indexOf(a),j=s.indexOf(b,i);if(i<0||j<0)throw Error('Source boundary changed');return s.slice(i,j)}
let body=slice(l,'export function symmetricEigen3','\nexport interface EigenN')+slice(l,'export function normalizeOrZ','/** Any two unit vectors')+slice(l,'export function centroidOf','/** The points `idx`')+slice(p,'export function planeResidual','export interface ClippedPlaneFit')+l.slice(l.indexOf('export function rmsOver'))
for(const [a,b] of [
 ['export function symmetricEigen3(input: ArrayLike<number>): Eigen3 {','function symmetricEigen3(input) {'],
 ['const pairs: [number, number][]','const pairs'],
 [' as [number, number, number]',''],[' as [Vec3, Vec3, Vec3]',''],
 ['export function normalizeOrZ(v: Vec3): Vec3 {','function normalizeOrZ(v) {'],
 ['export function centroidOf(positions: Float32Array, idx: ArrayLike<number>): Vec3 {','function centroidOf(positions,idx) {'],
 ['export function planeResidual(p: Plane, x: number, y: number, z: number): number {','function planeResidual(p,x,y,z) {'],
 ['export function rmsOver(positions: Float32Array, idx: ArrayLike<number>, residual: (x: number, y: number, z: number) => number): number {','function rmsOver(positions,idx,residual) {'],
 ['export function curvedBeyondPlane(positions: Float32Array, idx: ArrayLike<number>, curvedRms: number): boolean {','function curvedBeyondPlane(positions,idx,curvedRms) {'],
 ['export function fitPlaneTLS(positions: Float32Array, idx: ArrayLike<number>): Plane | null {','function fitPlaneTLS(positions,idx) {']
]){if(!body.includes(a))throw Error('Source annotation changed: '+a);body=body.replace(a,b)}
const api=vm.runInNewContext(body+'\n({fitPlaneTLS,planeResidual,rmsOver,curvedBeyondPlane})',{Float64Array})
const bytes=readFileSync(0),mode=bytes[0],callback=bytes[1],threshold=bytes.readDoubleLE(2),count=bytes.readUInt32LE(10)
const copy=(a,b)=>bytes.buffer.slice(bytes.byteOffset+a,bytes.byteOffset+b)
const positions=new Float32Array(copy(14,14+count*4)),idx=new Uint32Array(copy(14+count*4,bytes.length))
let out
if(mode===0){
 const calls=[]
 const rms=api.rmsOver(positions,idx,(x,y,z)=>{
  calls.push(x,y,z)
  switch(callback){case 0:return x;case 1:return NaN;case 2:return x===0?NaN:y;case 3:return x>0?Number.MAX_VALUE:Infinity;case 4:return x-y+z;default:throw Error('Unknown callback')}
 })
 out=Buffer.alloc(8+calls.length*8);out.writeDoubleLE(rms,0)
 calls.forEach((v,i)=>out.writeDoubleLE(v,8+i*8))
}else if(mode===1){
 const fit=api.fitPlaneTLS(positions,idx)
 out=Buffer.alloc(10);out[0]=Number(api.curvedBeyondPlane(positions,idx,threshold));out[1]=Number(fit!==null)
 out.writeDoubleLE(fit?api.rmsOver(positions,idx,(x,y,z)=>api.planeResidual(fit,x,y,z)):0,2)
}else throw Error('Unknown mode')
process.stdout.write(out)
