// SPDX-License-Identifier: AGPL-3.0-only
import {readFileSync} from 'node:fs'
const bundle=new URL('../../../rust/target/watch-stores-reference/dist-session/activity.mjs',import.meta.url);let count=0
const programs=[['get','a','r0','b','r0','r1','get'],['a','b','r0','get','r1'],['a','d0','get','b','d1','get'],['a','r0','r0','get','a','r0','get'],['a','b','a','r0','get','r1','get','r2']]
for(const line of readFileSync(0,'utf8').trimEnd().split('\n')){
 const [id,a,b,mode]=line.split('\t'),api=await import(bundle.href+'?case='+count++),resets=[],frames=[]
 let calls=0;const stores=Array.from({length:3},()=>({subscribe(){calls++;throw Error('Unexpected subscription')}})),arrays=[[],[stores[0],stores[1]],[stores[1],stores[0],stores[1]],[stores[0],stores[1]]],seen=[api.sessionStores,...arrays]
 const frame=()=>{let index=seen.indexOf(api.sessionStores);if(index<0){index=seen.length;seen.push(api.sessionStores)}const members=api.sessionStores.map(s=>{const i=stores.indexOf(s);if(i<0)throw Error('Unknown store');return i});frames.push([index,members.join(','),calls].join('|'))}
 frame()
 for(const op of programs[Number(mode)]){
  if(op==='a'||op==='b')resets.push(api.watchSessionStores(arrays[Number(op==='a'?a:b)]))
  else if(op.startsWith('r'))resets[Number(op.slice(1))]?.()
  else if(op.startsWith('d'))resets[Number(op.slice(1))]=null
  frame()
 }
 process.stdout.write(id+'\t'+frames.join(';')+'\n')
}
