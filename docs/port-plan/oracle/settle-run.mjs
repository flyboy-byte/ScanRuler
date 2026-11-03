// SPDX-License-Identifier: AGPL-3.0-only
import {readFileSync} from 'node:fs'
const bundle=new URL('../../../rust/target/settle-reference/dist-session/registry.mjs',import.meta.url)
const turn=()=>new Promise(setImmediate),hex=s=>Array.from({length:s.length},(_,i)=>s.charCodeAt(i).toString(16).padStart(4,'0')).join('');let count=0
for(const line of readFileSync(0,'utf8').trimEnd().split('\n')){
 const [id,kinds,ro,order,h]=line.split('\t'),modes=kinds?kinds.split(',').map(Number):[],name=String.fromCharCode(...(h.match(/.{4}/g)||[]).map(x=>parseInt(x,16))),api=await import(bundle.href+'?case='+count++),log=[],pending=new Map(),frames=[]
 let unhandled=0;const onUnhandled=()=>unhandled++;process.on('unhandledRejection',onUnhandled)
 try{
 for(let i=0;i<modes.length;i++)api.registerSettle(next=>{
  log.push(i+':'+hex(next.name)+':'+Number(next.readOnly))
  if(modes[i]===0)return
  if(modes[i]===1)throw Error('sync-'+i)
  if(modes[i]===2)return Promise.resolve()
  if(modes[i]===3)return Promise.reject(Error('async-'+i))
  return new Promise((resolve,reject)=>pending.set(i,()=>modes[i]===4?resolve():reject(Error('late-'+i))))
 })
 let status;const wait=api.settle(name,{readOnly:ro==='1'})
 if(wait===null)status='null';else{status='pending';wait.then(()=>{status='done'})}
 const frame=()=>frames.push([status,log.join(','),unhandled].join('|'));frame();await turn();frame()
 let indices=[...pending.keys()];if(order==='reverse')indices.reverse()
 if(order==='all'){for(const i of indices)pending.get(i)();frame();await turn();frame()}
 else for(const i of indices){pending.get(i)();frame();await turn();frame()}
 await turn();frame();if(status==='pending')throw Error('Unfinished settle')
 process.stdout.write(id+'\t'+frames.join(';')+'\n')
 }finally{process.removeListener('unhandledRejection',onUnhandled)}
}
