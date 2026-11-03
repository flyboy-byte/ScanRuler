// SPDX-License-Identifier: AGPL-3.0-only
import {readFileSync} from 'node:fs'
import {pathToFileURL} from 'node:url'
const path=process.argv[2]??0,module=process.argv[3]??new URL('../../../rust/target/activity-state-reference/dist-session/activity.mjs',import.meta.url).pathname
for(const line of readFileSync(path,'utf8').trim().split('\n')) {
 const [id,m,s]=line.split('\t'),mode=+m,seed=+s
 const {useCommandActivity:store,commandRunning}=await import(pathToFileURL(module)+'?case='+id)
 const ids=new Map(),logs=[],held=[],stops=[],fault={opaque:true};let fired=false
 const identity=x=>{if(x===null)return 0;if(!ids.has(x))ids.set(x,ids.size+1);return ids.get(x)}
 const hex=x=>{if(x===null)return '-';let out='';for(let i=0;i<x.length;i++)out+=x.charCodeAt(i).toString(16).padStart(4,'0');return out}
 const bits=x=>{const b=Buffer.alloc(8);b.writeDoubleLE(x);return b.toString('hex')}
 const show=x=>`${identity(x)}:${x.running===null?'-':`${identity(x.running)},${hex(x.running.name)},${hex(x.running.title)},${bits(x.running.since)}`}:${hex(x.progress)}`
 const take=()=>{const state=store.getState();held.push(state);logs.push('S:'+show(state)+':'+Number(commandRunning()))}
 const progress=mode===12?'':'P'+seed+'\ud800',clock=[0,-0,1.5,-37,NaN,Infinity,-Infinity,8640000000000001][seed%8]
 const callbacks=['A','B','C'].map((name,index)=>(state,previous)=>{
   logs.push(name+':'+show(state)+'<'+show(previous)+'@'+identity(store.getState())+':'+Number(commandRunning()))
   if((index===1&&mode===13)||(index===2&&mode===14))throw fault
   if(index!==0||fired)return
   fired=true
   if(mode===2){stops[1]();stops[2]=store.subscribe(callbacks[2])}
   if(mode===3){stops[0]();stops.push(store.subscribe(callbacks[0]))}
   if(mode===4)throw fault
   if(mode===5||mode===13)store.setState({progress})
   if(mode===14)stops[2]=store.subscribe(callbacks[2])
   if(mode===15){stops[1]();stops.push(store.subscribe(callbacks[1]))}
   if(mode===8)store.setState({running:null,progress:null})
 })
 take()
 if(mode!==0){stops[0]=store.subscribe(callbacks[0]);stops[1]=store.subscribe(callbacks[1])}
 if(mode===1)stops.push(store.subscribe(callbacks[0]))
 if(mode===6){stops[0]();stops.push(store.subscribe(callbacks[0]));stops[0]()}
 if(mode===7){stops.pop() /* dropping a release does not unsubscribe */}
 const run=patch=>{try{store.setState(patch);logs.push('ok')}catch(e){if(e!==fault)throw e;logs.push('fault:same')}take()}
 run({running:{name:seed%2?'':'run\ud800',title:'ΟΣ'+seed,since:clock},progress:null})
 if(mode===4)stops[0]()
 run({progress})
 run({progress})
 run({running:null,progress:null})
 run({running:null,progress:null})
 logs.push('held:'+held.map(show).join('/'))
 console.log(id+'\t'+logs.join('|'))
}
