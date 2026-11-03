// SPDX-License-Identifier: AGPL-3.0-only
// Full pinned history/store graph, real callback wrappers and controlled promises.
import {readFileSync}from'node:fs'
import {pathToFileURL}from'node:url'
import {resolve}from'node:path'
const bytes=readFileSync(0),url=pathToFileURL(resolve('rust/target/deadline-reference/dist-session/registry.mjs')).href
const word=n=>{const b=Buffer.alloc(4);b.writeUInt32LE(n);return b},text=s=>Buffer.concat([word(s.length),Buffer.from(s,'utf16le')])
function encode(v){if(v===undefined)return Buffer.from([0]);if(v===null)return Buffer.from([1]);if(typeof v==='boolean')return Buffer.from([v?3:2]);if(typeof v==='number'){const b=Buffer.alloc(9);b[0]=4;b.writeDoubleLE(v,1);return b}if(typeof v==='string')return Buffer.concat([Buffer.from([5]),text(v)]);if(v instanceof Uint8Array)return Buffer.concat([Buffer.from([8]),word(v.length),Buffer.from(v)]);if(Array.isArray(v))return Buffer.concat([Buffer.from([6]),word(v.length),...v.map(encode)]);return Buffer.concat([Buffer.from([7]),word(Object.keys(v).length),...Object.entries(v).flatMap(([k,v])=>[text(k),encode(v)])])}
function decode(b){let at=0;const n=()=>{const x=b.readUInt32LE(at);at+=4;return x},str=()=>{const n_=n(),s=b.subarray(at,at+n_*2).toString('utf16le');at+=n_*2;return s};function read(){const t=b[at++];switch(t){case 0:return undefined;case 1:return null;case 2:return false;case 3:return true;case 4:{const x=b.readDoubleLE(at);at+=8;return x}case 5:return str();case 6:return Array.from({length:n()},read);case 7:{const o={};let c=n();while(c--){const k=str();Object.defineProperty(o,k,{value:read(),enumerable:true,writable:true,configurable:true})}return o}case 8:{const c=n(),x=new Uint8Array(b.subarray(at,at+c));at+=c;return x}default:throw Error('Input tag')}}const result=read();if(at!==b.length)throw Error('Trailing input');return result}
const realSetTimeout=globalThis.setTimeout,realClearTimeout=globalThis.clearTimeout
let cursor=0,scenario=0
while(cursor<bytes.length){
 const size=bytes.readUInt32LE(cursor);cursor+=4;const input=decode(bytes.subarray(cursor,cursor+size));cursor+=size
 const api=await import(url+'?case='+scenario++),abort=new AbortController(),log=[],frames=[],unhandled=[]
 let timerActive=false,timerCallback,timerId=0,caller={status:'pending'},body={status:'pending'},doc=0,release,reject
 const promise=new Promise((resolve,reject_)=>{release=resolve;reject=reject_})
 const bodyError=input.body_kind==='typed'?new api.CommandError('invalid_state','body failed \ud800'):input.body_kind==='typeerror'?new TypeError('body failed \ud800'):new Error('body failed \ud800')
 function error(e){return{name:e.name,message:e.message,...(e.code===undefined?{}:{code:e.code}),sameReason:e===abort.signal.reason}}
 function resolveBody(value=input.value){if(body.status!=='pending')return;body={status:'ok',value};doc=42;log.push({event:'body:resolved',value});release(value)}
 function rejectBody(e=bodyError){if(body.status!=='pending')return;body={status:'error',error:e};doc=-1;log.push({event:'body:rejected',error:error(e)});reject(e)}
 function fire(){if(!timerActive){log.push({event:'timer:ignored'});return}timerActive=false;log.push({event:'timer:fire',id:timerId});timerCallback()}
 abort.signal.addEventListener('abort',()=>{log.push({event:'abort',reason:error(abort.signal.reason)});if(input.abort==='resolve')resolveBody(input.fallback);else if(input.abort==='reject-reason')rejectBody(abort.signal.reason)})
 const onUnhandled=e=>unhandled.push(error(e));process.on('unhandledRejection',onUnhandled)
 globalThis.setTimeout=(callback,ms)=>{timerId=1;timerActive=true;timerCallback=callback;log.push({event:'timer:set',id:timerId,ms});return timerId}
 globalThis.clearTimeout=id=>{log.push({event:'timer:clear',id});timerActive=false}
 const run=()=>{
  log.push({event:'run'})
  if(input.mode==='throw'){body={status:'thrown',error:bodyError};doc=-2;log.push({event:'body:thrown',error:error(bodyError)});throw bodyError}
  if(input.mode==='resolved')resolveBody()
  else if(input.mode==='rejected')rejectBody()
  else if(input.mode==='fire-only')fire()
  else if(input.mode==='fire-resolve'){fire();resolveBody()}
  else if(input.mode==='resolve-fire'){resolveBody();fire()}
  else if(input.mode==='fire-reject'){fire();rejectBody()}
  else if(input.mode==='reject-fire'){rejectBody();fire()}
  return promise
 }
 const result=api.limited(input.name,input.ms,abort,run)
 result.then(value=>{caller={status:'ok',value};log.push({event:'caller:ok'})},e=>{caller={status:'error',error:e};log.push({event:'caller:error'})})
 async function flush(){for(let i=0;i<20;i++)await Promise.resolve();await new Promise(resolve=>setImmediate(resolve))}
 function state(){return{caller:caller.status==='error'?{status:'error',error:error(caller.error)}:{...caller},body:body.status==='error'||body.status==='thrown'?{status:body.status,error:error(body.error)}:{...body},doc,aborted:abort.signal.aborted,reason:abort.signal.aborted?error(abort.signal.reason):undefined,timerActive,log:[...log],unhandled:[...unhandled]}}
 await flush();frames.push(state())
 for(const op of input.ops){if(op==='fire')fire();else if(op==='resolve')resolveBody();else if(op==='reject')rejectBody();else if(op==='resolve-fire'){resolveBody();fire()}else if(op==='fire-resolve'){fire();resolveBody()}else if(op==='reject-fire'){rejectBody();fire()}else if(op==='fire-reject'){fire();rejectBody()}else throw Error('Fixture operation');await flush();frames.push(state())}
 globalThis.setTimeout=realSetTimeout;globalThis.clearTimeout=realClearTimeout;process.removeListener('unhandledRejection',onUnhandled)
 if(caller.status==='pending'||body.status==='pending'||unhandled.length)throw Error('Incomplete/unhandled deadline fixture')
 const output=Buffer.concat([Buffer.from('CDL1'),encode({defaultTimeoutMs:api.COMMAND_TIMEOUT_MS,frames})]);process.stdout.write(Buffer.concat([word(output.length),output]))
}
