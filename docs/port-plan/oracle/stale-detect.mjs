// SPDX-License-Identifier: AGPL-3.0-only
import {readFileSync} from 'node:fs'
import vm from 'node:vm'
const dir=new URL('../../../rust/target/stale-detect-reference/',import.meta.url)
function load(name,require){const exports={};vm.runInThisContext('(function(exports,require){'+readFileSync(new URL(name+'.cjs',dir),'utf8')+'\n})',{filename:name+'.cjs'})(exports,require);return exports}
const types=load('types',()=>{throw Error('Unexpected runtime dependency')})
const api=load('stalePage',name=>{if(name!=='./types')throw Error('Unexpected dependency '+name);return types})
for(const line of readFileSync(0,'utf8').trimEnd().split('\n')){
 const [id,kind,hex='']=line.split('\t'),s=String.fromCharCode(...(hex.match(/.{4}/g)||[]).map(x=>parseInt(x,16)))
 const value={text:s,error:new Error(s),'type-error':new TypeError(s),command:new types.CommandError('failed',s),object:{message:s},array:[s],null:null,undefined:undefined,number:17,boolean:true}[kind]
 const result=api.isStaleImport(value)
 if(api.pageStale()!==null)throw Error('Predicate mutated page state')
 process.stdout.write(id+'\t'+Number(result)+'\t'+String(api.pageStale())+'\n')
}
