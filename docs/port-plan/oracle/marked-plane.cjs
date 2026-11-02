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

const q=source('src/core/fit/fitPlaneFromSeed.ts')
let helpers=slice(q,'const FRAME_STEPS','/** Full auto-fit pipeline')
for(const[a,b]of[
 [`function patchExtents(
  positions: Float32Array,
  region: Uint32Array,
  p: Plane,
): { center: Vec3; basisU: Vec3; basisV: Vec3; extentU: number; extentV: number } | null {`,'function patchExtents(positions,region,p) {'],
 ['const sa: number[]','const sa'],['const sb: number[]','const sb'],
 ['const basisU: Vec3','const basisU'],['const basisV: Vec3','const basisV'],
 ['function orientToSurface(p: Plane, normals: Float32Array, region: Uint32Array): Plane {','function orientToSurface(p,normals,region) {'],
 [`function orientedExtremes(
  fin: { plane: Plane; min: number; max: number },
  plane: Plane,
): { residualMin: number; residualMax: number } {`,'function orientedExtremes(fin,plane) {']
]){if(!helpers.includes(a))throw Error('Frozen declaration changed: '+a);helpers=helpers.replace(a,b)}
let selected=q.slice(q.indexOf('export function fitPlaneOnSelection'))
selected=selected.replace(`export function fitPlaneOnSelection(
  g: MeshGraph,
  selection: Uint32Array,
  settings: FitSettings,
): PlaneFit & { region: Uint32Array } {`,'function fitPlaneOnSelection(g,selection,settings) {')
if(!selected.startsWith('function fitPlaneOnSelection'))throw Error('Frozen selection declaration changed')
let ortho=slice(l,'export function orthoBasis','/** The mean of the points').replace('export function orthoBasis(n: Vec3): [Vec3, Vec3] {','function orthoBasis(n) {').replace('const helper: Vec3','const helper')
let cross=slice(source('src/core/vec.ts'),'export const cross','export const len').replace('export const cross = (a: Vec3, b: Vec3): Vec3 =>','const cross = (a,b) =>')
let selection=source('src/core/fit/selection.ts').slice(source('src/core/fit/selection.ts').indexOf('export const MIN_SELECTED'))
selection=selection.replace('export const MIN_SELECTED','const MIN_SELECTED').replace('export function requireSelection(selection: Uint32Array, noun: string): void {','function requireSelection(selection,noun) {')
const error=source('src/core/fit/errors.ts').replace('export class FitError','class FitError')
const api=vm.runInNewContext([body,clipBody,wrapper,cross,ortho,error,selection,helpers,selected,'({fitPlaneOnSelection})'].join('\n'),{Float64Array,Uint32Array})
const bytes=readFileSync(0),count=bytes.readUInt32LE(0),sigma=bytes.readDoubleLE(4),copy=(a,b)=>bytes.buffer.slice(bytes.byteOffset+a,bytes.byteOffset+b)
const positions=new Float32Array(copy(12,12+count*4)),normals=new Float32Array(copy(12+count*4,12+count*8)),idx=new Uint32Array(copy(12+count*8,bytes.length))
const chunks=[],u32=v=>{const b=Buffer.alloc(4);b.writeUInt32LE(v);chunks.push(b)},number=v=>{const b=Buffer.alloc(8);b.writeDoubleLE(v);chunks.push(b)},text=s=>{const b=Buffer.from(s);u32(b.length);chunks.push(b)}
try{
 const r=api.fitPlaneOnSelection({positions,normals},idx,{sigma});chunks.push(Buffer.from([0]));text(r.kind)
 for(const v of [...r.center,...r.normal,...r.basisU,...r.basisV,r.extentU,r.extentV,r.sigma,r.formError,r.residualMin,r.residualMax])number(v)
 u32(r.usedPoints);u32(r.regionSize);u32(r.region.length);r.region.forEach(u32)
}catch(e){chunks.push(Buffer.from([1]));text(e.message)}
for(let t=0;t<45;t++){const a=(t/45)*(Math.PI/2);number(Math.cos(a));number(Math.sin(a))}
process.stdout.write(Buffer.concat(chunks))
