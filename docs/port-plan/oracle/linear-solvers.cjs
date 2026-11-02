// SPDX-License-Identifier: AGPL-3.0-only
const {execFileSync}=require('node:child_process'),{readFileSync}=require('node:fs'),vm=require('node:vm')
const s=execFileSync('git',['show','2cd89c6769a10c27ee7754b477ada521c9ef5cb7:src/core/fit/linalg.ts'],{encoding:'utf8'})
const start=s.indexOf('export function choleskySolve'),end=s.indexOf('\nexport interface Eigen3')
let body=s.slice(start,end)
for(const [a,b]of[
 ['export function choleskySolve(n: number, a: Float64Array, b: Float64Array, tiny = 1e-12): Float64Array | null {','function choleskySolve(n,a,b,tiny=1e-12) {'],
 ['export function solveLinear(n: number, a: Float64Array, b: Float64Array): Float64Array | null {','function solveLinear(n,a,b) {']
]){if(!body.includes(a))throw Error('Frozen signature changed');body=body.replace(a,b)}
const api=vm.runInNewContext(body+'\n({choleskySolve,solveLinear})',{Float64Array})
const input=readFileSync(0),n=input.readUInt32LE(0),mode=input[4],tiny=input.readDoubleLE(5)
let off=13,a=new Float64Array(n*n),b=new Float64Array(n);for(let i=0;i<a.length;i++,off+=8)a[i]=input.readDoubleLE(off);for(let i=0;i<n;i++,off+=8)b[i]=input.readDoubleLE(off)
const x=mode?api.choleskySolve(n,a,b,tiny):api.solveLinear(n,a,b)
const out=Buffer.alloc(1+(x?x.length*8:0)+a.length*8);out[0]=Number(x!==null);off=1
if(x)for(const v of x){out.writeDoubleLE(v,off);off+=8}for(const v of a){out.writeDoubleLE(v,off);off+=8}process.stdout.write(out)
