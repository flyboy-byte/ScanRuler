// SPDX-License-Identifier: AGPL-3.0-only
import {readFileSync} from 'node:fs'
const bundle=new URL('../../../rust/target/watch-session-reference/dist-session/activity.mjs',import.meta.url),hex=s=>Array.from({length:s.length},(_,i)=>s.charCodeAt(i).toString(16).padStart(4,'0')).join('');let count=0
for(const line of readFileSync(0,'utf8').trimEnd().split('\n')){
 const p=line.split('\t'),id=p[0],roles=p.slice(1,5).map(s=>s?s.split(',').map(Number):[]),[subFail,stopFail,stopMode,immediate,listenerFail,mutate]=p.slice(5).map(Number),api=await import(bundle.href+'?case='+count++),log=[],active=Array.from({length:4},()=>new Map()),subs=[0,0,0,0],unsubs=new Map(),faults=[...Array.from({length:4},(_,i)=>Error('sub-'+i)),...Array.from({length:4},(_,i)=>Error('unsub-'+i)),Error('listener')]
 let serial=0,listenerCalls=0,root=roles[0].slice(),changed=false,stop=null
 const listener=()=>{log.push('listener:'+ ++listenerCalls);if(listenerFail&&listenerCalls===1)throw faults[8]}
 const stores=Array.from({length:4},(_,i)=>({subscribe(got){
  log.push('sub:'+i+':'+Number(got===listener));subs[i]++
  if(i===subFail&&subs[i]===1)throw faults[i]
  const key=serial++;active[i].set(key,got)
  if(i===0&&mutate&&!changed){changed=true;root=[3];api.watchSessionStores([stores[3]])}
  if(immediate&(1<<i))got()
  return ()=>{log.push('unsub:'+i+':'+key);const calls=(unsubs.get(key)||0)+1;unsubs.set(key,calls);if(i===stopFail&&(stopMode===2||calls===1))throw faults[4+i];active[i].delete(key)}
 }}))
 api.watchSessionStores(roles[0].map(i=>stores[i]))
 api.registerBusy('p0',()=>{throw Error('Unexpected busy check')},roles[2].map(i=>stores[i]))
 api.registerBusy('p1',()=>{throw Error('Unexpected busy check')},roles[3].map(i=>stores[i]))
 const frame=result=>[result,Number(stop!==null),root.join(','),active.map(a=>[...a.keys()].join('.')).join('/'),log.join(',')].join('|'),frames=[frame('-')]
 for(const op of ['watch','emit','stop','emit','stop','drop','emit']){
  let result='ok'
  try{if(op==='watch')stop=roles[1].length?api.watchSession(listener,roles[1].map(i=>stores[i])):api.watchSession(listener)
   else if(op==='emit'){for(const values of active)for(const callback of [...values.values()])callback()}
   else if(op==='stop')stop?.()
   else stop=null
  }catch(e){const index=faults.indexOf(e);if(index<0)throw e;result='throw:'+e.name+':'+hex(e.message)+':'+index}
  frames.push(frame(result))
 }
 process.stdout.write(id+'\t'+frames.join(';')+'\n')
}
