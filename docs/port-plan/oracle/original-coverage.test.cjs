// SPDX-License-Identifier: AGPL-3.0-only
const test=require('node:test'),assert=require('node:assert/strict'),fs=require('node:fs'),zlib=require('node:zlib'),path=require('node:path'),crypto=require('node:crypto')
const {summarize,markdown,countAt,segments}=require('./original-coverage-report.cjs')
const folder=path.join(__dirname,'../evidence/original-coverage-2026-11-02'),hash=b=>crypto.createHash('sha256').update(b).digest('hex')
function fixture(){return JSON.parse(zlib.gunzipSync(fs.readFileSync(folder+'/raw.json.gz')))}
test('deepest zero-count block overrides executed outer ranges',()=>{
 const ranges=[{startOffset:0,endOffset:100,count:1},{startOffset:10,endOffset:40,count:0},{startOffset:20,endOffset:25,count:7}]
 assert.equal(countAt(ranges,9),1);assert.equal(countAt(ranges,10),0);assert.equal(countAt(ranges,22),7);assert.equal(countAt(ranges,40),1);assert.equal(countAt(ranges,100),0)
})
test('regenerate full committed report from raw V8 maps/ranges',()=>{
 const raw=fixture(),actual=summarize(raw),stored=JSON.parse(fs.readFileSync(folder+'/report.json'))
 assert.deepEqual(actual,stored);assert.equal(markdown(actual),fs.readFileSync(folder+'/report.md','utf8'));assert.equal(actual.cases,1422);assert.equal(actual.files.length,8);assert.equal(raw.dirty,'');assert.equal(actual.acceptanceClaimed,false)
 assert.ok(actual.unmappedSources.length>0);assert.ok(actual.files.find(f=>f.path==='src/commands/schema.ts').lines.unreached.length>0)
 assert.ok(actual.files.some(f=>f.blocks.some(b=>b.count===0)))
})
test('coverage manifest hashes every committed output',()=>{
 const m=JSON.parse(fs.readFileSync(folder+'/manifest.json'))
 for(const [name,item]of Object.entries(m.files)){const b=fs.readFileSync(folder+'/'+name);assert.equal(b.length,item.bytes);assert.equal(hash(b),item.sha256)}
})
test('missing coverage, source maps, script ranges or duplicate identities fail',()=>{
 const raw=fixture();for(const mutate of [r=>r.coverage=[],r=>r.observations=[],r=>r.observations.push(r.observations[0]),r=>r.coverage=r.coverage.filter(s=>s.url!==r.sourceMaps[0].url),r=>r.sourceMaps[0].map.sourcesContent[0]+='changed']){const r=structuredClone(raw);mutate(r);assert.throws(()=>summarize(r))}
})
test('changed block counts change the report; unknown source mapping refuses',()=>{
 const raw=fixture(),r=structuredClone(raw);for(const s of r.coverage)for(const f of s.functions)for(const b of f.ranges)b.count=0
 assert.notDeepEqual(summarize(r),summarize(raw));assert.ok(summarize(r).files.every(f=>f.lines.reached.length===0));assert.throws(()=>segments({map:{sources:[],sourcesContent:[]}}))
})
