#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-only
"""Temporary bootstrap inventory/status tooling; not a numerical comparator or P37 ratchet."""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[3]
PLAN = ROOT / 'docs/port-plan'


def units(plan=PLAN):
    return [json.loads(p.read_text()) for p in sorted((plan / 'plan/units').glob('*.json'))]


def roster(root=ROOT):
    plan = root / 'docs/port-plan'
    suites = json.loads((plan / 'ci/native-suites.json').read_text())['suites']
    if not suites:
        raise ValueError('Empty suite roster')
    for key in ['test', 'unit', 'directory']:
        values = [s[key] for s in suites]
        if len(set(values)) != len(values):
            raise ValueError('Duplicate roster ' + key)
    tests = re.findall(r'^    "([a-z_][a-z0-9_:]+)",$', (root / 'rust/xtask/src/main.rs').read_text(), re.M)
    if set(tests) != {s['test'] for s in suites}:
        raise ValueError('Suite roster differs from Rust discovery guard')
    by_id = {u['id']: u for u in units(plan)}
    for s in suites:
        if not re.fullmatch(r'[a-z][a-z-]*-diff', s['directory']):
            raise ValueError('Unsafe evidence directory')
        cases = [e['case_ids'] for e in by_id[s['unit']]['evidence'] if e['kind'] == 'differential']
        if len(cases) != 1 or not cases[0] or len(set(cases[0])) != len(cases[0]):
            raise ValueError('Missing/ambiguous/duplicate case inventory: ' + s['unit'])
        s['case_ids'] = sorted(cases[0])
        if any(not re.fullmatch(r'[A-Za-z0-9_.-]+', c) for c in s['case_ids']):
            raise ValueError('Unsafe case identity')
        if not set(s['refusals_without_rust_bin']) <= set(s['case_ids']):
            raise ValueError('Unknown declared refusal')
    numerical = {u['id'] for u in by_id.values() if any(e['kind'] == 'differential' for e in u['evidence'])}
    if numerical != {s['unit'] for s in suites}:
        raise ValueError('Numerical unit missing from roster')
    return suites


def status(root=ROOT):
    plan = root / 'docs/port-plan'
    all_units = units(plan)
    suites = roster(root)
    return {'contract': 'port.status.v1', 'reference_sha': json.loads((plan / 'reference-pin.json').read_text())['commit'],
            'unit_states': dict(sorted(Counter(u['status'] for u in all_units).items())),
            'native_suites': len(suites), 'native_case_identities_including_refusals': sum(len(s['case_ids']) for s in suites),
            'accepted_units': [u['id'] for u in all_units if u['status'] == 'accepted'],
            'units': [{'id': u['id'], 'status': u['status'], 'parent': u.get('parent'), 'pr': u.get('claim_pr'),
                       'tested_sha': u.get('implemented_commit'), 'open_items': u.get('open_items', [])} for u in all_units]}


def status_markdown(report):
    lines = ['# Current Rust port status', '', 'Generated from canonical unit files and ci/native-suites.json; do not hand-edit.', '',
             f"Reference: `{report['reference_sha']}`.", '',
             f"Native coverage inventory: **{report['native_suites']} suites / {report['native_case_identities_including_refusals']} case identities including refusals**. This counts declared cases, not successful measurements or feature completion.", '',
             'No aggregate percentage establishes parity. Unit states describe evidence scope; hosted run outcomes are in [AUDIT-QUEUE.md](AUDIT-QUEUE.md). Browser-Wasm execution, engine/session/UI integration and the exact-base numerical ratchet remain open.', '',
             '| State | Units |', '| --- | ---: |']
    lines += [f'| {state} | {count} |' for state, count in report['unit_states'].items()]
    lines += ['', '| Unit | State | Tested implementation | PR |', '| --- | --- | --- | --- |']
    for u in report['units']:
        sha = f"`{u['tested_sha'][:7]}`" if u['tested_sha'] else '—'
        pr = f"[#{u['pr'].split('/')[-1]}]({u['pr']})" if u['pr'] else '—'
        lines.append(f"| [{u['id']}](plan/units/{u['id']}.json) | {u['status']} | {sha} | {pr} |")
    return '\n'.join(lines) + '\n'


def evidence(target, suites, candidate, reference):
    if not re.fullmatch(r'[0-9a-f]{40}', candidate):
        raise ValueError('Expected full candidate SHA')
    results = []
    for suite in suites:
        runs = list((target / suite['directory']).glob('run-*'))
        if not runs:
            raise ValueError('Missing suite output: ' + suite['unit'])
        run = max(runs, key=lambda p: int(p.name.removeprefix('run-')))
        if run.is_symlink():
            raise ValueError('Unexpected linked run')
        marker = run / 'candidate.txt'
        if marker.is_file():
            identity = marker.read_text().strip()
        else:
            text = (run / 'identity.txt').read_text()
            identity = re.search(r'^candidate: ([0-9a-f]{40})$', text, re.M)
            if not identity or f'reference: {reference}\n' not in text:
                raise ValueError('Missing reference/candidate identity')
            identity = identity[1]
        if identity != candidate:
            raise ValueError('Stale candidate output: ' + suite['unit'])
        actual = sorted(p.name for p in run.iterdir() if p.is_dir())
        if actual != suite['case_ids']:
            raise ValueError('Missing/extra cases: ' + suite['unit'])
        cases = []
        for case_id in actual:
            directory = run / case_id
            if directory.is_symlink():
                raise ValueError('Unexpected linked case')
            required = ['input.bin', 'original.bin']
            if case_id in suite['refusals_without_rust_bin']:
                required += ['original.progress-or-error.txt']
                required += ['rust.progress.txt' if suite['directory'] == 'stl-diff' else 'rust.progress-or-error.txt']
            else:
                required += ['rust.bin']
            if any(not (directory / name).is_file() for name in required):
                raise ValueError('Missing raw comparison file: ' + case_id)
            files = {}
            for path in sorted(directory.iterdir()):
                if path.is_symlink() or not path.is_file():
                    raise ValueError('Unexpected evidence entry: ' + str(path))
                data = path.read_bytes()
                files[path.name] = {'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest()}
            cases.append({'id': case_id, 'files': files})
        results.append({'unit': suite['unit'], 'test': suite['test'], 'run': str(run), 'cases': cases})
    return {'contract': 'native.oracle.artifacts.v1', 'candidate_sha': candidate, 'reference_sha': reference,
            'meaning': 'Artifact identity/completeness only; numerical assertions run in cargo xtask oracle. Not an exact-base ratchet.', 'suites': results}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', choices=['status', 'evidence'])
    parser.add_argument('--check', action='store_true')
    parser.add_argument('--candidate')
    parser.add_argument('--target', type=Path, default=ROOT / 'rust/target')
    parser.add_argument('--output', type=Path, default=ROOT / 'rust/target/ci/native-evidence.json')
    args = parser.parse_args()
    if args.command == 'status':
        report = status()
        outputs = {PLAN / 'status.json': json.dumps(report, indent=2) + '\n', PLAN / 'STATUS.md': status_markdown(report)}
        for path, text in outputs.items():
            if args.check:
                if not path.exists() or path.read_text() != text:
                    raise SystemExit('Stale generated status: ' + str(path))
            else:
                path.write_text(text)
        print(f"Status inventory valid: {report['native_suites']} suites; no runtime/acceptance claim")
    else:
        report = evidence(args.target, roster(), args.candidate, status()['reference_sha'])
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(report, indent=2) + '\n')
        print(f"Raw evidence complete for {len(report['suites'])} suites at {args.candidate}")


if __name__ == '__main__':
    main()
