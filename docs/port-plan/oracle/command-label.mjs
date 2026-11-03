// SPDX-License-Identifier: AGPL-3.0-only
import {readFileSync} from 'node:fs'
if(process.versions.unicode!=='17.0')throw Error('Unicode profile changed; review before updating')
const api=await import(new URL('../../../rust/target/command-label-reference/dist-session/registry.mjs',import.meta.url))
const patterns=['PLANE FIT','ΟΣ','Α\u0345Σ','AΣ\u0301','İKẞ','A\ud800Σ','AΣ\udfffA','\ud800\udc00\ud800x\udfff','\u0000\n','', 'ΜΆΙΟΣ','𝔄𐐀💩']
function model(kind,a,b){if(kind===2)return patterns[a];let s='';for(let c=a;c<b;c++){const ch=String.fromCodePoint(c);s+=kind===0?ch+'\0':'AΣ'+ch+'\0'+ch+'Σ\0AΣ'+ch+'Σ\0'+ch+'ΣA\0'}return s}
function bytes(s){const out=Buffer.alloc(s.length*2);for(let i=0;i<s.length;i++)out.writeUInt16LE(s.charCodeAt(i),i*2);return out.toString('hex')}
for(const line of readFileSync(0,'utf8').trimEnd().split('\n')){
 const[id,k,aa,bb]=line.split('\t'),kind=+k,a=+aa,b=+bb,title=model(kind,a,b),mode=kind===2?b:0,log=[],error={opaque:true},input={identity:true}
 const command={get title(){log.push('title');if(mode===5||mode===6||mode===7)throw error;return title}}
 if(mode)command.label=function(arg){log.push('label:'+Number(this===command)+':'+Number(arg===input));if(mode===4)throw error;return mode===1||mode===6?undefined:mode===2?null:mode===3?'':mode===7?'CuStOm\ud800':mode===7?'CuStOm\ud800':'custom\ud800'}
 let output
 try{output='ok:'+bytes(api.portCommandLabel(command,input))}catch(e){if(e!==error)throw e;output='error:same'}
 process.stdout.write(id+'\t'+bytes(title)+'\t'+output+'\t'+log.join(',')+'\n')
}
