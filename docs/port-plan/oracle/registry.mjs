// SPDX-License-Identifier: AGPL-3.0-only
// Each scenario imports a fresh full original registry graph; only fixture
// callbacks echo independently supplied arguments. No real command is faked.
import {readFileSync} from 'node:fs'
import {resolve} from 'node:path'
import {pathToFileURL} from 'node:url'
const data=readFileSync(0),bundle=resolve('rust/target/registry-reference/dist-session/registry.mjs')
const u32=n=>{const b=Buffer.alloc(4);b.writeUInt32LE(n);return b},txt=s=>Buffer.concat([u32(s.length),Buffer.from(s,'utf16le')])
function encode(v){if(v===undefined)return Buffer.from([0]);if(v===null)return Buffer.from([1]);if(typeof v==='boolean')return Buffer.from([v?3:2]);if(typeof v==='number'){const b=Buffer.alloc(9);b[0]=4;b.writeDoubleLE(v,1);return b}if(typeof v==='string')return Buffer.concat([Buffer.from([5]),txt(v)]);if(v instanceof Uint8Array)return Buffer.concat([Buffer.from([8]),u32(v.length),Buffer.from(v)]);if(Array.isArray(v))return Buffer.concat([Buffer.from([6]),u32(v.length),...v.map(encode)]);return Buffer.concat([Buffer.from([7]),u32(Object.keys(v).length),...Object.entries(v).flatMap(([k,v])=>[txt(k),encode(v)])])}
function decode(b){let at=0;const n=()=>{const v=b.readUInt32LE(at);at+=4;return v},text=()=>{const len=n(),s=b.subarray(at,at+len*2).toString('utf16le');at+=len*2;return s};function read(){const tag=b[at++];switch(tag){case 0:return undefined;case 1:return null;case 2:return false;case 3:return true;case 4:{const x=b.readDoubleLE(at);at+=8;return x}case 5:return text();case 6:return Array.from({length:n()},read);case 7:{const o={};let len=n();while(len--){const k=text();Object.defineProperty(o,k,{value:read(),enumerable:true,writable:true,configurable:true})}return o}case 8:{const len=n(),v=new Uint8Array(b.subarray(at,at+len));at+=len;return v}default:throw Error('Value tag')}}const v=read();if(at!==b.length)throw Error('Trailing input');return v}
let cursor=0,index=0
while(cursor<data.length){const size=data.readUInt32LE(cursor);cursor+=4;const input=decode(data.subarray(cursor,cursor+size));cursor+=size
 const api=await import(pathToFileURL(bundle).href+'?scenario='+index++),cleanups=[],observations=[];let target
 for(const op of input.ops){target=undefined;let value,error
  try{
   if(op.op==='register'){const commands=op.indices.map(i=>({...input.defs[i],history:false,run:async args=>{target=i;return args}}));const clean=api.registerCommands(op.namespace,commands);value=cleanups.length;cleanups.push(clean)}
   else if(op.op==='cleanup'){cleanups[op.handle]();value=undefined}
   else if(op.op==='admit')value=await api.runCommand(op.name,op.input)
   else if(op.op==='headless')value=api.runsHeadless(op.name)
   else if(op.op!=='list')throw Error('Unknown fixture operation')
  }catch(e){error={name:e.name,message:e.message,...(e.code===undefined?{}:{code:e.code})}}
  const commands=api.listCommands();observations.push({value,error,afterInput:op.op==='admit'?op.input:undefined,target,commands,headless:commands.map(c=>api.runsHeadless(c.name))})
 }
 const frame=Buffer.concat([Buffer.from('REG3'),encode(observations)]);process.stdout.write(Buffer.concat([u32(frame.length),frame]))
}
