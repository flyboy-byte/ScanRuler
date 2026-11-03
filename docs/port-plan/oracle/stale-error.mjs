// SPDX-License-Identifier: AGPL-3.0-only
// Full pinned staleError body. Date and coercion events are observed synchronously.
import {readFileSync} from 'node:fs'
import vm from 'node:vm'
const dir=new URL('../../../rust/target/stale-detect-reference/',import.meta.url),NativeDate=Date
function load(name,require){const exports={};vm.runInThisContext('(function(exports,require){'+readFileSync(new URL(name+'.cjs',dir),'utf8')+'\n})',{filename:name+'.cjs'})(exports,require);return exports}
const types=load('types',()=>{throw Error('Unexpected dependency')}),hex=s=>Array.from({length:s.length},(_,i)=>s.charCodeAt(i).toString(16).padStart(4,'0')).join('')
for(const line of readFileSync(0,'utf8').trimEnd().split('\n')){
 const [id,marked,fail,kind,h,stamp]=line.split('\t'),s=String.fromCharCode(...(h.match(/.{4}/g)||[]).map(x=>parseInt(x,16))),log=[],api=load('stalePage',name=>{if(name!=='./types')throw Error(name);return types})
 if(marked==='1'){globalThis.Date=class extends NativeDate{constructor(){super('1999-12-31T23:59:59.999Z')}};try{api.markPageStale()}finally{globalThis.Date=NativeDate}}
 let e
 if(kind==='text')e=s
 else if(kind==='error')e=Error(s)
 else if(kind==='command')e=new types.CommandError('busy',s)
 else if(kind==='object'||kind==='coerce-fault')e={toString(){log.push('coerce');if(kind==='coerce-fault')throw Error('coercion-fault');return s}}
 else if(kind==='getter'||kind==='getter-fault'){e=Error();Object.defineProperty(e,'message',{get(){log.push('message');if(kind==='getter-fault')throw Error('message-fault');return s}})}
 else if(kind==='null')e=null
 else if(kind==='undefined')e=undefined
 else if(kind==='array')e=[s,null,undefined]
 else throw Error('Unknown kind')
 globalThis.Date=class extends NativeDate{constructor(){log.push('clock');if(fail==='1')throw Error('clock-fault');super(stamp)}}
 let result
 try{const out=api.staleError(e);result=[out.name,out.code,hex(out.message)].join(',')}
 catch(e){result='throw,'+hex(e.message)}finally{globalThis.Date=NativeDate}
 process.stdout.write([id,api.pageStale()??'null',log.join(','),result].join('\t')+'\n')
}
