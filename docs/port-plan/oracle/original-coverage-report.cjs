// SPDX-License-Identifier: AGPL-3.0-only
// Convert V8 detailed ranges through the exact compiler's source maps.
const fs=require('node:fs'),crypto=require('node:crypto')
const hash=b=>crypto.createHash('sha256').update(b).digest('hex')
function segments(info){
 if(info.map.sources.length!==1||info.map.sourcesContent?.[0]!==info.source)throw Error('Source map identity')
 const alphabet='ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/',out=[];let source=0,originalLine=0,originalColumn=0;let offset=0
 for(const [line,row]of info.map.mappings.split(';').entries()){
  let column=0
  for(const part of row.split(',').filter(Boolean)){
   let fields=[],n=0,shift=0
   for(const c of part){const d=alphabet.indexOf(c);if(d<0)throw Error('Bad VLQ');n+=(d&31)*2**shift;if(d&32)shift+=5;else{fields.push(n&1?-(n>>1):n>>1);n=0;shift=0}}
   if(shift||![1,4,5].includes(fields.length))throw Error('Malformed source-map segment')
   column+=fields[0];if(fields.length===1)continue
   source+=fields[1];originalLine+=fields[2];originalColumn+=fields[3];if(source!==0)throw Error('Unexpected source index')
   out.push({offset:offset+column,line:originalLine+1,column:originalColumn,generatedLine:line+1,generatedColumn:column})
  }
  offset+=(info.js.split('\n')[line]?.length??0)+1
 }
 return out
}
function countAt(ranges,offset){
 let closest=null
 for(const r of ranges)if(r.startOffset<=offset&&offset<r.endOffset&&(!closest||r.endOffset-r.startOffset<closest.endOffset-closest.startOffset))closest=r
 return closest?.count??0
}
function summarize(raw){
 if(raw.contract!=='original.coverage.raw.v1'||!raw.observations.length||!raw.coverage.length)throw Error('Empty/wrong coverage')
 if(new Set(raw.observations.map(c=>c.unit+'/'+c.id)).size!==raw.observations.length)throw Error('Duplicate observations')
 const files=[]
 for(const info of raw.sourceMaps){
  const records=raw.coverage.filter(r=>r.url===info.url);if(!records.length)throw Error('Missing mapped source '+info.path)
  const map=segments(info);if(!map.length)throw Error('Empty source map')
  const lines=new Map(),blocks=new Map(),functions=new Map();const at=offset=>{let chosen=null;for(const s of map){if(s.offset>offset)break;chosen=s}return chosen?{line:chosen.line,column:chosen.column}:null}
  for(const record of records){
   if(!record.functions.length)throw Error('No function ranges')
   const ranges=record.functions.flatMap(f=>f.ranges)
   for(const s of map){const count=countAt(ranges,s.offset);let row=lines.get(s.line);if(!row){row={line:s.line,hit:false,miss:false};lines.set(s.line,row)}if(count>0)row.hit=true;else row.miss=true}
   for(const f of record.functions){
    const root=f.ranges[0];if(!root)throw Error('Empty function');const id=[f.functionName,root.startOffset,root.endOffset].join(':');let fn=functions.get(id)
    if(!fn){fn={name:f.functionName,start:root.startOffset,end:root.endOffset,at:at(root.startOffset),count:0};functions.set(id,fn)}fn.count+=root.count
    if(f.isBlockCoverage)for(const r of f.ranges.slice(1)){
     const id=[root.startOffset,r.startOffset,r.endOffset].join(':');let row=blocks.get(id)
     if(!row){row={function:f.functionName,start:r.startOffset,end:r.endOffset,at:at(r.startOffset),to:at(Math.max(r.startOffset,r.endOffset-1)),count:0};blocks.set(id,row)}row.count+=r.count
    }
   }
  }
  const lineRows=[...lines.values()].sort((a,b)=>a.line-b.line),blockRows=[...blocks.values()].sort((a,b)=>a.start-b.start||a.end-b.end),functionRows=[...functions.values()].sort((a,b)=>a.start-b.start||a.end-b.end)
  files.push({path:info.path,sourceSha256:hash(info.source),generatedSha256:hash(info.js),sourceMapSha256:hash(JSON.stringify(info.map)),scriptInstances:records.length,lines:{mapped:lineRows.length,reached:lineRows.filter(r=>r.hit).map(r=>r.line),unreached:lineRows.filter(r=>!r.hit).map(r=>r.line),partial:lineRows.filter(r=>r.hit&&r.miss).map(r=>r.line)},blocks:blockRows,functions:functionRows})
 }
 return {contract:'original.coverage.report.v1',candidateSha:raw.candidateSha,nativeCandidateSha:raw.nativeCandidateSha,referenceSha:raw.referenceSha,node:raw.node,typescript:raw.typescript,cases:raw.observations.length,suites:raw.runs,files,unmappedSources:raw.unmappedSources,branchMetric:'V8 observed detailed block-range union; uncalled functions may hide internal branches. Not a static condition/edge or MC/DC denominator.',lineMetric:'Compiler source-map segments classified by smallest enclosing V8 function/block range; line reached if any mapped segment executes; partial if some also unexecuted.',excludedCampaigns:['Independent fuzz campaigns','Chrome image audit and browser-only work','Unmapped stripped/concatenated geometry helpers'],acceptanceClaimed:false}
}
function markdown(d){
 const lines=['<!-- SPDX-License-Identifier: AGPL-3.0-only -->','# Original corpus coverage — 2026-11-02','',`Reference \`${d.referenceSha}\`; tool checkpoint \`${d.candidateSha}\`; native input/results checkpoint \`${d.nativeCandidateSha}\`.`, '', `${d.cases} complete output/sidecar replays under ${d.node}, TypeScript${d.typescript}; all identical to the native-gate original artifacts. Coverage is diagnostic, not parity or acceptance.`, '', '| Original source | Mapped lines reached / total | Unreached lines | Unreached observed blocks / total |','| --- | ---: | --- | ---: |']
 for(const f of d.files)lines.push(`| ${f.path} | ${f.lines.reached.length} / ${f.lines.mapped} | ${f.lines.unreached.join(', ')||'none'} | ${f.blocks.filter(b=>b.count===0).length} / ${f.blocks.length} |`)
 lines.push('',d.lineMetric,'',d.branchMetric,'','## Sources not mapped by this report','',...d.unmappedSources.map(p=>'- `'+p+'`'),'', 'Those source/helper comparisons still run in the mandatory native gate. Their source locations are unknown here, never zero uncovered or 100% reached. Each affected implemented unit retains an explicit coverage follow-up. Unreached lines and zero-count block/function records are generated in report.json; do not edit the generated lists.','', 'Independent fuzz, actual browser execution and static branch/condition coverage remain open. No thresholds, original source code, fixtures, seeds or comparisons changed.','')
 return lines.join('\n')
}
if(require.main===module){const input=process.argv[2],out=process.argv[3];if(!input||!out)throw Error('usage: original-coverage-report.cjs RAW_JSON OUTPUT_DIRECTORY');const raw=JSON.parse(fs.readFileSync(input)),report=summarize(raw);fs.mkdirSync(out,{recursive:true});fs.writeFileSync(out+'/report.json',JSON.stringify(report,null,2)+'\n');fs.writeFileSync(out+'/report.md',markdown(report));console.log(report.files.length,'mapped files;',report.cases,'cases')}
module.exports={segments,countAt,summarize,markdown}
