// SPDX-License-Identifier: AGPL-3.0-only
const {model:base,packet}=require('./sphere-cases.cjs')
function model(line){
 const f=line.split('\t'),m=base(line),mode=+f[1]
 if(mode>=16){const r=mode===17?0.03125:mode===18?1.03125:10;m.positions=Float32Array.from([r,0,0,-r,0,0,0,r,0,0,-r,0,0,0,r,0,0,-r]);m.idx=Uint32Array.from([0,1,2,3,4,5])}
 if(f[5]!=='default'){const previous=m.idx,n=+f[5];m.idx=Uint32Array.from({length:n},(_,i)=>previous[i%previous.length])}
 return m
}
module.exports={model,packet}
