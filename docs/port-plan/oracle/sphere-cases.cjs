// SPDX-License-Identifier: AGPL-3.0-only
// Independently authored input model, never any reference result.
function number(s){return s==='nan'?NaN:s==='inf'?Infinity:s==='-inf'?-Infinity:Number(s)}
function model(line){
 const [id,m,s,k,h]=line.split('\t'),mode=+m,seed=+s;let state=seed,p=[]
 const next=()=>state=(Math.imul(state,1664525)+1013904223)>>>0
 const axis=()=>((next()%2049)-1024)
 if(mode===1)p=[0,0,0,1,0,0,0,1,0]
 else if(mode===2)p=Array(48).fill(1)
 else if(mode===3)for(let i=0;i<20;i++)p.push(i,2*i,-3*i)
 else if(mode===4)for(let i=0;i<32;i++)p.push(i%8,Math.floor(i/8),5)
 else if(mode===5)p=[1,1,1,1,-1,-1,-1,1,-1,-1,-1,1]
 else if(mode===13)p=[10,0,0,-10,0,0,0,10,0,0,-10,0,0,0,10,0,0,-10,0,0,0]
 else if(mode!==0){
  const radius=mode===11?1e-7:mode===12?1e12:10,center=mode===10?[1e6,-2e6,3e6]:[13,-7,3]
  for(let i=0;i<128;i++){
   const x=axis(),y=axis(),z=mode===8?Math.abs(axis())+2048:axis(),norm=Math.sqrt(x*x+y*y+z*z)
   let r=radius
   if(mode===7||mode===8||mode===9||mode===15)r+=((next()%1001)-500)/20000
   if(mode===9&&i%17===0)r+=2
   p.push(center[0]+r*x/norm,center[1]+r*y/norm,center[2]+r*z/norm)
  }
 }
 const positions=Float32Array.from(p)
 if(mode===14)positions[0]=seed===1?NaN:seed===7?Infinity:seed===99?-Infinity:3.4028234663852886e38
 let ids=Array.from({length:p.length/3},(_,i)=>i)
 if(mode===15){ids=ids.filter(i=>i%3!==0).reverse();ids.push(1,1,2)}
 return{id,positions,idx:Uint32Array.from(ids),k:number(k),held:h==='none'?undefined:number(h)}
}
function packet(m){
 const header=Buffer.alloc(17+(m.held===undefined?0:8));header.writeUInt32LE(m.positions.length);header.writeUInt32LE(m.idx.length,4);header.writeDoubleLE(m.k,8);header[16]=Number(m.held!==undefined);if(m.held!==undefined)header.writeDoubleLE(m.held,17)
 return Buffer.concat([header,Buffer.from(m.positions.buffer),Buffer.from(m.idx.buffer)])
}
module.exports={model,packet}
