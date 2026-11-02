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
const api=vm.runInNewContext(body+'\n({fitPlaneTLS,planeResidual})',{Float64Array})
const bytes=readFileSync(0),count=bytes.readUInt32LE(0),copy=(a,b)=>bytes.buffer.slice(bytes.byteOffset+a,bytes.byteOffset+b)
const positions=new Float32Array(copy(4,4+count*4)),idx=new Uint32Array(copy(4+count*4,bytes.length)),fit=api.fitPlaneTLS(positions,idx)
const out=Buffer.alloc(1+(fit?(4+idx.length)*8:0));out[0]=Number(fit!==null)
if(fit){let off=1;for(const v of [fit.nx,fit.ny,fit.nz,fit.d]){out.writeDoubleLE(v,off);off+=8}for(const i of idx){out.writeDoubleLE(api.planeResidual(fit,positions[i*3],positions[i*3+1],positions[i*3+2]),off);off+=8}}
process.stdout.write(out)
