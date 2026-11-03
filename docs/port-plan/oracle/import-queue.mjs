// SPDX-License-Identifier: AGPL-3.0-only
// Full pinned history/store graph, real callback wrappers and controlled promises.
import {readFileSync}from'node:fs'
import {pathToFileURL}from'node:url'
import {resolve}from'node:path'
import vm from 'node:vm'
const bytes=readFileSync(0),url=pathToFileURL(resolve('rust/target/import-queue-reference/queue.cjs')).href
const word=n=>{const b=Buffer.alloc(4);b.writeUInt32LE(n);return b},text=s=>Buffer.concat([word(s.length),Buffer.from(s,'utf16le')])
function encode(v){if(v===undefined)return Buffer.from([0]);if(v===null)return Buffer.from([1]);if(typeof v==='boolean')return Buffer.from([v?3:2]);if(typeof v==='number'){const b=Buffer.alloc(9);b[0]=4;b.writeDoubleLE(v,1);return b}if(typeof v==='string')return Buffer.concat([Buffer.from([5]),text(v)]);if(v instanceof Uint8Array)return Buffer.concat([Buffer.from([8]),word(v.length),Buffer.from(v)]);if(Array.isArray(v))return Buffer.concat([Buffer.from([6]),word(v.length),...v.map(encode)]);return Buffer.concat([Buffer.from([7]),word(Object.keys(v).length),...Object.entries(v).flatMap(([k,v])=>[text(k),encode(v)])])}
function decode(b){let at=0;const n=()=>{const x=b.readUInt32LE(at);at+=4;return x},str=()=>{const n_=n(),s=b.subarray(at,at+n_*2).toString('utf16le');at+=n_*2;return s};function read(){const t=b[at++];switch(t){case 0:return undefined;case 1:return null;case 2:return false;case 3:return true;case 4:{const x=b.readDoubleLE(at);at+=8;return x}case 5:return str();case 6:return Array.from({length:n()},read);case 7:{const o={};let c=n();while(c--){const k=str();Object.defineProperty(o,k,{value:read(),enumerable:true,writable:true,configurable:true})}return o}case 8:{const c=n(),x=new Uint8Array(b.subarray(at,at+c));at+=c;return x}default:throw Error('Input tag')}}const result=read();if(at!==b.length)throw Error('Trailing input');return result}
const original=readFileSync(new URL(url),'utf8'),exportsObject={}
vm.runInNewContext(original,{exports:exportsObject,Promise,Set})
const {ImportQueue}=exportsObject
let cursor=0
while(cursor<bytes.length){
 const size=bytes.readUInt32LE(cursor);cursor+=4;const input=decode(bytes.subarray(cursor,cursor+size));cursor+=size
 const q=new ImportQueue(),log=[],tokens=new Map(),results=new Map(),controls=new Map(),hits=input.listeners.map(()=>0),callbacks=[],frames=[]
 for(const job of input.jobs){let release;const gate=new Promise(r=>release=r);controls.set(job.id,{job,gate,release})}
 function subscribe(id,token){tokens.set(token,q.subscribe(callbacks[id]))}
 function run(id){if(results.has(id))throw Error('Duplicate fixture job');const c=controls.get(id);const promise=q.run(async()=>{log.push('start:'+id+':'+q.pending);if(!c.job.immediate)await c.gate;log.push('finish:'+id+':'+q.pending);if(c.job.fail)throw Error('job '+id+'\ud800');return c.job.value});const result={status:'pending'};results.set(id,result);promise.then(value=>{result.status='ok';result.value=value;log.push('settled:'+id+':ok')},error=>{result.status='error';result.error={name:error.name,message:error.message};log.push('settled:'+id+':error')})}
 for(let i=0;i<input.listeners.length;i++)callbacks.push(()=>{const hit=++hits[i],def=input.listeners[i];log.push('notify:'+i+':'+hit+':'+q.pending+':'+q.busy);if(def.action?.at===hit){const a=def.action;if(a.kind==='add')subscribe(a.listener,a.token);else if(a.kind==='remove')tokens.get(a.token)();else if(a.kind==='enqueue')run(a.job);else throw Error('Fixture action')}if(def.throwAt===hit)throw Error('listener '+i+'\ud800')})
 for(const op of input.ops){let error
  try{if(op.op==='subscribe')subscribe(op.listener,op.token);else if(op.op==='unsubscribe')tokens.get(op.token)();else if(op.op==='run')run(op.job);else if(op.op==='resolve')controls.get(op.job).release();else throw Error('Fixture op')}catch(e){error={name:e.name,message:e.message}}
  for(let turn=0;turn<500;turn++)await Promise.resolve()
  frames.push({error,busy:q.busy,pending:q.pending,log:[...log],results:[...results].map(([id,result])=>({id,...result}))})
 }
 if([...results.values()].some(r=>r.status==='pending'))throw Error('Unfinished source fixture job')
 const output=Buffer.concat([Buffer.from('IQU1'),encode(frames)]);process.stdout.write(Buffer.concat([word(output.length),output]))
}
