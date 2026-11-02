// SPDX-License-Identifier: AGPL-3.0-only
// Full pinned module, TypeScript erasure only. No Rust answers or stubs.
const {execFileSync}=require('node:child_process'),fs=require('node:fs'),vm=require('node:vm')
const ts=require(process.cwd()+'/node_modules/typescript')
const pin='2cd89c6769a10c27ee7754b477ada521c9ef5cb7'
const source=execFileSync('git',['show',`${pin}:src/commands/schema.ts`],{encoding:'utf8'})
const js=ts.transpileModule(source,{compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.CommonJS}}).outputText
const exports_={};vm.runInNewContext(js,{exports:exports_,Uint8Array})
const u32=n=>{const b=Buffer.alloc(4);b.writeUInt32LE(n);return b}
const txt=s=>Buffer.concat([u32(s.length),Buffer.from(s,'utf16le')])
function encode(v){
 if(v===undefined)return Buffer.from([0]);if(v===null)return Buffer.from([1]);if(typeof v==='boolean')return Buffer.from([v?3:2])
 if(typeof v==='number'){const b=Buffer.alloc(9);b[0]=4;b.writeDoubleLE(v,1);return b}
 if(typeof v==='string')return Buffer.concat([Buffer.from([5]),txt(v)])
 if(v instanceof Uint8Array)return Buffer.concat([Buffer.from([8]),u32(v.length),Buffer.from(v)])
 if(Array.isArray(v))return Buffer.concat([Buffer.from([6]),u32(v.length),...v.map(encode)])
 return Buffer.concat([Buffer.from([7]),u32(Object.keys(v).length),...Object.entries(v).flatMap(([k,v])=>[txt(k),encode(v)])])
}
function run(b){
 let at=0;const n=()=>{const v=b.readUInt32LE(at);at+=4;return v};const text=()=>{const len=n();const s=b.subarray(at,at+len*2).toString('utf16le');at+=len*2;return s}
 function read(){const tag=b[at++];switch(tag){case 0:return undefined;case 1:return null;case 2:return false;case 3:return true;case 4:{const x=b.readDoubleLE(at);at+=8;return x}case 5:return text();case 6:return Array.from({length:n()},read);case 7:{const o={};let count=n();while(count--){const k=text();Object.defineProperty(o,k,{value:read(),enumerable:true,writable:true,configurable:true})}return o}case 8:{const len=n(),v=new Uint8Array(b.subarray(at,at+len));at+=len;return v}default:throw Error('tag')}}
 const schema=read(),value=read(),path=text();if(at!==b.length)throw Error('trailing input')
 let errors=[],fault=[];try{errors=exports_.validate(schema,value,path)}catch(e){fault=[e.name,e.message]}
 const weight=exports_.schemaWeight(schema)
 return Buffer.concat([Buffer.from('VLD1'),u32(fault.length),...fault.map(txt),u32(errors.length),...errors.map(txt),encode(value),u32(weight.bytes),u32(weight.depth),u32(exports_.SCHEMA_BUDGET.bytes),u32(exports_.SCHEMA_BUDGET.depth)])
}
const data=fs.readFileSync(0);let at=0;while(at<data.length){const len=data.readUInt32LE(at);at+=4;const out=run(data.subarray(at,at+len));at+=len;process.stdout.write(Buffer.concat([u32(out.length),out]))}
