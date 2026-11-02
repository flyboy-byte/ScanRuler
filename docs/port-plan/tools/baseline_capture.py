# SPDX-License-Identifier: AGPL-3.0-only
"""Inventory actual original results without turning exclusions/failures into passes.

Temporary Python bootstrap (P39); not a product dependency. Run records are supplied
by the operator from completed commands, not inferred from a green aggregate.
"""
import argparse
import datetime
import hashlib
import json
from pathlib import Path
import platform
import re
import os
import shutil
import subprocess

REFERENCE = '2cd89c6769a10c27ee7754b477ada521c9ef5cb7'
ORIGINAL = ['src', 'tests', 'scripts', 'plugins', 'mcp/src', 'mcp/tests',
            'package.json', 'package-lock.json', 'mcp/package.json', 'mcp/package-lock.json']
GUARD = re.compile(r'\b(?:describe|it|test)\.(?:skipIf|runIf|skip|todo)\b')
REQUIRED = {'plugin-boundary', 'display-units', 'unit', 'mcp', 'build', 'browser'}


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def git(root, *args):
    return subprocess.check_output(['git', *args], cwd=root, text=True).strip()


def check_sites(root, contract):
    actual = []
    for directory in ['tests', 'plugins']:
        for path in sorted((root / directory).rglob('*.test.ts')):
            for line in path.read_text().splitlines():
                if GUARD.search(line):
                    actual.append((path.relative_to(root).as_posix(), line.strip()))
    expected = [(s['path'], s['guard']) for s in contract['sites']]
    if len(set(expected)) != len(expected) or sorted(actual) != sorted(expected):
        raise ValueError('original exclusion guards changed or are missing/duplicated')
    if any(not s.get('reason', '').strip() for s in contract['sites']):
        raise ValueError('every exclusion guard needs an explicit reason')


def unit_results(root, data, contract):
    tests = [dict(t, source=Path(f['name']).relative_to(root).as_posix())
             for f in data['testResults'] for t in f['assertionResults']]
    if not tests or len(tests) != data['numTotalTests']:
        raise ValueError('missing or inconsistent original test results')
    if any(t['status'] not in ('passed', 'failed', 'pending', 'todo', 'skipped') for t in tests):
        raise ValueError('unknown original test status')
    reasons = {}
    for s in contract['sites']:
        reasons.setdefault(s['path'], []).append({'guard': s['guard'], 'reason': s['reason']})
    exclusions = []
    for t in tests:
        if t['status'] in ('pending', 'todo', 'skipped'):
            matches = [s for s in reasons.get(t['source'], [])
                       if (m := re.search(r"\)\('([^']+)'", s['guard'])) and m[1] in t['fullName']]
            if not matches:
                raise ValueError('unmapped excluded original test: ' + t['source'])
            exclusions.append({'source': t['source'], 'name': t['fullName'],
                               'status': t['status'], 'source_guards': matches})
    return {'total': len(tests), 'passed': sum(t['status'] == 'passed' for t in tests),
            'failed': sum(t['status'] == 'failed' for t in tests), 'excluded': exclusions,
            'failures': [{'source': t['source'], 'name': t['fullName']} for t in tests if t['status'] == 'failed']}


def run_records(records):
    if len(records) != len(REQUIRED) or {r['id'] for r in records} != REQUIRED:
        raise ValueError('missing or duplicate complete-original run record')
    out = []
    for r in records:
        if not isinstance(r['exit_code'], int) or not r.get('command'):
            raise ValueError('run must record an actual command and integer exit status')
        log = Path(r['log'])
        if not log.is_file() or not log.stat().st_size:
            raise ValueError('missing or empty original run log: ' + r['id'])
        out.append({'id': r['id'], 'command': r['command'], 'exit_code': r['exit_code'],
                    'log': log.name, 'sha256': digest(log), 'bytes': log.stat().st_size})
    return out


def capture(root, units, records, generated):
    contract_path = root / 'docs/port-plan/ci/original-exclusions.json'
    contract = json.loads(contract_path.read_text())
    if contract['reference_sha'] != REFERENCE:
        raise ValueError('exclusion contract has wrong reference')
    check_sites(root, contract)
    changed = git(root, 'diff', '--name-only', REFERENCE, '--', *ORIGINAL)
    if changed:
        raise ValueError('original differs from pinned tree: ' + changed)
    summary = unit_results(root, json.loads(units.read_text()), contract)
    runs = run_records(records)
    unit = next(r for r in runs if r['id'] == 'unit')
    if (unit['exit_code'] == 0) != (summary['failed'] == 0):
        raise ValueError('unit exit status disagrees with individual results')
    if generated.get('generator_sha256') != digest(root / 'scripts/e2e-fixtures.mjs'):
        raise ValueError('stale generated-fixture source identity')
    if len({a['id'] for a in generated['assets']}) != len(generated['assets']):
        raise ValueError('duplicate generated fixture identity')
    for asset in generated['assets']:
        if not asset.get('recipe') or not re.fullmatch('[0-9a-f]{64}', asset.get('sha256', '')):
            raise ValueError('generated asset needs recipe and content hash')
        if asset.get('license') != 'AGPL-3.0-only':
            raise ValueError('unexpected generated asset provenance')
        if asset.get('repeated_bytes_equal') is not True:
            raise ValueError('fixture repeatability was not verified')
    if not generated['assets']:
        raise ValueError('zero generated fixtures')
    return {'schema': 1, 'reference_sha': REFERENCE, 'candidate_sha': git(root, 'rev-parse', 'HEAD'),
            'observed_at': datetime.datetime.now(datetime.timezone.utc).isoformat(),
            'environment': {'system': platform.system(), 'release': platform.release(),
                            'machine': platform.machine(),
                            'node': subprocess.check_output(['node', '--version'], text=True).strip(),
                            'npm': subprocess.check_output(['npm', '--version'], text=True).strip(),
                            'rustc': subprocess.check_output(['rustc', '--version'], text=True).strip(),
                            'chrome': subprocess.check_output([os.environ.get('CHROME') or shutil.which('google-chrome') or shutil.which('google-chrome-stable') or '/opt/google/chrome/chrome', '--version'], text=True).strip()},
            'original_paths_identical': ORIGINAL,
            'locks': {p: digest(root / p) for p in ['package-lock.json', 'mcp/package-lock.json', 'rust/Cargo.lock']},
            'exclusion_contract_sha256': digest(contract_path),
            'unit_results_sha256': digest(units), 'unit_results': summary,
            'runs': runs, 'all_original_commands_passed': all(r['exit_code'] == 0 for r in runs),
            'generated_fixtures': generated,
            'private_root_fixtures': [{'name': p, 'present': (root / p).exists(),
                                       'redistribution': 'not authorized; no private bytes or hashes collected'}
                                      for p in ['ballbar.stl', 'side bracket left.stl', 'block-marius.stl']]}


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--root', type=Path, default=Path.cwd())
    p.add_argument('--units', type=Path, required=True)
    p.add_argument('--runs', type=Path, required=True)
    p.add_argument('--fixtures', type=Path, required=True)
    p.add_argument('--out', type=Path, required=True)
    a = p.parse_args()
    if int(subprocess.check_output(['node', '--version'], text=True).strip()[1:].split('.')[0]) < 22:
        p.error('use Node 22+ on PATH (fnm exec --using=22 python3 ...)')
    report = capture(a.root.resolve(), a.units, json.loads(a.runs.read_text()), json.loads(a.fixtures.read_text()))
    a.out.write_text(json.dumps(report, indent=2) + '\n')
    print('Recorded original commands:', len(report['runs']), '; all passed:', report['all_original_commands_passed'])


if __name__ == '__main__':
    main()
