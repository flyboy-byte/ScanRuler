// SPDX-License-Identifier: AGPL-3.0-only
import {readFileSync} from 'node:fs'
const bundle=new URL('../../../rust/target/plugin-busy-reference/dist-session/activity.mjs',import.meta.url)
const hex=s=>Array.from({length:s.length},(_,i)=>s.charCodeAt(i).toString(16).padStart(4,'0')).join('')
let count=0
for(const line of readFileSync(0,'utf8').trimEnd().split('\n')){
 const [id,aa,bb,mm]=line.split('\t'),a=+aa,b=+bb,mode=+mm,api=await import(bundle.href+'?case='+count++)
 const names=['x','','__proto__','q"u','💩','unpaired:\ud800'],key=names[mode],other=key+'B',third=key+'C',stores=Array.from({length:3},()=>({subscribe(){throw Error('unexpected subscribe')}})),arrays=[[],[stores[0],stores[1]],[stores[1],stores[0],stores[1]]],stops=[()=>{},()=>{},()=>{}],releases=[],calls=[0,0,0],frames=[];let fired=false
 const value=n=>{if(n===3)throw Error('failure');return n===0?null:n===1?'':'Busy \ud800💩'}
 const checks=[()=>{calls[0]++;return value(a)},()=>{calls[1]++;return value(b)},()=>{calls[2]++;if(!fired){fired=true;if(mode===2){releases[1]();install(other,0,2,2)}if(mode===3)install(third,1,1,0);if(mode===4){releases[0]();install(key,0,2,2)}}return null}]
 const install=(name,c,s,t)=>{releases.push(api.registerBusy(name,checks[c],arrays[s],t<0?undefined:stops[t]));return '-'}
 const frame=result=>{const check=[...api.busyChecks].map(([k,v])=>hex(k)+':'+checks.indexOf(v)).join(',');const store=[...api.busyStores].map(([k,v])=>hex(k)+':'+arrays.indexOf(v)).join(',');const stop=[...api.busyStops].map(([k,v])=>hex(k)+':'+stops.indexOf(v)).join(',');frames.push([result,check,store,stop,calls.join(',')].join('|'))}
 const op=fn=>{try{frame(fn())}catch(e){frame('!'+hex(e.message))}}
 frame('-')
 if(mode===0){op(()=>install(key,0,1,0));op(()=>install(key,1,2,1));op(()=>hex(api.pluginBusy()??'<null>'));op(()=>{releases[0]();return '-'});op(()=>install(key,1,2,1));op(()=>{releases[0]();return '-'});op(()=>hex(api.pluginBusy()??'<null>'))}
 if(mode===1){op(()=>install(key,0,1,0));op(()=>{releases[0]();return '-'});op(()=>install(key,0,2,-1));op(()=>{releases[0]();return '-'});op(()=>{releases[0]();return '-'});op(()=>hex(api.pluginBusy()??'<null>'))}
 if(mode>=2&&mode<=4){op(()=>install(key,2,1,1));op(()=>install(other,1,2,-1));op(()=>hex(api.pluginBusy()??'<null>'));op(()=>hex(api.pluginBusy()??'<null>'));op(()=>{releases[0]();return '-'});op(()=>hex(api.pluginBusy()??'<null>'))}
 if(mode===5){op(()=>install(key,0,0,-1));op(()=>{releases[0]=null;return '-'});op(()=>install(other,1,1,2));op(()=>hex(api.pluginBusy()??'<null>'));op(()=>install(key,1,2,1));op(()=>{releases[1]();return '-'});op(()=>hex(api.pluginBusy()??'<null>'))}
 process.stdout.write(id+'\t'+frames.join(';')+'\n')
}
