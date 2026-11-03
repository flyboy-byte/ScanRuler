// SPDX-License-Identifier: AGPL-3.0-only
import {readFileSync} from 'node:fs'
const bundle=new URL('../../../rust/target/normalize-reference/dist-session/registry.mjs',import.meta.url),NativeDate=Date
const codes=['unknown_command','invalid_input','busy','no_scan','not_found','invalid_state','failed','unavailable','not_implemented','timeout','internal','stale_page']
const hex=s=>Array.from({length:s.length},(_,i)=>s.charCodeAt(i).toString(16).padStart(4,'0')).join('');let count=0
for(const line of readFileSync(0,'utf8').trimEnd().split('\n')){
 const [id,marked,fail,kind,h,stamp]=line.split('\t'),s=String.fromCharCode(...(h.match(/.{4}/g)||[]).map(x=>parseInt(x,16))),log=[],api=await import(bundle.href+'?case='+count++)
 if(marked==='1'){globalThis.Date=class extends NativeDate{constructor(){super('1999-12-31T23:59:59.999Z')}};try{api.markPageStale()}finally{globalThis.Date=NativeDate}}
 let e
 if(kind==='text')e=s
 else if(kind==='error')e=Error(s)
 else if(kind.startsWith('command-'))e=new api.CommandError(codes[Number(kind.slice(8))],s)
 else if(kind==='object'||kind==='coerce-fault')e={toString(){log.push('coerce');if(kind==='coerce-fault')throw Error('coercion-fault');return s}}
 else if(kind==='array')e=[s,null,undefined]
 else throw Error('Unknown kind')
 globalThis.Date=class extends NativeDate{constructor(){log.push('clock');if(fail==='1')throw Error('clock-fault');super(stamp)}}
 let result
 try{const out=api.asCommandError(e);result=[out.name,out.code,hex(out.message),Number(out===e)].join(',')}
 catch(e){result='throw,'+hex(e.message)}finally{globalThis.Date=NativeDate}
 process.stdout.write([id,api.pageStale()??'null',log.join(','),result].join('\t')+'\n')
}
