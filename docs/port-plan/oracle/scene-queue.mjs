// SPDX-License-Identifier: AGPL-3.0-only
import {readFileSync} from 'node:fs'
const bundle=new URL('../../../rust/target/scene-queue-reference/dist-session/registry.mjs',import.meta.url),turn=()=>new Promise(setImmediate)
let count=0
for(const line of readFileSync(0,'utf8').trimEnd().split('\n')){
 const [id,mm,pp,oo]=line.split('\t'),mask=+mm,pattern=+pp,order=+oo,api=await import(bundle.href+'?case='+count++),log=[],frames=[],states=[],identity=[],owned=[],jobs=[],values=Array.from({length:4},()=>({value:true})),faults=Array.from({length:4},()=>({error:true}));let unhandled=0
 const onUnhandled=()=>unhandled++;process.on('unhandledRejection',onUnhandled)
 try{
 const modes=[pattern,(pattern+1)%5,(pattern+3)%5,0]
 const run=i=>{log.push('start:'+i);const mode=modes[i];if(mode===4)throw faults[i];const promise=mode===0?Promise.resolve(values[i]):mode===1?Promise.reject(faults[i]):new Promise((resolve,reject)=>jobs[i]=()=>mode===2?resolve(values[i]):reject(faults[i]));owned[i]=promise;return promise}
 const frame=()=>frames.push([states.join(','),identity.join(','),log.join(','),unhandled].join('|'))
 const submit=i=>{try{const wait=api.inScene({scene:i===3||!!(mask&(1<<i))},()=>run(i));states[i]='pending';identity[i]=Number(wait===owned[i]);wait.then(v=>states[i]='ok:'+Number(v===values[i]),e=>states[i]='err:'+Number(e===faults[i]))}catch(e){states[i]='throw:'+Number(e===faults[i]);identity[i]='-'}}
 for(let i=0;i<3;i++){submit(i);frame()}
 await turn();frame()
 const orders=[[0,1,2],[2,1,0],[1,2,0]],finished=new Set()
 for(let pass=0;pass<3;pass++)for(const i of orders[order])if(jobs[i]&&!finished.has(i)){finished.add(i);jobs[i]();log.push('finish:'+i);frame();await turn();frame()}
 await turn();frame();submit(3);frame();await turn();frame();if(states.some(s=>s==='pending'))throw Error('unfinished')
 process.stdout.write(id+'\t'+frames.join(';')+'\n')
 }finally{process.removeListener('unhandledRejection',onUnhandled)}
}
