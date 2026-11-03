// SPDX-License-Identifier: AGPL-3.0-only
// Actual Date conversion and complete original page state; clock input only is injected.
import {readFileSync} from 'node:fs'
import vm from 'node:vm'
const dir=new URL('../../../rust/target/stale-detect-reference/',import.meta.url),NativeDate=Date
function load(name,require){const exports={};vm.runInThisContext('(function(exports,require){'+readFileSync(new URL(name+'.cjs',dir),'utf8')+'\n})',{filename:name+'.cjs'})(exports,require);return exports}
const types=load('types',()=>{throw Error('Unexpected dependency')}),hex=s=>Array.from({length:s.length},(_,i)=>s.charCodeAt(i).toString(16).padStart(4,'0')).join('')
for(const line of readFileSync(0,'utf8').trimEnd().split('\n')){
 const [id,bits]=line.split('\t'),ms=Buffer.from(bits,'hex').readDoubleBE(0),api=load('stalePage',name=>{if(name!=='./types')throw Error(name);return types}),frames=[]
 let direct;try{direct='ok:'+hex(new NativeDate(ms).toISOString())}catch(e){direct='throw:'+e.name+':'+hex(e.message)}
 let calls=0
 for(const action of ['mark','get','epoch','invalid','reset','mark']){
  let error='-'
  globalThis.Date=class extends NativeDate{constructor(){calls++;super(action==='epoch'?0:action==='invalid'?NaN:ms)}}
  try{if(action==='reset')api.resetPageStale();else if(action!=='get')api.markPageStale()}catch(e){error=e.name+':'+hex(e.message)}finally{globalThis.Date=NativeDate}
  frames.push([api.pageStale()??'null',calls,error].join(','))
 }
 process.stdout.write([id,direct,frames.join(';')].join('\t')+'\n')
}
