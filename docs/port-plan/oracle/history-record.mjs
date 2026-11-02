// SPDX-License-Identifier: AGPL-3.0-only
// Full pinned history/store graph, real callback wrappers and controlled promises.
import {readFileSync}from'node:fs'
import {pathToFileURL}from'node:url'
import {resolve}from'node:path'
const bytes=readFileSync(0),url=pathToFileURL(resolve('rust/target/history-reference/dist-session/history.mjs')).href
const word=n=>{const b=Buffer.alloc(4);b.writeUInt32LE(n);return b},text=s=>Buffer.concat([word(s.length),Buffer.from(s,'utf16le')])
function encode(v){if(v===undefined)return Buffer.from([0]);if(v===null)return Buffer.from([1]);if(typeof v==='boolean')return Buffer.from([v?3:2]);if(typeof v==='number'){const b=Buffer.alloc(9);b[0]=4;b.writeDoubleLE(v,1);return b}if(typeof v==='string')return Buffer.concat([Buffer.from([5]),text(v)]);if(v instanceof Uint8Array)return Buffer.concat([Buffer.from([8]),word(v.length),Buffer.from(v)]);if(Array.isArray(v))return Buffer.concat([Buffer.from([6]),word(v.length),...v.map(encode)]);return Buffer.concat([Buffer.from([7]),word(Object.keys(v).length),...Object.entries(v).flatMap(([k,v])=>[text(k),encode(v)])])}
function decode(b){let at=0;const n=()=>{const x=b.readUInt32LE(at);at+=4;return x},str=()=>{const n_=n(),s=b.subarray(at,at+n_*2).toString('utf16le');at+=n_*2;return s};function read(){const t=b[at++];switch(t){case 0:return undefined;case 1:return null;case 2:return false;case 3:return true;case 4:{const x=b.readDoubleLE(at);at+=8;return x}case 5:return str();case 6:return Array.from({length:n()},read);case 7:{const o={};let c=n();while(c--){const k=str();Object.defineProperty(o,k,{value:read(),enumerable:true,writable:true,configurable:true})}return o}case 8:{const c=n(),x=new Uint8Array(b.subarray(at,at+c));at+=c;return x}default:throw Error('Input tag')}}const result=read();if(at!==b.length)throw Error('Trailing input');return result}
let cursor=0,scenario=0
while(cursor<bytes.length){const size=bytes.readUInt32LE(cursor);cursor+=4;const input=decode(bytes.subarray(cursor,cursor+size));cursor+=size
 const api=await import(url+'?case='+scenario++),pool=[],identities=new Map(),log=[],frames=[],pending=new Map(),flags={busy:false,hook:false,participant:false},current=input.sections.map(s=>s.value),modes=input.sections.map(s=>s.identity??false);let capture=0,failCapture=0,failCode=null,unregister
 const atom=v=>v&&typeof v==='object'&&!(v instanceof Uint8Array)?pool[v.ref]:v
 for(const node of input.nodes){let value;if(node.kind==='object')value=Object.fromEntries(node.fields.map(([k,v])=>[k,atom(v)]));else if(node.kind==='array')value=node.items.map(atom);else if(node.kind==='bytes')value=new Uint8Array(node.values);else if(node.kind==='words')value=Uint32Array.from(node.values);else throw Error('Node kind');identities.set(value,pool.length);pool.push(value)}
 function cell(v){if(v===null||typeof v!=='object')return v;const id=identities.get(v);if(id===undefined)throw Error('Unexpected source patch payload');if(Array.isArray(v))return{id,kind:'array',items:v.map(cell)};if(v instanceof Uint8Array)return{id,kind:'bytes',values:v};if(v instanceof Uint32Array)return{id,kind:'words',values:[...v]};return{id,kind:'object',fields:Object.entries(v).map(([k,x])=>[k,cell(x)])}}
 function captureAt(i){capture++;log.push('capture:'+input.sections[i].key+':'+api.historyBlocked());if(capture===failCapture){if(failCode)throw new api.CommandError(failCode,'capture failed 🦀');throw Error('capture failed 🦀')};return atom(current[i])}
 const scope=input.sections.findIndex(s=>s.key==='scope'),scan=input.sections.findIndex(s=>s.key==='scan')
 api.configureHistory({blocked:()=>flags.hook,...(scope<0?{}:{getScope:()=>captureAt(scope)}),...(scan<0?{}:{getScan:()=>captureAt(scan)})})
 function install(){unregister?.();unregister=api.registerHistory({blocked:()=>flags.participant,sections:input.sections.flatMap((s,i)=>['scope','scan'].includes(s.key)?[]:[{key:s.key,capture:()=>captureAt(i),compare:modes[i]?'identity':'fields',apply:()=>{throw Error('travel is outside this corpus')}}])})}
 install();api.useShell.getState().setWorkspace(input.workspace??'elements')
 function patch(p){return Object.entries(p).map(([k,v])=>[k,cell(v)])}
 function state(error){const s=api.useHistory.getState();return{error,workspace:api.useShell.getState().workspace,past:s.past.map(e=>({before:patch(e.before),after:patch(e.after),label:e.label,workspace:e.workspace,group:e.group})),future:s.future.map(e=>({before:patch(e.before),after:patch(e.after),label:e.label,workspace:e.workspace,group:e.group})),revision:s.revision,restoring:s.restoring,importing:s.importing,blocked:api.historyBlocked(),current:input.sections.map((s,i)=>({key:s.key,identity:modes[i],value:cell(atom(current[i]))})),pending:[...pending.keys()],log:[...log]}}
 function sync(op){
  if(op.op==='set'){current[op.section]=op.value;log.push('set:'+op.section)}
  else if(op.op==='workspace')api.useShell.getState().setWorkspace(op.value)
  else if(op.op==='group')api.beginHistoryGroup()
  else if(op.op==='end_group')api.endHistoryGroup()
  else if(op.op==='clear')api.clearHistory()
  else if(op.op==='mode'){modes[op.section]=op.identity;install()}
  else if(op.op==='flags'){Object.assign(flags,op.value);if('busy'in op.value)api.useStore.setState({busy:flags.busy});if('restoring'in op.value)api.useHistory.setState({restoring:op.value.restoring});if('importing'in op.value)api.useHistory.setState({importing:op.value.importing})}
  else if(op.op==='capture_fail'){failCapture=capture+op.offset;failCode=op.code??null}
  else if(op.op==='seed_future'){const s=api.useHistory.getState();if(!s.past.length)throw Error('No past entry');api.useHistory.setState({future:[s.past.at(-1)]})}
  else if(op.op==='throw'){if(op.code)throw new api.CommandError(op.code,op.message??'body failed 🦀');throw Error(op.message??'body failed 🦀')}
  else if(op.op==='sync'){log.push('sync:'+op.label);try{api.historyAction(op.label,()=>{for(const step of op.steps)sync(step)})}finally{log.push('sync-end:'+op.label)}}
  else if(op.op==='start_async'){
   if(pending.has(op.id))throw Error('Duplicate fixture promise')
   let release;const gate=new Promise(r=>release=r);log.push('async:'+op.label)
   const promise=api.historyAsync(op.label,async()=>{for(const step of op.enter??[])sync(step);await gate;for(const step of op.leave??[])sync(step)});promise.catch(()=>{});pending.set(op.id,{release,promise})
  }else throw Error('Unknown sync fixture operation')
 }
 for(const op of input.ops){let error;try{if(op.op==='finish_async'){const p=pending.get(op.id);if(!p)throw Error('Missing fixture promise');pending.delete(op.id);p.release();try{await p.promise}finally{log.push('async-end:'+op.id)}}else sync(op)}catch(e){error={name:e.name,message:e.message,...(e.code===undefined?{}:{code:e.code})}}frames.push(state(error))}
 if(pending.size)throw Error('Unfinished fixture bodies')
 const output=Buffer.concat([Buffer.from('HRC1'),encode(frames)]);process.stdout.write(Buffer.concat([word(output.length),output]))
}
