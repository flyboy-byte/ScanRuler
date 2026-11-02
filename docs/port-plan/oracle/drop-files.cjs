// SPDX-License-Identifier: AGPL-3.0-only
// Execute complete pinned modules; only erase TS and bind original dependencies.
const { execFileSync } = require('node:child_process')
const { readFileSync } = require('node:fs')
const vm = require('node:vm')
const ts = require(process.cwd() + '/node_modules/typescript')
const reference = '2cd89c6769a10c27ee7754b477ada521c9ef5cb7'
function load(path, imports) {
  const source = execFileSync('git', ['show', reference + ':' + path], { encoding: 'utf8' })
  const exports = {}
  vm.runInNewContext(ts.transpileModule(source, { compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.CommonJS } }).outputText,
    { exports, require: id => { if (!(id in imports)) throw Error('Unexpected source import ' + id); return imports[id] }, TextDecoder, Uint8Array })
  return exports
}
const f = load('src/core/formats.ts', {})
const s = load('src/app/scanFiles.ts', { '../core/formats': f, fflate: require(process.cwd() + '/node_modules/fflate') })
const b=readFileSync(0);let at=0
const byte=()=>b[at++],word=()=>{const n=b.readUInt32LE(at);at+=4;return n}
const str=()=>{const n=word(),s=b.subarray(at,at+n).toString('utf8');at+=n;return s}
const list=fn=>Array.from({length:word()},fn),err=()=>byte()?{name:str(),message:str()}:null
const present=byte(),files=Array.from({length:word()},(_,id)=>({id})),fallback=list(word)
const items=list(()=>({kind:byte(),api:byte(),entry:word(),error:err()}))
const specs=list(()=>({flags:byte(),file:word(),fe:err(),re:err(),batches:list(()=>({error:err(),ids:list(word)}))}))
if(at!==b.length)throw Error('trailing fixture')
const log=[],error=e=>Object.assign(new Error(e.message),{name:e.name})
const entries=specs.map((e,id)=>({isFile:!!(e.flags&1),isDirectory:!!(e.flags&2),file(ok,bad){log.push('file:'+id);if(e.fe)bad(error(e.fe));else ok(files[e.file])},createReader(){log.push('reader:'+id);if(e.re)throw error(e.re);let cursor=0;return{readEntries(ok,bad){log.push('batch:'+id+':'+cursor);const batch=e.batches[cursor++]??{ids:[]};if(batch.error)bad(error(batch.error));else ok(batch.ids.map(i=>entries[i]))}}}}))
const data=present?{files:fallback.map(i=>files[i]),items:items.map((i,id)=>({kind:i.kind?'file':'string',webkitGetAsEntry:i.api?()=>{log.push('entry:'+id);if(i.error)throw error(i.error);return i.entry===0xffffffff?null:entries[i.entry]}:undefined}))}:null
const out=[Buffer.from('DRP1')],w=n=>{const b=Buffer.alloc(4);b.writeUInt32LE(n);out.push(b)},text=s=>{const b=Buffer.from(s);w(b.length);out.push(b)}
;(async()=>{try{const result=await s.droppedFiles(data);out.push(Buffer.from([0]));w(result.length);for(const file of result)w(files.indexOf(file))}catch(e){out.push(Buffer.from([1]));text(e.name);text(e.message)}w(log.length);for(const event of log)text(event);process.stdout.write(Buffer.concat(out))})().catch(e=>{console.error(e);process.exitCode=1})
