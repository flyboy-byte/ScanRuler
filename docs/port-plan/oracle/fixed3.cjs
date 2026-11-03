// SPDX-License-Identifier: AGPL-3.0-only
// Original primitive intrinsic, after verifying both pinned application calls.
const {execFileSync}=require('node:child_process'),{readFileSync,writeFileSync}=require('node:fs'),{createHash}=require('node:crypto')
const pin='2cd89c6769a10c27ee7754b477ada521c9ef5cb7',path='src/core/fit/fitSphereFromSeed.ts',source=execFileSync('git',['show',pin+':'+path],{encoding:'utf8'})
for(const expression of ['free.toFixed(3)','known.toFixed(3)'])if(source.split(expression).length!==2)throw Error('Pinned fixed3 expression changed: '+expression)
for(const line of readFileSync(process.argv[2]??0,'utf8').trim().split('\n')){
 const [id,hex]=line.split('\t'),input=Buffer.from(hex,'hex'),value=input.readDoubleBE()
 const kind=Number.isNaN(value)?'nan':value===Infinity?'inf':value===-Infinity?'-inf':Object.is(value,-0)?'-0':'finite'
 const text=value.toFixed(3)
 process.stdout.write(id+'\t'+hex+'\t'+kind+'\t'+Buffer.from(text).toString('hex')+'\t'+input.toString('hex')+'\n')
}
if(process.argv[3])writeFileSync(process.argv[3],JSON.stringify({reference_sha:pin,source_path:path,source,source_sha256:createHash('sha256').update(source).digest('hex'),expressions:['free.toFixed(3)','known.toFixed(3)'],node:process.versions,comparator:'exact primitive category and complete output bytes; no numerical tolerance'},null,2)+'\n')
