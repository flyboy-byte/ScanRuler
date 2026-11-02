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

const r=source('src/core/fit/ransac.ts'),bounds=source('src/core/geometry/bounds.ts')
let rng=slice(r,'export function mulberry32','export interface RansacConsensus').replace('export function mulberry32(seed: number): () => number {','function mulberry32(seed) {')
let consensus='function ransacConsensus(positions,patch,opts,shape) {\n'+slice(r,'  const n = patch.length','/** Exact sphere through 4 points')
consensus=consensus.replace('const medianResidual = (m: M): number => {','const medianResidual = (m) => {').replace('let best: M | null','let best')
let extent=slice(bounds,'export function extentOfIndexed','export function boundsOf').replace('export function extentOfIndexed(positions: Float32Array, idx: ArrayLike<number>): Extent {','function extentOfIndexed(positions,idx) {').replaceAll(': Vec3','')
let robust='function ransacPlane(positions,patch,opts={}) {\n'+p.slice(p.indexOf('  const core = ransacConsensus<Plane>')).replace('ransacConsensus<Plane>','ransacConsensus')
const api=vm.runInNewContext(body+clipBody+wrapper+extent+rng+consensus+robust+'\n({mulberry32,ransacConsensus,ransacPlane,planeResidual})',{Float64Array,Uint32Array})
const bytes=readFileSync(0),mode=bytes[0],n=bytes.readUInt32LE(1),seed=bytes.readUInt32LE(5),iterations=bytes.readUInt32LE(9)
if(mode===0){const rand=api.mulberry32(seed),out=Buffer.alloc(iterations*8);for(let i=0;i<iterations;i++)out.writeDoubleLE(rand(),i*8);process.stdout.write(out)}else{
const copy=(a,b)=>bytes.buffer.slice(bytes.byteOffset+a,bytes.byteOffset+b),positions=new Float32Array(copy(13,13+n*4)),idx=new Uint32Array(copy(13+n*4,bytes.length));let calls=0
const core=mode===2?api.ransacConsensus(positions,idx,{iterations,seed},()=>({initial:{nx:1,ny:0,nz:0,d:0},generate:()=>{calls++;return{nx:0,ny:1,nz:0,d:0}},residual:api.planeResidual})):null
const result=mode===2?(core?{plane:core.model,sigma:core.sigmaEst,inliers:core.inliers}:null):api.ransacPlane(positions,idx,{iterations,seed})
const out=Buffer.alloc(1+(result?44+result.inliers.length*4:0)+4);out[0]=Number(result!==null);let off=1
if(result){for(const v of[result.plane.nx,result.plane.ny,result.plane.nz,result.plane.d,result.sigma]){out.writeDoubleLE(v,off);off+=8}out.writeUInt32LE(result.inliers.length,off);off+=4;for(const i of result.inliers){out.writeUInt32LE(i,off);off+=4}}
out.writeUInt32LE(calls,off);process.stdout.write(out)
}
