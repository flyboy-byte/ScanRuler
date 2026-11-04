// SPDX-License-Identifier: AGPL-3.0-only
import {readFileSync} from 'node:fs'
if(Number(process.versions.node.split('.')[0])!==22)throw Error('Pinned primitive oracle requires Node22')
const api=await import(new URL('../../../rust/target/number-string-reference/dist-session/registry.mjs',import.meta.url))
for(const line of readFileSync(0,'utf8').trimEnd().split('\n')){
 const [id,bits]=line.split('\t');const b=Buffer.alloc(8);b.writeBigUInt64LE(BigInt('0x'+bits));const value=b.readDoubleLE();const error=api.asCommandError(value)
 if(error.name!=='CommandError'||error.code!=='failed')throw Error('unexpected original numeric branch')
 const category=Number.isNaN(value)?'nan':value===Infinity?'inf':value===-Infinity?'-inf':Object.is(value,-0)?'-0':'finite'
 process.stdout.write([id,bits,category,error.message,JSON.stringify(value)].join('\t')+'\n')
}
