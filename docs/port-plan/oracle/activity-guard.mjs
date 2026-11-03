// SPDX-License-Identifier: AGPL-3.0-only
import {readFileSync} from 'node:fs'
const bundle=new URL('../../../rust/target/guard-reference/dist-session/activity.mjs',import.meta.url),hex=s=>Array.from({length:s.length},(_,i)=>s.charCodeAt(i).toString(16).padStart(4,'0')).join('');let count=0
for(const line of readFileSync(0,'utf8').trimEnd().split('\n')){
 const [id,modeText,action,h='']=line.split('\t'),modes=modeText.split(',').map(Number),text=String.fromCharCode(...(h.match(/.{4}/g)||[]).map(x=>parseInt(x,16))),api=await import(bundle.href+'?case='+count++),free=api.commandGuard(),log=[],resets=[],mutated=new Set(),fault=Error('fault:'+text)
 const functions=Array.from({length:4},(_,mode)=>()=>{
  log.push('busy:'+mode)
  if(!mutated.has(mode)){mutated.add(mode);if(action==='1')api.configureCommands(guards[(mode+1)%3]);else if(action==='2')resets[0]?.()}
  if(mode===0)return null;if(mode===1)return '';if(mode===2)return text;throw fault
 })
 const guards=modes.map(mode=>({busy:functions[mode]})),frames=[]
 for(const op of ['get','c0','busy','c1','busy','r0','get','c2','busy','drop1','get','c0','r2','busy','r0','get']){
  let result='-'
  if(op.startsWith('c'))resets.push(api.configureCommands(guards[Number(op.slice(1))]))
  else if(op.startsWith('r'))resets[Number(op.slice(1))]?.()
  else if(op==='drop1')resets[1]=null
  else if(op==='busy'){try{const value=api.commandGuard().busy();result=value===null?'null':'text:'+hex(value)}catch(e){result='throw:'+e.name+':'+hex(e.message)+':'+Number(e===fault)}}
  const current=api.commandGuard(),active=current===free?0:guards.indexOf(current)+1;if(!active&&current!==free)throw Error('Unknown guard identity')
  frames.push([active,result,log.join(',')].join('|'))
 }
 process.stdout.write(id+'\t'+frames.join(';')+'\n')
}
