// SPDX-License-Identifier: AGPL-3.0-only
import {readFileSync} from 'node:fs'
const bundle=new URL('../../../rust/target/plugin-stop-reference/dist-session/activity.mjs',import.meta.url),turn=()=>new Promise(setImmediate)
let count=0
for(const line of readFileSync(0,'utf8').trimEnd().split('\n')){
 const [id,...nums]=line.split('\t'),[a,b,action,phase,s0,s1]=nums.map(Number),api=await import(bundle.href+'?case='+count++),frames=[],log=[],jobs=[],status=[],releases=[],stores=[[],[],[]];let mutated=false,unhandled=0
 const done=new Promise(()=>{}),onUnhandled=()=>unhandled++;process.on('unhandledRejection',onUnhandled)
 try{
 const install=i=>{releases[i]=api.registerBusy(String(i),checks[i],stores[i],action===7?undefined:stops[i])}
 const mutate=()=>{if(mutated)return;mutated=true;switch(action){case 1:releases[1]();break;case 2:install(2);break;case 3:releases[1]();install(1);break;case 4:releases[0]();install(0);break;case 5:releases[0]();break;case 6:releases[0]();releases[0]=api.registerBusy("0",checks[2],stores[2],stops[2]);break}}
 const checks=[0,1,2].map(i=>()=>{log.push('check:'+i);if(i===0&&phase===0)mutate();const n=i===0?a:i===1?b:2;if(n===3)throw Error('check');return n===0?null:n===1?'':'busy'})
 const stops=[0,1,2].map(i=>d=>{log.push('stop:'+i+':'+Number(d===done));if(i===0&&phase===1)mutate();const n=i===0?s0:i===1?s1:3;if(n===0)throw Error('sync');if(n===1)return Promise.resolve();if(n===2)return Promise.reject(Error('ready'));return new Promise((resolve,reject)=>jobs.push({done:false,finish:()=>n===3?resolve():reject(Error('late'))}))})
 if(action!==8){install(0);install(1)}
 const frame=()=>frames.push([status.join(','),log.join(','),[...api.busyChecks].map(([k])=>k).join(','),[...api.busyStores].map(([k,v])=>k+':'+stores.indexOf(v)).join(','),[...api.busyStops].map(([k,v])=>k+':'+stops.indexOf(v)).join(','),unhandled].join('|'))
 for(let round=0;round<2;round++){
  const wait=api.stopPluginWork(done);status.push('pending');wait.then(()=>status[round]='done');frame();await turn();frame()
  for(let j=jobs.length-1;j>=0;j--)if(!jobs[j].done){jobs[j].done=true;jobs[j].finish();log.push('finish:'+j);frame();await turn();frame()}
  await turn();frame();if(status[round]!=='done')throw Error('unfinished')
 }
 process.stdout.write(id+'\t'+frames.join(';')+'\n')
 }finally{process.removeListener('unhandledRejection',onUnhandled)}
}
