# SPDX-License-Identifier: AGPL-3.0-only
"""Evidence metadata/integrity gates; never substitutes for executing a comparison."""
import gzip
import hashlib
import json
from pathlib import Path
import re

LEVELS = ('none', 'fixed-corpus', 'fuzzed', 'coverage-reviewed')
SUMMARY = re.compile(rb'fuzz unit=([a-z][a-z0-9_-]*) seed=(\d+) cases=(\d+) identical_success=(\d+) identical_refusal=(\d+) host_limit=(\d+) original_timeout=(\d+) divergences=(\d+)')
COUNTERS = ('seed', 'cases', 'identical_success', 'identical_refusal', 'host_limit', 'original_timeout', 'divergences')
OBSERVATIONS = ['success', 'progress', 'error', 'output-bytes']


def local(base, name):
    if not isinstance(name, str) or not name or Path(name).is_absolute() or '..' in Path(name).parts:
        raise ValueError('unsafe local evidence path')
    path = (base / name).resolve()
    if not path.is_relative_to(base.resolve()) or not path.is_file():
        raise ValueError('missing or escaped evidence path: ' + name)
    return path


def blob(path):
    value = path.read_bytes()
    return gzip.decompress(value) if path.suffix == '.gz' else value


def source_hashes(unit, report, plan):
    values = report.get('source_hashes')
    if not isinstance(values, dict) or not values:
        raise ValueError('missing current source/generator hashes')
    checkout = plan.parents[1]
    needed = {p for p in unit.get('rust_paths', []) if '/src/' in p and p.endswith('.rs')}
    if not needed <= values.keys():
        raise ValueError('proof omits owned implementation paths')
    for name, expected in values.items():
        if not isinstance(expected, str) or not re.fullmatch('[0-9a-f]{64}', expected):
            raise ValueError('invalid source hash')
        if hashlib.sha256(local(checkout, name).read_bytes()).hexdigest() != expected:
            raise ValueError('stale source/generator hash: ' + name)


def proof(unit, kind, baseline, plan):
    if plan is None:
        raise ValueError('proof validation requires a plan root')
    links = [e for e in unit.get('verification_evidence', []) if isinstance(e, dict) and e.get('kind') == kind]
    if len(links) != 1:
        raise ValueError('missing/ambiguous ' + kind + ' proof')
    path = local(plan, links[0].get('location'))
    report = json.loads(blob(path))
    expected = {'fixed-corpus': 'port.verification.fixed.v1', 'fuzz-campaign': 'port.verification.fuzz.v1', 'coverage-review': 'port.verification.review.v1'}[kind]
    if report.get('schema') != expected or report.get('reference_sha') != baseline:
        raise ValueError('wrong proof schema or original reference')
    if not re.fullmatch('[0-9a-f]{40}', str(report.get('candidate_sha', ''))):
        raise ValueError('missing exact tested proof candidate')
    scopes = report.get('scope_units')
    if not isinstance(scopes, list) or unit['id'] not in scopes or len(set(scopes)) != len(scopes):
        raise ValueError('proof scope does not identify this unit')
    source_hashes(unit, report, plan)
    return path, report


def cases(report):
    values = report.get('case_ids')
    if not isinstance(values, list) or not values or any(not isinstance(v, str) or not v for v in values) or len(set(values)) != len(values):
        raise ValueError('empty/duplicate proof case profile')


def fixed(unit, baseline, plan):
    for e in unit.get('evidence', []):
        values = e.get('case_ids', [])
        if (e.get('kind') == 'differential' and e.get('reference_sha') == baseline
                and re.fullmatch('[0-9a-f]{40}', str(unit.get('implemented_commit', '')))
                and e.get('candidate_sha') == unit['implemented_commit']
                and isinstance(values, list) and values and all(isinstance(v, str) and v for v in values)
                and len(set(values)) == len(values)):
            if plan is not None:
                local(plan, e.get('location'))
            return
    path, report = proof(unit, 'fixed-corpus', baseline, plan)
    cases(report)
    if report['candidate_sha'] != unit.get('implemented_commit'):
        raise ValueError('fixed proof differs from tested implementation')
    if report.get('passed') is not True or report.get('runner') != 'python-unittest':
        raise ValueError('fixed tooling proof is not a passing unittest run')
    raw = blob(local(path.parent, report.get('log')))
    if hashlib.sha256(raw).hexdigest() != report.get('log_sha256'):
        raise ValueError('fixed tooling log hash differs')
    text = raw.decode('utf-8')
    identities = re.findall(r'^\S+ \(([^)]+)\) \.\.\. ok$', text, re.M)
    summary = re.findall(r'^Ran (\d+) tests? in [0-9.]+s$', text, re.M)
    if (len(summary) != 1 or int(summary[0]) != len(report['case_ids'])
            or sorted(identities) != sorted(report['case_ids'])
            or not re.search(r'\nOK\s*\Z', text)):
        raise ValueError('fixed tooling log does not support positive passing case profile')


def fuzz(unit, baseline, plan):
    path, report = proof(unit, 'fuzz-campaign', baseline, plan)
    if report.get('observations') != OBSERVATIONS:
        raise ValueError('fuzz does not compare full declared values/state')
    family = report.get('unit_families', {}).get(unit['id'])
    records = report.get('records')
    if not isinstance(family, str) or not family or not isinstance(records, list) or not records:
        raise ValueError('missing parser-family campaign records')
    selected = []
    identities = set()
    for row in records:
        if not isinstance(row, dict) or type(row.get('sabotage')) is not bool:
            raise ValueError('invalid campaign record')
        identity = (row.get('unit'), row.get('seed'), row['sabotage'])
        if identity in identities:
            raise ValueError('duplicate campaign seed/run')
        identities.add(identity)
        if any(type(row.get(k)) is not int or row[k] < 0 for k in COUNTERS) or row['cases'] == 0:
            raise ValueError('invalid/empty fuzz counters')
        if sum(row[k] for k in COUNTERS[2:]) != row['cases']:
            raise ValueError('fuzz outcomes do not partition all inputs')
        raw = blob(local(path.parent, row.get('log')))
        if hashlib.sha256(raw).hexdigest() != row.get('log_sha256'):
            raise ValueError('fuzz log hash differs')
        lines = SUMMARY.findall(raw)
        if len(lines) != 1 or lines[0][0].decode() != row['unit'] or dict(zip(COUNTERS, map(int, lines[0][1:]))) != {k: row[k] for k in COUNTERS}:
            raise ValueError('fuzz log does not support counters')
        if row['sabotage'] and row['divergences'] == 0:
            raise ValueError('corruption canary found no divergence')
        if not row['sabotage'] and row['divergences']:
            raise ValueError('ordinary fuzz campaign diverged')
        if row['unit'] == family:
            selected.append(row)
    canaries = report.get('canary_reproductions', [])
    reduced = [r for r in canaries if r.get('unit') == family]
    if len(reduced) != 1:
        raise ValueError('missing actual corruption reproducer')
    files = reduced[0].get('files', {})
    required = {'input.bin', 'original.bin', 'rust.bin', 'original.status.txt', 'rust.status.txt', 'original.message.txt', 'rust.message.txt', 'original.progress.txt', 'rust.progress.txt'}
    if not required <= files.keys():
        raise ValueError('incomplete canary values/state')
    decoded = {}
    for name, item in files.items():
        raw = bytes.fromhex(item['hex'])
        if hashlib.sha256(raw).hexdigest() != item.get('sha256'):
            raise ValueError('canary artifact hash differs')
        decoded[name] = raw
    if any(decoded[n + '.status.txt'] != b'ok=true timeout=false\n' for n in ['original', 'rust']):
        raise ValueError('canary is not an actual successful value comparison')
    if all(decoded['original.'+suffix] == decoded['rust.'+suffix] for suffix in ['bin','message.txt','progress.txt']):
        raise ValueError('corruption reproducer has no value/state difference')
    normal = [r for r in selected if not r['sabotage']]
    canary = [r for r in selected if r['sabotage']]
    if not normal or not canary or sum(r['identical_success'] for r in normal) == 0:
        raise ValueError('fuzz needs positive successful comparisons and a corruption canary')
    # Host refusals/timeouts are recorded but never counted as comparisons.
    if sum(r['identical_success'] + r['identical_refusal'] for r in normal) == 0:
        raise ValueError('zero original/Rust comparisons')


def review(unit, baseline, plan):
    path, report = proof(unit, 'coverage-review', baseline, plan)
    cases(report)
    if (report.get('profile') != 'comparison-only' or not isinstance(report.get('reviewed_by'), str)
            or not report['reviewed_by'].strip() or report.get('unresolved') != []
            or report.get('unknown_reach') is not False):
        raise ValueError('coverage is not explicitly reviewed/resolved')
    for key in ('original_coverage', 'rust_coverage'):
        item = report.get(key)
        if not isinstance(item, dict):
            raise ValueError('coverage review needs both engine profiles')
        data = local(path.parent, item.get('location')).read_bytes()
        if not data or hashlib.sha256(data).hexdigest() != item.get('sha256'):
            raise ValueError('coverage profile missing or changed')


def validate(unit, baseline, plan=None):
    level = unit.get('verification')
    if level not in LEVELS:
        return [str(unit.get('id', '<missing>')) + ': missing/unknown verification level']
    errors = []
    if unit.get('status') == 'accepted' and level != LEVELS[-1]:
        errors.append(unit['id'] + ': accepted requires coverage-reviewed verification')
    try:
        if LEVELS.index(level) >= 1:
            fixed(unit, baseline, plan)
        if LEVELS.index(level) >= 2:
            fuzz(unit, baseline, plan)
        if LEVELS.index(level) >= 3:
            review(unit, baseline, plan)
    except (ValueError, TypeError, KeyError, OSError, UnicodeError, EOFError) as error:
        errors.append(unit['id'] + ': unsupported ' + level + ' verification: ' + str(error))
    return errors


def candidate(unit, plan):
    level = unit.get('verification', 'none')
    if level == 'none':
        return None
    if level == 'fixed-corpus':
        return unit.get('implemented_commit')
    kind = 'coverage-review' if level == 'coverage-reviewed' else 'fuzz-campaign'
    _, report = proof(unit, kind, unit['reference_sha'], plan)
    return report['candidate_sha']
