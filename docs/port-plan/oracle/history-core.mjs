// SPDX-License-Identifier: AGPL-3.0-only
// Full pinned history/store graph, real callback wrappers and controlled promises.
import {readFileSync}from'node:fs'
import {pathToFileURL}from'node:url'
import {resolve}from'node:path'
const bytes=readFileSync(0),url=pathToFileURL(resolve('rust/target/history-core-reference/dist-session/history.mjs')).href
const word=n=>{const b=Buffer.alloc(4);b.writeUInt32LE(n);return b},text=s=>Buffer.concat([word(s.length),Buffer.from(s,'utf16le')])
function encode(v){if(v===undefined)return Buffer.from([0]);if(v===null)return Buffer.from([1]);if(typeof v==='boolean')return Buffer.from([v?3:2]);if(typeof v==='number'){const b=Buffer.alloc(9);b[0]=4;b.writeDoubleLE(v,1);return b}if(typeof v==='string')return Buffer.concat([Buffer.from([5]),text(v)]);if(v instanceof Uint8Array)return Buffer.concat([Buffer.from([8]),word(v.length),Buffer.from(v)]);if(Array.isArray(v))return Buffer.concat([Buffer.from([6]),word(v.length),...v.map(encode)]);return Buffer.concat([Buffer.from([7]),word(Object.keys(v).length),...Object.entries(v).flatMap(([k,v])=>[text(k),encode(v)])])}
function decode(b){let at=0;const n=()=>{const x=b.readUInt32LE(at);at+=4;return x},str=()=>{const n_=n(),s=b.subarray(at,at+n_*2).toString('utf16le');at+=n_*2;return s};function read(){const t=b[at++];switch(t){case 0:return undefined;case 1:return null;case 2:return false;case 3:return true;case 4:{const x=b.readDoubleLE(at);at+=8;return x}case 5:return str();case 6:return Array.from({length:n()},read);case 7:{const o={};let c=n();while(c--){const k=str();Object.defineProperty(o,k,{value:read(),enumerable:true,writable:true,configurable:true})}return o}case 8:{const c=n(),x=new Uint8Array(b.subarray(at,at+c));at+=c;return x}default:throw Error('Input tag')}}const result=read();if(at!==b.length)throw Error('Trailing input');return result}
let cursor=0,scenario=0
while(cursor<bytes.length){
 const size=bytes.readUInt32LE(cursor);cursor+=4;const input=decode(bytes.subarray(cursor,cursor+size));cursor+=size
 const api=await import(url+'?case='+scenario++),pool=[],identities=new Map(),log=[],frames=[]
 const atom=v=>v&&typeof v==='object'&&!(v instanceof Uint8Array)?pool[v.ref]:v
 for(const node of input.nodes){let value;if(node.kind==='object')value=Object.fromEntries(node.fields.map(([k,v])=>[k,atom(v)]));else if(node.kind==='array')value=node.items.map(atom);else if(node.kind==='bytes')value=new Uint8Array(node.values);else if(node.kind==='words')value=Uint32Array.from(node.values);else throw Error('Node kind');identities.set(value,pool.length);pool.push(value)}
 let next=pool.length
 function cell(v){if(v===null||typeof v!=='object')return v;if(!identities.has(v))identities.set(v,next++);const id=identities.get(v);if(Array.isArray(v))return{id,kind:'array',items:v.map(cell)};if(v instanceof Uint8Array)return{id,kind:'bytes',values:v};if(v instanceof Uint32Array)return{id,kind:'words',values:[...v]};return{id,kind:'object',fields:Object.entries(v).map(([k,v])=>[k,cell(v)])}}
 const stores=[api.useStore,api.useFlat,api.useDeviation,api.useThickness],names=['measure','flat','deviation','thickness'],keys=input.stores.map(fields=>fields.map(([k])=>k)),slots=input.participants.map(()=>undefined)
 for(let i=0;i<4;i++)stores[i].setState(Object.fromEntries(input.stores[i].map(([k,v])=>[k,atom(v)])))
 let scope=atom(input.scope),scan=atom(input.scan)
 function event(name){log.push(name+':'+api.useFlat.getState().subjectVersion+':'+api.useDeviation.getState().scopeVersion)}
 function fail(name){if(input.fault===name)throw Error('callback '+name+'\ud800')}
 api.configureHistory({getScope(){event('scope-get');fail('scope-get');return scope},getScan(){event('scan-get');fail('scan-get');return scan},setScope(value){event('scope-set');fail('scope-set');scope=value;fail('scope-after')}})
 input.participants.forEach((key,i)=>api.registerHistory({sections:[{key,capture:()=>slots[i],apply(value){event('participant-'+i);fail('participant-'+i);slots[i]=value}}]}))
 function state(){return{stores:stores.map((s,i)=>keys[i].map(k=>[k,cell(s.getState()[k])])),scope:cell(scope),scan:cell(scan),participants:slots.map(cell),log:[...log]}}
 for(const op of input.ops){let error,snapshot
  try{if(op.op==='capture')snapshot=Object.entries(api.captureCore()).map(([k,v])=>[k,cell(v)])
   else{const patch=Object.fromEntries(op.patch.map(([k,v])=>[k,atom(v)]));for(let i=0;i<4;i++){const value=patch[names[i]];if(value&&typeof value==='object')for(const k of Object.keys(value))if(!keys[i].includes(k))keys[i].push(k)}api.apply(patch)}
  }catch(e){error={name:e.name,message:e.message}}
  frames.push({error,snapshot,...state()})
 }
 const output=Buffer.concat([Buffer.from('HCP1'),encode(frames)]);process.stdout.write(Buffer.concat([word(output.length),output]))
}
