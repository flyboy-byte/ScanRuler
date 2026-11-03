# SPDX-License-Identifier: AGPL-3.0-only
"""Read-only dated PR snapshot; integrity/display tooling, not merge authorization."""
import argparse
from concurrent.futures import ThreadPoolExecutor
from datetime import datetime, timezone
import json
from pathlib import Path
import re
import subprocess

PLAN=Path(__file__).resolve().parents[1]
REPO='flyboy-byte/ScanRuler'
CHECKS=['plan','npm-licenses','fast','native-oracle','catalog-contracts','browser-wasm','gate','source-tests','source-mcp','source-browser','test-and-build']
PIN='2cd89c6769a10c27ee7754b477ada521c9ef5cb7'
SNAPSHOT=PLAN/'evidence/2026-11-03-pr-review-refresh.json'
DOCUMENT=PLAN/'REVIEW-QUEUE-2026-11-03-SECOND-PASS.md'


def api(path):
    return json.loads(subprocess.check_output(['gh','api',path],text=True))


def capture(exclude):
    def row(p):
        checks=api(f'repos/{REPO}/commits/{p["head"]["sha"]}/check-runs?filter=latest&per_page=100')['check_runs']
        return {'number':p['number'],'title':p['title'],'url':p['html_url'],'isDraft':p['draft'],'baseRefName':p['base']['ref'],'headRefName':p['head']['ref'],'headRefOid':p['head']['sha'],
                'statusCheckRollup':[{'name':c['name'],'id':c['id'],'status':c['status'].upper(),'conclusion':(c['conclusion'] or '').upper(),'url':c['details_url']}for c in checks]}
    prs=api(f'repos/{REPO}/pulls?state=open&per_page=100')
    if len(prs)==100:raise ValueError('Open PR pagination required; no truncated snapshot')
    with ThreadPoolExecutor(max_workers=4)as pool:rows=list(pool.map(row,[p for p in prs if p['number']!=exclude]))
    present={p['number']for p in rows};merged={}
    for number in [74,69,64]:
        if number not in present:
            p=api(f'repos/{REPO}/pulls/{number}')
            if not p['merged_at']:raise ValueError('Closed integration dependency is not merged')
            merged[str(number)]={'number':number,'head_sha':p['head']['sha'],'merge_sha':p['merge_commit_sha'],'merged_at':p['merged_at'],'url':p['html_url']}
    main=api(f'repos/{REPO}/branches/main')['commit']['sha']
    for p in merged.values():
        relation=api(f'repos/{REPO}/compare/{p["merge_sha"]}...{main}')
        p['main_contains']=relation['status']in ['ahead','identical']
    data={'schema':'port.review.queue.v1','reference_sha':PIN,'observed_at_utc':datetime.now(timezone.utc).strftime('%Y-%m-%d %H:%M:%S UTC'),'excluded_handoff_pr':exclude,
          'main_sha':main,'required_checks':CHECKS,'merged_dependencies':merged,
          'extra_dependencies':{'75':[74,69,64]},'prs':sorted(rows,key=lambda p:p['number'])}
    validate(data);return data


def validate(data):
    if data.get('schema')!='port.review.queue.v1' or data.get('required_checks')!=CHECKS:raise ValueError('Wrong schema/required check identities')
    if data.get('reference_sha')!=PIN:raise ValueError('Wrong immutable reference')
    if not re.fullmatch('[0-9a-f]{40}',str(data.get('main_sha',''))):raise ValueError('Missing exact main SHA')
    if not re.fullmatch(r'\d{4}-\d\d-\d\d \d\d:\d\d:\d\d UTC',str(data.get('observed_at_utc',''))):raise ValueError('Missing absolute UTC observation')
    prs=data.get('prs');excluded=data.get('excluded_handoff_pr')
    if not isinstance(prs,list)or not prs or type(excluded)is not int:raise ValueError('Empty/incomplete finite snapshot')
    numbers=[p['number']for p in prs];refs={p['headRefName']:p['number']for p in prs}
    if len(set(numbers))!=len(prs)or len(refs)!=len(prs)or excluded in numbers:raise ValueError('Duplicate/self-referential PR snapshot')
    if 75 in numbers and data.get('extra_dependencies',{}).get('75')!=[74,69,64]:raise ValueError('Missing known integration edges')
    edges={}
    for p in prs:
        if type(p['number'])is not int or not re.fullmatch('[0-9a-f]{40}',p['headRefOid']):raise ValueError('Invalid exact PR head')
        if type(p['isDraft'])is not bool or p['url']!=f'https://github.com/{REPO}/pull/{p["number"]}':raise ValueError('Invalid PR identity')
        base=p['baseRefName']
        if base!='main' and base not in refs:raise ValueError('Missing open base dependency: '+base)
        edges[p['number']]=[]if base=='main'else[refs[base]]
        seen=set()
        for c in p['statusCheckRollup']:
            if c['name']in seen:raise ValueError('Ambiguous duplicate check name')
            seen.add(c['name'])
            if c['status']not in ['COMPLETED','IN_PROGRESS','QUEUED','WAITING','REQUESTED','PENDING']:raise ValueError('Unknown check state')
            if c['status']!='COMPLETED'and c['conclusion']:raise ValueError('Unfinished check has a conclusion')
    merged=data.get('merged_dependencies',{})
    for key,p in merged.items():
        if int(key)in edges or p.get('number')!=int(key)or p.get('main_contains')is not True or not p.get('merged_at'):
            raise ValueError('Invalid merged dependency/ancestry')
        if any(not re.fullmatch('[0-9a-f]{40}',str(p.get(k,'')))for k in ['head_sha','merge_sha']):raise ValueError('Missing exact merged dependency identity')
    for number,parents in data.get('extra_dependencies',{}).items():
        number=int(number)
        if number not in edges or not isinstance(parents,list)or len(set(parents))!=len(parents):raise ValueError('Unknown/duplicate integration dependency')
        if any(p not in edges and str(p)not in merged for p in parents):raise ValueError('Missing integration parent')
        edges[number]+=[p for p in parents if p in edges]
    def walk(number,active,done):
        if number in active:raise ValueError('Review dependency cycle')
        if number in done:return
        for parent in edges[number]:walk(parent,active|{number},done)
        done.add(number)
    done=set()
    for number in edges:walk(number,set(),done)
    return edges


def check_state(p):
    checks={c['name']:c for c in p['statusCheckRollup']}
    failed=[name for name in CHECKS if name in checks and checks[name]['status']=='COMPLETED'and checks[name]['conclusion']!='SUCCESS']
    pending=[name for name in CHECKS if name not in checks or checks[name]['status']!='COMPLETED']
    parts=[]
    if failed:parts.append('red: '+','.join(failed))
    if pending:parts.append('pending/missing: '+','.join(pending))
    return '; '.join(parts)or'11 green'


def render(data):
    edges=validate(data)
    lines=['# Second-pass port review queue','',f"Observed {data['observed_at_utc']}. Main: `{data['main_sha']}`.",
           f"This finite snapshot covers {len(data['prs'])} open PRs and excludes its own handoff PR #{data['excluded_handoff_pr']}. Refresh live heads/checks before merging. No merge or acceptance is authorized by this table.",'',
           'The [earlier dated handoff](REVIEW-HANDOFF-2026-11-03.md) remains historical. This snapshot retains every required check identity; missing, pending, skipped or red checks cannot be displayed as green. Raw data: [review snapshot](evidence/2026-11-03-pr-review-refresh.json).','',
           '| PR | Dependencies | Exact head | Draft | Observed required checks | Scope |','| --- | --- | --- | --- | --- | --- |']
    for p in data['prs']:
        extra=data.get('extra_dependencies',{}).get(str(p['number']),[])
        deps=', '.join('#'+str(n)+(' (merged)'if str(n)in data.get('merged_dependencies',{})else'')for n in sorted(set(edges[p['number']]+extra)))or'main'
        title=p['title'].replace('|','\\|').replace('\n',' ')
        lines.append(f"| [#{p['number']}]({p['url']}) | {deps} | `{p['headRefOid']}` | {'yes'if p['isDraft']else'no'} | {check_state(p)} | {title} |")
    if data.get('merged_dependencies'):
        lines+=['','Merged integration dependencies are retained with exact identities and checked ancestry in the observed main:','']
        for p in data['merged_dependencies'].values():lines.append(f"- [#{p['number']}]({p['url']}): `{p['merge_sha']}`, merged {p['merged_at']}.")
    return '\n'.join(lines)+'\n'


def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('command',choices=['capture','check']);parser.add_argument('--exclude',type=int,default=84);args=parser.parse_args()
    if args.command=='capture':
        data=capture(args.exclude);SNAPSHOT.write_text(json.dumps(data,indent=2)+'\n');DOCUMENT.write_text(render(data))
    else:
        data=json.loads(SNAPSHOT.read_text())
        if DOCUMENT.read_text()!=render(data):raise ValueError('Stale generated review table')
    print(f"Review snapshot valid: {len(data['prs'])} exact heads; metadata only, not merge approval")

if __name__=='__main__':main()
