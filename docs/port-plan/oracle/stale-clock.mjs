// SPDX-License-Identifier: AGPL-3.0-only
// Complete original modules with an injected platform clock, no behavior stubs.
import {readFileSync} from 'node:fs'
import vm from 'node:vm'
const dir=new URL('../../../rust/target/stale-detect-reference/',import.meta.url)
function load(name,require){const exports={};vm.runInThisContext('(function(exports,require){'+readFileSync(new URL(name+'.cjs',dir),'utf8')+'\n})',{filename:name+'.cjs'})(exports,require);return exports}
const types=load('types',()=>{throw Error('Unexpected runtime dependency')}),NativeDate=Date
for(const line of readFileSync(0,'utf8').trimEnd().split('\n')){
 const [id,actions,failAt,clock]=line.split('\t'),stamps=clock.split(','),api=load('stalePage',name=>{if(name!=='./types')throw Error(name);return types})
 let calls=0;const frames=[]
 globalThis.Date=class extends NativeDate {constructor(...args){if(args.length)super(...args);else {const stamp=stamps[calls++%stamps.length];if(calls===Number(failAt))throw Error('clock-fault');super(stamp)}}}
 try{for(const action of actions.split(',')){
  let error='-';try{if(action==='mark')api.markPageStale();else if(action==='reset')api.resetPageStale();else if(action!=='get')throw Error('Unknown action')}catch(e){error=e.message}
  frames.push([api.pageStale()??'null',calls,error].join(','))
 }}finally{globalThis.Date=NativeDate}
 process.stdout.write(id+'\t'+frames.join(';')+'\n')
}
