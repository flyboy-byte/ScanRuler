// SPDX-License-Identifier: AGPL-3.0-only
const {execFileSync}=require('node:child_process'),{readFileSync}=require('node:fs'),vm=require('node:vm')
const source=p=>execFileSync('git',['show','2cd89c6769a10c27ee7754b477ada521c9ef5cb7:'+p],{encoding:'utf8'})
const l=source('src/core/fit/linalg.ts'),p=source('src/core/fit/plane.ts')
const slice=(s,a,b)=>{const i=s.indexOf(a),j=s.indexOf(b,i);if(i<0||j<0)throw Error('Source boundary changed');return s.slice(i,j)}
let body=slice(l,'export function symmetricEigen3','\nexport interface EigenN')+slice(l,'export function normalizeOrZ','/** Any two unit vectors')+slice(l,'export function centroidOf','/** The points `idx`')+slice(p,'export function planeResidual','/** How much nearer than the best plane')
for(const [a,b] of [
 ['export function symmetricEigen3(input: ArrayLike<number>): Eigen3 {','function symmetricEigen3(input) {'],
 ['const pairs: [number, number][]','const pairs'],
 [' as [number, number, number]',''],[' as [Vec3, Vec3, Vec3]',''],
 ['export function normalizeOrZ(v: Vec3): Vec3 {','function normalizeOrZ(v) {'],
 ['export function centroidOf(positions: Float32Array, idx: ArrayLike<number>): Vec3 {','function centroidOf(positions,idx) {'],
 ['export function planeResidual(p: Plane, x: number, y: number, z: number): number {','function planeResidual(p,x,y,z) {'],
 ['export function fitPlaneTLS(positions: Float32Array, idx: ArrayLike<number>): Plane | null {','function fitPlaneTLS(positions,idx) {']
]){if(!body.includes(a))throw Error('Source annotation changed: '+a);body=body.replace(a,b)}

const clip=source('src/core/fit/clip.ts')
let clipBody='function clippedRefit(positions,idx,k,fit,residual) {\n'+clip.slice(clip.indexOf('  let used:'))
clipBody=clipBody.replace('let used: Uint32Array','let used').replace('idx as ArrayLike<number>','idx').replace('let result: ClippedRefit<M> | null','let result')
let wrapper='function fitPlaneClipped(positions,idx,k) {\n'+slice(p,'  const r = clippedRefit<Plane>','/** How much nearer two planes').replace('clippedRefit<Plane>','clippedRefit')
const api=vm.runInNewContext(body+clipBody+wrapper+'\n({clippedRefit,fitPlaneClipped})',{Float64Array,Uint32Array})
const bytes=readFileSync(0),count=bytes.readUInt32LE(0),k=bytes.readDoubleLE(4),mode=bytes[12],copy=(a,b)=>bytes.buffer.slice(bytes.byteOffset+a,bytes.byteOffset+b)
const positions=new Float32Array(copy(13,13+count*4)),idx=new Uint32Array(copy(13+count*4,bytes.length))
let calls=0
const result=mode?api.clippedRefit(positions,idx,k,()=>{calls++;return mode===1&&calls>1?null:mode===4?calls-1:0},(model,x)=>mode===2||mode===4&&model>0||mode===3&&x<0?NaN:x):api.fitPlaneClipped(positions,idx,k)
const out=Buffer.alloc(1+(result?64+4+result.used.length*4:0)+4);out[0]=Number(result!==null);let off=1
if(result){const plane=mode?[0,0,0,result.model]:[result.plane.nx,result.plane.ny,result.plane.nz,result.plane.d];for(const v of [...plane,result.sigma,result.span,result.min,result.max]){out.writeDoubleLE(v,off);off+=8}out.writeUInt32LE(result.used.length,off);off+=4;for(const i of result.used){out.writeUInt32LE(i,off);off+=4}}
out.writeUInt32LE(calls,off);process.stdout.write(out)
