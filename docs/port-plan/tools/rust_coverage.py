#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-only
"""LLVM coverage of mandatory DIFFERENTIAL tests only; system Rust, no rustup."""
import argparse,gzip,hashlib,json,os,re,subprocess,time
from pathlib import Path
import ci_report
ROOT=Path(__file__).resolve().parents[3];PLAN=ROOT/'docs/port-plan'

def sha(b):return hashlib.sha256(b).hexdigest()
def capture(args,cwd=ROOT,env=None):return subprocess.check_output(args,cwd=cwd,env=env,text=True)
def parse_lcov(text):
 files={};current=None
 for line in text.splitlines():
  if line.startswith('SF:'):
   name=line[3:]
   if name in files:raise ValueError('Duplicate LCOV source')
   current={'lines':{},'branches':[],'summary_lines':None,'summary_hits':None};files[name]=current
  elif line.startswith('DA:'):
   if current is None:raise ValueError('Line without source')
   fields=line[3:].split(',');n,count=int(fields[0]),int(fields[1])
   if n<=0 or count<0 or n in current['lines']:raise ValueError('Invalid/duplicate LCOV line')
   current['lines'][n]=count
  elif line.startswith('BRDA:'):
   if current is None:raise ValueError('Branch without source')
   current['branches'].append(line[5:])
  elif line.startswith('LF:'):
   if current is None or current['summary_lines'] is not None:raise ValueError('LCOV line total')
   current['summary_lines']=int(line[3:])
  elif line.startswith('LH:'):
   if current is None or current['summary_hits'] is not None:raise ValueError('LCOV hit total')
   current['summary_hits']=int(line[3:])
  elif line=='end_of_record':
   if current is None or current['summary_lines'] is None or current['summary_hits'] is None:raise ValueError('Missing LCOV totals')
   if not 0<=current['summary_hits']<=current['summary_lines'] or current['summary_lines']<len(current['lines']):raise ValueError('Invalid LCOV totals')
   current=None
 if not files or current is not None:raise ValueError('Empty/unfinished LCOV')
 return files

def summarize(lcov,metadata,sources):
 if metadata['contract']!='rust.coverage.run.v1' or not metadata['native_cases'] or not metadata['native_suites']:raise ValueError('Missing comparison evidence')
 parsed=parse_lcov(lcov);by_relative={}
 for name,item in parsed.items():
  # Raw paths are recorded by the instrumented compiler. Source identity, not
  # this report's local machine prefix, maps them to committed owned files.
  marker='/rust/crates/'
  if marker not in name:continue
  relative='rust/crates/'+name.split(marker,1)[1]
  if relative in by_relative:raise ValueError('Ambiguous owned source path')
  by_relative[relative]=item
 files=[]
 for source in sources:
  p=source['path'];item=by_relative.get(p);lines=item['lines'] if item else {};line_count=len(source['text'].splitlines())
  if any(n>line_count for n in lines):raise ValueError('Coverage line beyond source')
  covered=sorted(n for n,c in lines.items()if c>0);missed=sorted(n for n,c in lines.items()if c==0)
  inline_start=next((i for i,line in enumerate(source['text'].splitlines(),1)if line.strip()=='#[cfg(test)]'),None)
  # Current owned files put their inline test modules at the end. Do not infer
  # their boundaries: retain all file data, and expose the marker as a caution.
  inline_test_lines=[n for n in lines if inline_start is not None and n>=inline_start]
  discrepancy=bool(item and (item['summary_lines']!=len(lines) or item['summary_hits']!=len(covered)))
  files.append(dict(llvm_summary_lines=item['summary_lines'] if item else None,llvm_summary_hits=item['summary_hits'] if item else None,line_summary_disagrees_with_explicit_records=discrepancy,inline_test_section_marker=inline_start,inline_test_instrumented_lines=sorted(inline_test_lines),path=p,sha256=sha(source['text'].encode()),units=source['units'],status='measured' if lines else 'no-emitted-instrumented-lines',instrumented_lines=len(lines),reached=covered,unreached=missed,uninstrumented=[n for n in range(1,line_count+1)if n not in lines],branch_records=item['branches'] if item else []))
 if not any(f['instrumented_lines']for f in files):raise ValueError('No owned source coverage')
 branches=any(f['branch_records']for f in files)
 return dict(contract='rust.coverage.report.v1',candidate_sha=metadata['candidate_sha'],reference_sha=metadata['reference_sha'],toolchain=metadata['toolchain'],native_suites=metadata['native_suites'],native_cases=metadata['native_cases'],ordinary_tests_included=False,files=files,branch_metric='LLVM emitted branch records only; not an exhaustive static denominator' if branches else 'Unavailable: stable -C instrument-coverage emits regions/lines, no branch records in this run. Branch reach is unknown, not 100%.',acceptance_claimed=False)

def markdown(report):
 lines=['<!-- SPDX-License-Identifier: AGPL-3.0-only -->','# Rust differential corpus coverage — 2026-11-02','',f"Candidate `{report['candidate_sha']}`; original `{report['reference_sha']}`.", '',f"Only mandatory differential tests: {report['native_suites']} suites /{report['native_cases']} identities. Ordinary unit tests are excluded from execution. File-level records can include their unexecuted inline bodies. LLVM explicit DA line counts are retained separately from LLVM LF/LH summary counters; discrepancies are reported, never normalized away.",'','| Rust source | Instrumented lines reached / total | Unreached explicit DA lines | LLVM LF / LH | DA/summary discrepancy |','| --- | ---: | --- | ---: | --- |']
 for f in report['files']:lines.append(f"| {f['path']} | {len(f['reached'])} / {f['instrumented_lines']} | {', '.join(map(str,f['unreached'])) or ('none measured' if f['instrumented_lines'] else 'unknown: no emitted coverage')} | {f['llvm_summary_lines']} / {f['llvm_summary_hits']} | {f['line_summary_disagrees_with_explicit_records']} |")
 lines+=['',report['branch_metric'],'','LLVM LF/LH counters can disagree with explicit DA records for these multi-object exports. Both are preserved, with unresolved interpretation flagged per file; no denominator or pass percentage is inferred. Inline cfg(test) section markers and their instrumented lines are also explicit (some differential harnesses live there). Uninstrumented lines (comments, declarations, code not emitted/instantiated) are listed separately in report.json, never presumed covered or executable. Every measured source is hashed and mapped to canonical owning units. Missing owners/files remain explicit. Host allocation/depth/work guards may be unexecuted by the source comparison corpus; ordinary host tests are deliberately not blended into this report.','', 'The system toolchain is recorded, not an immutable release pin. No tests, tolerances, seeds, outputs or source code were weakened. No acceptance or exact-base ratchet claim.','']
 return '\n'.join(lines)

def source_inventory():
 units=ci_report.units();sources=[]
 for crate in ['port-core','port-io','port-engine']:
  for f in sorted((ROOT/'rust/crates'/crate/'src').rglob('*.rs')):
   p=f.relative_to(ROOT).as_posix();owners=[u['id']for u in units if u['status']=='implemented' and p in u.get('rust_paths',[])]
   sources.append(dict(path=p,text=f.read_text(),units=owners))
 return sources

def write_report(folder,lcov,metadata,sources):
 report=summarize(lcov,metadata,sources)
 (folder/'report.json').write_text(json.dumps(report,indent=2)+'\n');(folder/'report.md').write_text(markdown(report));return report

def run(out):
 if capture(['git','status','--porcelain']).strip():raise ValueError('Coverage run requires a clean committed checkpoint')
 out=out.resolve();out.mkdir(parents=True,exist_ok=False);profiles=out/'profiles';profiles.mkdir()
 candidate=capture(['git','rev-parse','HEAD']).strip();rust=capture(['rustc','-vV']);prof=capture(['llvm-profdata','--version']);cov=capture(['llvm-cov','--version'])
 major=re.search(r'LLVM version: (\d+)',rust)[1]
 if not re.search(r'LLVM version '+major+r'\.',prof) or not re.search(r'LLVM version '+major+r'\.',cov):raise ValueError('LLVM tool major differs from rustc; no install performed')
 env=os.environ.copy()
 if env.get('RUSTFLAGS') or env.get('CARGO_ENCODED_RUSTFLAGS'):raise ValueError('Explicit coverage environment required; inherited RUSTFLAGS refused')
 env.update(RUSTFLAGS='-C instrument-coverage -C link-dead-code',CARGO_INCREMENTAL='0',CARGO_TARGET_DIR=str(ROOT/'rust/target/coverage-build'),LLVM_PROFILE_FILE=str(profiles/'%m-%p.profraw'))
 build=['cargo','test','-p','port-core','-p','port-io','-p','port-engine','--tests','--no-run','--message-format=json']
 with (out/'build.stderr.log').open('w')as err:
  result=subprocess.run(build,cwd=ROOT/'rust',env=env,text=True,stdout=subprocess.PIPE,stderr=err,check=True)
 (out/'build-messages.jsonl').write_text(result.stdout);executables=[]
 for line in result.stdout.splitlines():
  m=json.loads(line)
  if m.get('reason')=='compiler-artifact'and m.get('executable')and m.get('profile',{}).get('test'):executables.append(m['executable'])
 executables=sorted(set(executables))
 if not executables:raise ValueError('No instrumented test objects')
 with (out/'oracle.log').open('w')as log:
  subprocess.run(['fnm','exec','--using=22','--','cargo','xtask','oracle'],cwd=ROOT/'rust',env=env,stdout=log,stderr=subprocess.STDOUT,check=True)
 if capture(['git','rev-parse','HEAD']).strip()!=candidate or capture(['git','status','--porcelain']).strip():raise ValueError('Checkpoint changed during coverage')
 evidence=ci_report.evidence(ROOT/'rust/target',ci_report.roster(),candidate,ci_report.status()['reference_sha'])
 (out/'native-evidence.json').write_text(json.dumps(evidence,indent=2)+'\n')
 raw_profiles=sorted(profiles.glob('*.profraw'))
 if not raw_profiles or any(p.stat().st_size==0 for p in raw_profiles):raise ValueError('Missing/empty LLVM profiles')
 merged=out/'merged.profdata';subprocess.run(['llvm-profdata','merge','-sparse',*map(str,raw_profiles),'-o',str(merged)],check=True)
 command=['llvm-cov','export',executables[0],'-instr-profile='+str(merged)]
 for obj in executables[1:]:command+=['-object',obj]
 for name,args in [('llvm-export.json',[]),('coverage.lcov',['-format=lcov'])]:
  with (out/name).open('w')as output,(out/(name+'.stderr')).open('w')as err:subprocess.run(command+args,stdout=output,stderr=err,check=True)
 sources=source_inventory();(out/'sources.json').write_text(json.dumps(sources,indent=2)+'\n')
 metadata=dict(contract='rust.coverage.run.v1',candidate_sha=candidate,reference_sha=evidence['reference_sha'],toolchain=dict(rustc=rust,cargo=capture(['cargo','-V']).strip(),node=capture(['fnm','exec','--using=22','node','-v']).strip(),llvm_profdata=prof,llvm_cov=cov),flags=env['RUSTFLAGS'],ordinary_tests_included=False,native_suites=len(evidence['suites']),native_cases=sum(len(s['cases'])for s in evidence['suites']),profiles=[dict(name=p.name,bytes=p.stat().st_size,sha256=sha(p.read_bytes()))for p in raw_profiles],executables=[dict(path=p,bytes=Path(p).stat().st_size,sha256=sha(Path(p).read_bytes()))for p in executables],source_inventory_sha256=sha((out/'sources.json').read_bytes()))
 (out/'run.json').write_text(json.dumps(metadata,indent=2)+'\n');report=write_report(out,(out/'coverage.lcov').read_text(),metadata,sources)
 print('LLVM differential coverage:',candidate,len(report['files']),'owned sources;',metadata['native_suites'],'suites /',metadata['native_cases'],'cases;',out)

if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('command',choices=['run','report']);p.add_argument('--out',type=Path,default=ROOT/('rust/target/rust-coverage/run-'+str(time.time_ns())));a=p.parse_args()
 if a.command=='run':run(a.out)
 else:write_report(a.out,(a.out/'coverage.lcov').read_text(),json.loads((a.out/'run.json').read_text()),json.loads((a.out/'sources.json').read_text()))
