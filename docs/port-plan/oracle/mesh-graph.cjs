// SPDX-License-Identifier: AGPL-3.0-only
// Full frozen pipeline and dependency bodies, with TS declarations removed only.
const {execFileSync}=require('node:child_process'),{readFileSync}=require('node:fs'),vm=require('node:vm')
const source=p=>execFileSync('git',['show','2cd89c6769a10c27ee7754b477ada521c9ef5cb7:src/core/geometry/'+p+'.ts'],{encoding:'utf8'})
function replace(s,a,b){if(!s.includes(a))throw Error('Frozen declaration changed: '+a);return s.replace(a,b)}
function slice(s,a,b){const i=s.indexOf(a),j=b?s.indexOf(b,i):s.length;if(i<0||j<0)throw Error('Frozen boundary changed');return s.slice(i,j)}
const bounds=source('bounds'),centroid=source('centroid'),normals=source('normals')
const extentSig='export function extentOf(positions: Float32Array, count = positions.length / 3): Extent {'
const extent=replace(slice(bounds,extentSig,'\n/** The extent of the points'),extentSig,'function extentOf(positions,count=positions.length/3) {').replaceAll(': Vec3','')
const tetraSig='export function tetraVolume6(p: Float32Array, a: number, b: number, c: number, o: Vec3 = ORIGIN): number {'
const tetra=replace(slice(centroid,tetraSig),tetraSig,'function tetraVolume6(p,a,b,c,o=[0,0,0]) {')
const normalSig='export function computeVertexNormals(positions: Float32Array, indices: Uint32Array): Float32Array {'
const compute=replace(slice(normals,normalSig,'\n/**'),normalSig,'function computeVertexNormals(positions,indices) {')
const orientSig='export function orientNormalsOutward(\n  positions: Float32Array,\n  indices: Uint32Array,\n  normals: Float32Array,\n): boolean {'
const orient=replace(slice(normals,orientSig),orientSig,'function orientNormalsOutward(positions,indices,normals) {').replace(': [number, number, number]','')
let weld=source('weld')
for(const[a,b]of[
 ['export function weldTriangleSoup(\n  soup: Float32Array,\n  onProgress?: (text: string) => void,\n): { positions: Float32Array; indices: Uint32Array } {','function weldTriangleSoup(soup,onProgress) {'],
 ['function hash3(a: number, b: number, c: number): number {','function hash3(a,b,c) {'],
 ['export function filterDegenerateTriangles(indices: Uint32Array): Uint32Array {','function filterDegenerateTriangles(indices) {']
])weld=replace(weld,a,b)
const adjacency=replace(source('adjacency'),'export function buildAdjacency(\n  indices: Uint32Array,\n  vertexCount: number,\n): { offsets: Uint32Array; list: Uint32Array } {','function buildAdjacency(indices,vertexCount) {')
let graph=source('buildGraph').replace(/^import .*\n/gm,'')
graph=replace(graph,'export function buildMeshGraph(parsed: ParsedMesh, onProgress?: (text: string) => void): MeshGraph {','function buildMeshGraph(parsed,onProgress) {')
graph=replace(graph,'let positions: Float32Array','let positions')
graph=replace(graph,'let indices: Uint32Array','let indices')
const build=vm.runInNewContext([extent,tetra,compute,orient,weld,adjacency,graph,'buildMeshGraph'].join('\n'),{Float32Array,Uint32Array,Int32Array})
const input=readFileSync(0),soup=input[0]!==0,keep=input[1]!==0,hasIndices=input[2]!==0,n=input.readUInt32LE(4)
const copy=(a,b)=>input.buffer.slice(input.byteOffset+a,input.byteOffset+b)
const parsed={kind:soup?'soup':'indexed',keepWinding:keep,positions:new Float32Array(copy(8,8+n*4))}
if(hasIndices)parsed.indices=new Uint32Array(copy(8+n*4,input.length))
const chunks=[],progress=[]
const u32=v=>{const b=Buffer.alloc(4);b.writeUInt32LE(v);chunks.push(b)}
const text=s=>{const b=Buffer.from(s);u32(b.length);chunks.push(b)}
try{
 const g=build(parsed,s=>progress.push(s));chunks.push(Buffer.from([0]));u32(g.vertexCount);u32(g.triangleCount)
 const diag=Buffer.alloc(8);diag.writeDoubleLE(g.bboxDiag);chunks.push(diag)
 for(const[name,float]of[['positions',true],['indices',false],['normals',true],['adjOffsets',false],['adjList',false]]){
  const a=g[name];u32(a.length);const b=Buffer.alloc(a.length*4);a.forEach((v,i)=>float?b.writeFloatLE(v,i*4):b.writeUInt32LE(v,i*4));chunks.push(b)
 }
}catch(error){chunks.push(Buffer.from([1]));text(error.message)}
u32(progress.length);progress.forEach(text);process.stdout.write(Buffer.concat(chunks))
