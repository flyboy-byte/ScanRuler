// SPDX-License-Identifier: AGPL-3.0-only
// Execute the complete frozen helper module; erase only TypeScript annotations.
const { execFileSync } = require('node:child_process')
const { readFileSync } = require('node:fs')
const vm = require('node:vm')
const ts = require(process.cwd() + '/node_modules/typescript')
const source = execFileSync('git',['show','2cd89c6769a10c27ee7754b477ada521c9ef5cb7:src/core/parsers/objTexture.ts'],{encoding:'utf8'})
const exported={}
vm.runInNewContext(ts.transpileModule(source,{compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.CommonJS}}).outputText,{exports:exported})
const b=readFileSync(0);let at=0
const u8=()=>b[at++], word=()=>{const v=b.readUInt32LE(at);at+=4;return v}
const str=()=>{const n=word(),v=b.subarray(at,at+n).toString('utf8');at+=n;return v}
const list=read=>Array.from({length:word()},read)
const doubles=()=>list(()=>{const x=b.readDoubleLE(at);at+=8;return x})
const ints=()=>Int32Array.from(list(()=>{const x=b.readInt32LE(at);at+=4;return x}))
const image=()=>{const name=str(),width=word(),height=word(),n=word(),rgba=Uint8Array.from(b.subarray(at,at+n));at+=n;return{name,width,height,rgba}}
const out=[Buffer.from('OTX1')];const w=n=>{const b=Buffer.alloc(4);b.writeUInt32LE(n);out.push(b)}
const text=s=>{const b=Buffer.from(s,'utf8');w(b.length);out.push(b)}
const strings=v=>{w(v.length);for(const s of v)text(s)}
const mode=u8();out.push(Buffer.from([mode]))
if(mode===1)text(exported.baseName(str()))
else if(mode===2){const m=exported.parseMtl(str());w(m.size);for(const[k,v]of m){text(k);text(v)}}
else if(mode===3)strings(exported.mtllibNames(str()))
else if(mode===4){const img=image(),u=b.readDoubleLE(at),v=b.readDoubleLE(at+8);at+=16;const offset=word(),n=word(),pixels=Uint8Array.from(b.subarray(at,at+n));at+=n;exported.sampleTexture(img,u,v,pixels,offset);w(pixels.length);out.push(Buffer.from(pixels))}
else if(mode===5){const refs={uvs:doubles(),vertexUv:ints(),vertexMaterial:ints(),materialNames:list(str),libraries:list(str)};let given
 if(u8())given={materials:list(()=>({name:str(),text:str()})),images:list(image)}
 const result=exported.bakeObjTexture(refs,given)
 if(result===null)out.push(Buffer.from([0]))
 else if(result.note.kind==='missing'){if(result.colors!==null)throw Error('unexpected missing colors');out.push(Buffer.from([1]));strings(result.note.files)}
 else{out.push(Buffer.from([2]));w(result.colors.length);out.push(Buffer.from(result.colors));strings(result.note.images)}
}else throw Error('unknown test operation')
if(at!==b.length)throw Error('unconsumed test input')
process.stdout.write(Buffer.concat(out))
