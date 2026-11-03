// SPDX-License-Identifier: AGPL-3.0-only
import {readFileSync} from 'node:fs'
import {pathToFileURL} from 'node:url'
const input=readFileSync(process.argv[2]??0,'utf8'),module=process.argv[3]??new URL('../../../rust/target/state-sections-reference/dist-session/activity.mjs',import.meta.url).pathname
for(const line of input.trim().split('\n')){
 const [id,m,s]=line.split('\t'),mode=+m,seed=+s
 const {registerStateSection:register,readStateSections:read,readStateBriefs:brief}=await import(pathToFileURL(module)+'?case='+id)
 const logs=[],stops=[],keys=['a','2','__proto__','constructor','','01','4294967294','\ud800'],key=mode>=16?'__proto__':keys[seed],fault={opaque:true};let fired=false
 const hex=x=>{let v='';for(let i=0;i<x.length;i++)v+=x.charCodeAt(i).toString(16).padStart(4,'0');return v}
 const values=[null,true,37,'T\ud800',undefined,{v:seed},[seed],{__proto__:null,inherited:'I'}]
 const data=values[mode>=16?mode-16:seed],refs=new Map([[values[5],'obj'],[values[6],'arr'],[values[7],'proto']])
 const show=v=>{if(refs.has(v))return '@'+refs.get(v);if(v===undefined)return'u';if(v===null)return'n';if(typeof v==='string')return's'+hex(v);if(typeof v==='number')return'd'+v;if(typeof v==='boolean')return'b'+Number(v);return'o{'+Object.keys(v).map(k=>hex(k)+':'+show(v[k])).join(',')+'}'}
 const output=x=>{const p=Object.getPrototypeOf(x);return Object.keys(x).map(k=>hex(k)+'='+show(x[k])).join(',')+'^'+(p===Object.prototype?'default':p===null?'null':show(p))}
 const convert=kind=>{const e=kind===0?new Error('error'):{};if(kind===0)Object.defineProperty(e,'message',{get(){logs.push('message');if(mode===10)throw fault;return data}});else e[Symbol.toPrimitive]=()=>{logs.push('string');if(mode===11)throw fault;return 'converted\ud800'};return e}
 const readers=['A','B','C'].map((name,index)=>()=>{
  logs.push(name)
  if(index===0&&!fired){fired=true
   if(mode===2){stops[1]();stops.push(register('c',readers[2],readers[2]))}
   if(mode===3){stops[0]();stops.push(register(key,readers[0],readers[2]))}
   if(mode===4)stops.push(register('c',readers[2]))
  }
  if(index===0&&(mode===8||mode===10))throw convert(0)
  if(index===0&&(mode===9||mode===11))throw convert(1)
  return index===0?data:index===1?{value:'B'}:'C'
 })
 // Stable shared returned values are explicitly identified, not reconstructed.
 const bvalue={value:'B'};readers[1]=()=>{logs.push('B');return bvalue};refs.set(bvalue,'B')
 const add=(name,reader,b)=>{try{stops.push(register(name,reader,b));logs.push('registered')}catch(e){logs.push('duplicate:'+hex(e.message))}}
 add(key,readers[0],mode===5?undefined:readers[2]);add('b',readers[1],readers[1])
 if(mode===1)add(key,readers[2],readers[2])
 if(mode===6){stops[0]();add(key,readers[0]);stops[0]()}
 if(mode===7){stops[0]();add(key,readers[2],readers[2]);stops[0]()}
 if(mode===12)stops.pop()
 if(mode===13)stops[1]()
 if(mode===14){stops[0]();stops[0]()}
 if(mode===15)add('0',readers[2],readers[2])
 for(const [name,fn] of [['read',read],['brief',brief],['read',read]]){
  try{logs.push(name+':'+output(fn()))}catch(e){if(e!==fault)throw e;logs.push(name+':fault:same')}
 }
 console.log(id+'\t'+logs.join('|'))
}
