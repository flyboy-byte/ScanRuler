#!/usr/bin/env python3
"""Live plan checker/generator; temporary bootstrap tooling, not a parity harness."""
import argparse
import copy
import json
import re
from pathlib import Path


def validate(metadata, units):
    errors = []
    by_id = {}
    statuses = set(metadata.get("statuses", []))
    for unit in units:
        key = unit.get("id", "")
        if not re.fullmatch(r"(?:P\d{2}|[a-z][a-z0-9_]*(?:\.[a-z0-9_]+)+)", key):
            errors.append(f"invalid unit ID {key!r}")
        if key in by_id:
            errors.append(f"duplicate ID {key}")
        by_id[key] = unit
        for field in ("title", "scope", "acceptance"):
            if not unit.get(field):
                errors.append(f"{key}: missing {field}")
        if unit.get("status") not in statuses:
            errors.append(f"{key}: unknown status")
        if not isinstance(unit.get("depends_on"), list):
            errors.append(f"{key}: depends_on must be a list")
        if unit.get("reference_sha") != metadata.get("baseline"):
            errors.append(f"{key}: reference SHA differs from pinned baseline")
        if unit.get("status") in {"compared", "integrated", "accepted"}:
            if not re.fullmatch(r"[0-9a-f]{40}", str(unit.get("implemented_commit", ""))):
                errors.append(f"{key}: verified state needs full tested implementation SHA")
            if not unit.get("evidence"):
                errors.append(f"{key}: verified state has no evidence")
        if unit.get("status") == "accepted" and unit.get("open_items"):
            errors.append(f"{key}: accepted with open items")
        for evidence in unit.get("evidence", []):
            if not isinstance(evidence, dict) or not evidence.get("kind") or not evidence.get("location"):
                errors.append(f"{key}: evidence needs kind/location")
                continue
            if evidence.get("candidate_sha") != unit.get("implemented_commit"):
                errors.append(f"{key}: evidence does not match tested implementation SHA")
            if evidence["kind"] == "differential":
                if evidence.get("reference_sha") != metadata.get("baseline") or not evidence.get("case_ids"):
                    errors.append(f"{key}: differential evidence missing pinned reference/cases")
    pending = {}
    for key, unit in by_id.items():
        deps = unit.get("depends_on", [])
        if not isinstance(deps, list):
            deps = []
        if len(set(deps)) != len(deps):
            errors.append(f"{key}: repeated dependency")
        pending[key] = set(deps)
        for dep in deps + unit.get("soft_depends_on", []):
            if dep not in by_id:
                errors.append(f"{key}: unknown dependency {dep}")
        if unit.get("parent") and unit["parent"] not in by_id:
            errors.append(f"{key}: unknown parent")
        if unit.get("status") == "accepted":
            for dep in deps:
                if by_id.get(dep, {}).get("status") != "accepted":
                    errors.append(f"{key}: hard dependency {dep} is not accepted")
    while pending:
        ready = {key for key, deps in pending.items() if not deps}
        if not ready:
            errors.append("hard dependency graph is cyclic or unresolved")
            break
        pending = {key: deps - ready for key, deps in pending.items() if key not in ready}
    return errors


def selftest():
    meta = {"baseline": "a" * 40, "statuses": ["planned", "compared", "accepted"]}
    base = {"id": "P00", "title": "Baseline", "scope": "Prepare reference", "acceptance": ["Captured"], "status": "planned", "depends_on": [], "reference_sha": meta["baseline"], "evidence": [], "implemented_commit": None, "open_items": []}
    if validate(meta, [base]):
        raise SystemExit("selftest: valid planned unit rejected")
    mutants = []
    def changed(**fields):
        value = copy.deepcopy(base)
        value.update(fields)
        return [value]
    mutants += [changed(depends_on=["P99"]), changed(depends_on=["P00"]), changed(status="accepted"), changed(reference_sha="b" * 40), changed(title=""), [base, copy.deepcopy(base)]]
    proof = {"kind": "differential", "location": "artifact://example", "candidate_sha": "c" * 40, "reference_sha": meta["baseline"], "case_ids": ["plane.exact.1"]}
    good = changed(status="accepted", implemented_commit="c" * 40, evidence=[proof])
    if validate(meta, good):
        raise SystemExit("selftest: valid evidence metadata rejected")
    bad = copy.deepcopy(good)
    bad[0]["open_items"] = ["missing success path"]
    mutants.append(bad)
    bad = copy.deepcopy(good)
    bad[0]["evidence"][0]["candidate_sha"] = "d" * 40
    mutants.append(bad)
    bad = copy.deepcopy(good)
    bad[0]["evidence"][0]["case_ids"] = []
    mutants.append(bad)
    for index, mutant in enumerate(mutants):
        if not validate(meta, mutant):
            raise SystemExit(f"selftest: invalid plan mutation {index} escaped")
    print(f"Plan checker self-test passed: {len(mutants)} invalid-state mutations rejected. This is not a geometry/parity test.")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("command", choices=["check", "check-generated", "render", "selftest"])
    args = parser.parse_args()
    if args.command == "selftest":
        selftest()
        return
    metadata = json.loads((args.root / "backlog.json").read_text())
    paths = sorted((args.root / "plan/units").glob("*.json"))
    if not paths:
        raise SystemExit("No canonical unit files found; refusing an empty plan")
    units = []
    for path in paths:
        unit = json.loads(path.read_text())
        if path.stem != unit.get("id"):
            raise SystemExit(f"Unit filename/ID mismatch: {path}")
        units.append(unit)
    errors = validate(metadata, units)
    if errors:
        raise SystemExit("Plan invalid:\n" + "\n".join(errors))
    if args.command == "check-generated" and metadata.get("tasks") != units:
        raise SystemExit("Generated backlog is stale; run port_plan.py --root docs/port-plan render")
    if args.command == "render":
        metadata["tasks"] = units
        metadata["generated_from"] = "plan/units/*.json"
        (args.root / "backlog.json").write_text(json.dumps(metadata, indent=2) + "\n")
    print(f"Live plan valid: {len(units)} units. Evidence metadata checked; runtime results must be validated by product CI.")


if __name__ == "__main__":
    main()
